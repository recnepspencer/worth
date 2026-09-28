use super::{
    UiAdmittedAllocationSourceOrder, UiAllocationFrameEpoch, UiAllocationFrameIngressDescriptor,
    UiAllocationFrameIngressSequence, UiAllocationFrameSourceGeneration,
    UiAllocationFrameSourceIdentity, UiAllocationFrameSourceLane,
};

const RETRY_LEDGER_CAPACITY: usize = 64;

#[cfg(test)]
mod test_support;
#[cfg(test)]
pub use test_support::{
    UiAllocationFrameSourceRetirementDenial, UiAllocationFrameSourceRetirementOutcome,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiAllocationFrameSourceDomain {
    lane: UiAllocationFrameSourceLane,
    identity: UiAllocationFrameSourceIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiAllocationFrameRetiredSourceOrder {
    domain: UiAllocationFrameSourceDomain,
    generation: UiAllocationFrameSourceGeneration,
    order: UiAdmittedAllocationSourceOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UiAllocationFrameRetryAssignment {
    descriptor: UiAllocationFrameIngressDescriptor,
    epoch: UiAllocationFrameEpoch,
    status: UiAllocationFrameRetryAssignmentStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiAllocationFrameRetryAssignmentStatus {
    Pending,
    Assigned(UiAllocationFrameIngressSequence),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationFrameRetryState {
    active_epoch: UiAllocationFrameEpoch,
    assignment_len: u16,
    assignments: [Option<UiAllocationFrameRetryAssignment>; RETRY_LEDGER_CAPACITY],
    retired_len: u16,
    retired: [Option<UiAllocationFrameRetiredSourceOrder>; RETRY_LEDGER_CAPACITY],
}

impl UiAllocationFrameRetryState {
    pub(super) fn empty(active_epoch: UiAllocationFrameEpoch) -> Self {
        Self {
            active_epoch,
            assignment_len: 0,
            assignments: std::array::from_fn(|_| None),
            retired_len: 0,
            retired: std::array::from_fn(|_| None),
        }
    }

    pub(super) fn assignment_for_ingress_identity(
        &self,
        descriptor: UiAllocationFrameIngressDescriptor,
    ) -> (Option<UiAllocationFrameRetryAssignment>, u64) {
        let mut slots_scanned = 0;
        for assignment in self.assignments().iter().flatten() {
            slots_scanned += 1;
            if source_domain(&assignment.descriptor) == source_domain(&descriptor)
                && assignment.descriptor.key().source_generation()
                    == descriptor.key().source_generation()
                && assignment.descriptor.key().ingress_identity()
                    == descriptor.key().ingress_identity()
            {
                return (Some(assignment.clone()), slots_scanned);
            }
        }
        (None, slots_scanned)
    }

    pub(super) fn assignment_for_source_position(
        &self,
        descriptor: UiAllocationFrameIngressDescriptor,
    ) -> (Option<UiAllocationFrameRetryAssignment>, u64) {
        let mut slots_scanned = 0;
        for assignment in self.assignments().iter().flatten() {
            slots_scanned += 1;
            if source_domain(&assignment.descriptor) == source_domain(&descriptor)
                && assignment.descriptor.key().source_generation()
                    == descriptor.key().source_generation()
                && assignment.descriptor.key().source_order() == descriptor.key().source_order()
            {
                return (Some(assignment.clone()), slots_scanned);
            }
        }
        (None, slots_scanned)
    }

    pub(super) fn is_retired(&self, descriptor: UiAllocationFrameIngressDescriptor) -> (bool, u64) {
        let domain = source_domain(&descriptor);
        let mut comparisons = 0;
        for entry in self.retired().iter().flatten() {
            comparisons += 1;
            if entry.domain == domain {
                return (
                    descriptor.key().source_generation() < entry.generation
                        || (descriptor.key().source_generation() == entry.generation
                            && descriptor.key().source_order() <= entry.order),
                    comparisons,
                );
            }
        }
        (false, comparisons)
    }

    pub(super) fn can_track(&self, descriptor: UiAllocationFrameIngressDescriptor) -> (bool, u64) {
        let domain = source_domain(&descriptor);
        let mut comparisons = 0;
        if self.retired().iter().flatten().any(|entry| {
            comparisons += 1;
            entry.domain == domain
        }) || self.assignments().iter().flatten().any(|assignment| {
            comparisons += 1;
            source_domain(&assignment.descriptor) == domain
        }) {
            return (true, comparisons);
        }
        let mut reserved_domains = 0;
        for (index, assignment) in self.assignments().iter().enumerate() {
            comparisons += 1;
            let Some(assignment) = assignment else {
                continue;
            };
            let assigned_domain = source_domain(&assignment.descriptor);
            let already_retired = self.retired().iter().flatten().any(|entry| {
                comparisons += 1;
                entry.domain == assigned_domain
            });
            let already_reserved = self.assignments()[..index].iter().flatten().any(|prior| {
                comparisons += 1;
                source_domain(&prior.descriptor) == assigned_domain
            });
            if !already_retired && !already_reserved {
                reserved_domains += 1;
            }
        }
        (
            usize::from(self.retired_len) + reserved_domains < RETRY_LEDGER_CAPACITY,
            comparisons,
        )
    }

    pub(super) fn begin_epoch(&mut self, epoch: UiAllocationFrameEpoch) -> (u64, u64) {
        if self.active_epoch == epoch {
            return (0, 0);
        }
        let mut comparisons = 0;
        let mut writes = 0;
        for index in 0..usize::from(self.assignment_len) {
            comparisons += 1;
            let assignment = self.assignments[index]
                .take()
                .expect("active retry assignment slots are populated");
            writes += 1;
            let (retire_comparisons, retire_writes) = self.retire(assignment.descriptor);
            comparisons += retire_comparisons;
            writes += retire_writes;
        }
        self.assignment_len = 0;
        self.active_epoch = epoch;
        writes += 2;
        (comparisons, writes)
    }

    #[cfg(test)]
    pub(super) fn retire_source(
        &mut self,
        retirement: &super::UiAllocationFrameSourceLease,
    ) -> Result<(), UiAllocationFrameSourceRetirementDenial> {
        let domain = UiAllocationFrameSourceDomain {
            lane: retirement.source_lane(),
            identity: retirement.source_identity(),
        };
        let active_generation = self.assignments().iter().flatten().find_map(|assignment| {
            (source_domain(&assignment.descriptor) == domain)
                .then_some(assignment.descriptor.key().source_generation())
        });
        let retired_generation = self
            .retired()
            .iter()
            .flatten()
            .find_map(|entry| (entry.domain == domain).then_some(entry.generation));
        let Some(tracked_generation) = active_generation.or(retired_generation) else {
            return Err(UiAllocationFrameSourceRetirementDenial::NotTracked);
        };
        if tracked_generation != retirement.source_generation()
            || active_generation.is_some_and(|generation| generation != tracked_generation)
            || retired_generation.is_some_and(|generation| generation != tracked_generation)
        {
            return Err(
                UiAllocationFrameSourceRetirementDenial::GenerationMismatch { tracked_generation },
            );
        }
        self.remove_assignments_for(domain.clone());
        self.remove_retired_for(domain);
        Ok(())
    }

    #[cfg(test)]
    fn remove_assignments_for(&mut self, domain: UiAllocationFrameSourceDomain) {
        let len = usize::from(self.assignment_len);
        let mut retained = 0;
        for index in 0..len {
            let assignment = self.assignments[index]
                .take()
                .expect("active retry assignment slots are populated");
            if source_domain(&assignment.descriptor) != domain {
                self.assignments[retained] = Some(assignment);
                retained += 1;
            }
        }
        self.assignments[retained..len].fill(None);
        self.assignment_len = u16::try_from(retained)
            .expect("the retry ledger holds at most RETRY_LEDGER_CAPACITY entries");
    }

    #[cfg(test)]
    fn remove_retired_for(&mut self, domain: UiAllocationFrameSourceDomain) {
        let len = usize::from(self.retired_len);
        let mut retained = 0;
        for index in 0..len {
            let entry = self.retired[index]
                .take()
                .expect("retired retry slots are populated");
            if entry.domain != domain {
                self.retired[retained] = Some(entry);
                retained += 1;
            }
        }
        self.retired[retained..len].fill(None);
        self.retired_len = u16::try_from(retained)
            .expect("the retry ledger holds at most RETRY_LEDGER_CAPACITY entries");
    }

    pub(super) fn record(
        &mut self,
        descriptor: UiAllocationFrameIngressDescriptor,
        epoch: UiAllocationFrameEpoch,
    ) -> (u64, u64) {
        let index = usize::from(self.assignment_len);
        debug_assert!(index < RETRY_LEDGER_CAPACITY);
        self.assignments[index] = Some(UiAllocationFrameRetryAssignment {
            descriptor,
            epoch,
            status: UiAllocationFrameRetryAssignmentStatus::Pending,
        });
        self.assignment_len += 1;
        (0, 2)
    }

    pub(super) fn commit_sealed_assignment(
        &mut self,
        sealed: super::UiAllocationFrameSubmissionAssignment,
    ) -> (u64, u64) {
        let mut comparisons = 0;
        for assignment in self.assignments[..usize::from(self.assignment_len)]
            .iter_mut()
            .flatten()
        {
            comparisons += 1;
            if assignment.descriptor.key() == sealed.clone().ingress_key() {
                assignment.status =
                    UiAllocationFrameRetryAssignmentStatus::Assigned(sealed.sequence());
                return (comparisons, 1);
            }
        }
        (comparisons, 0)
    }

    pub(super) fn discard_pending(
        &mut self,
        disposed: super::UiAllocationFrameIngressView<'_>,
    ) -> (u64, u64) {
        let len = usize::from(self.assignment_len);
        let mut comparisons = 0;
        let mut writes = 0;
        let mut retained = 0;
        for index in 0..len {
            comparisons += 1;
            let assignment = self.assignments[index]
                .take()
                .expect("active retry assignment slots are populated");
            writes += 1;
            let discard = matches!(
                assignment.status,
                UiAllocationFrameRetryAssignmentStatus::Pending
            ) && disposed.iter().any(|ingress| {
                comparisons += 1;
                ingress.key() == assignment.descriptor.key()
            });
            if !discard {
                self.assignments[retained] = Some(assignment);
                retained += 1;
                writes += 1;
            }
        }
        self.assignments[retained..len].fill(None);
        self.assignment_len = u16::try_from(retained)
            .expect("the retry ledger holds at most RETRY_LEDGER_CAPACITY entries");
        writes += (len - retained) as u64 + 1;
        (comparisons, writes)
    }

    fn retire(&mut self, descriptor: UiAllocationFrameIngressDescriptor) -> (u64, u64) {
        let domain = source_domain(&descriptor);
        let mut comparisons = 0;
        for entry in self.retired[..usize::from(self.retired_len)]
            .iter_mut()
            .flatten()
        {
            comparisons += 1;
            if entry.domain != domain {
                continue;
            }
            if descriptor.key().source_generation() > entry.generation {
                entry.generation = descriptor.key().source_generation();
                entry.order = descriptor.key().source_order();
                return (comparisons, 2);
            } else if descriptor.key().source_generation() == entry.generation {
                entry.order = entry.order.max(descriptor.key().source_order());
                return (comparisons, 1);
            }
            return (comparisons, 0);
        }
        let index = usize::from(self.retired_len);
        debug_assert!(index < RETRY_LEDGER_CAPACITY);
        self.retired[index] = Some(UiAllocationFrameRetiredSourceOrder {
            domain,
            generation: descriptor.key().source_generation(),
            order: descriptor.key().source_order(),
        });
        self.retired_len += 1;
        (comparisons, 2)
    }

    fn assignments(&self) -> &[Option<UiAllocationFrameRetryAssignment>] {
        &self.assignments[..usize::from(self.assignment_len)]
    }

    fn retired(&self) -> &[Option<UiAllocationFrameRetiredSourceOrder>] {
        &self.retired[..usize::from(self.retired_len)]
    }
}

fn source_domain(descriptor: &UiAllocationFrameIngressDescriptor) -> UiAllocationFrameSourceDomain {
    UiAllocationFrameSourceDomain {
        lane: descriptor.key().source_lane(),
        identity: descriptor.key().source_identity(),
    }
}

impl UiAllocationFrameRetryAssignment {
    pub(super) fn descriptor(&self) -> UiAllocationFrameIngressDescriptor {
        self.descriptor.clone()
    }

    pub(super) fn epoch(&self) -> UiAllocationFrameEpoch {
        self.epoch
    }

    pub(super) fn sequence(&self) -> Option<UiAllocationFrameIngressSequence> {
        match self.status {
            UiAllocationFrameRetryAssignmentStatus::Pending => None,
            UiAllocationFrameRetryAssignmentStatus::Assigned(sequence) => Some(sequence),
        }
    }
}

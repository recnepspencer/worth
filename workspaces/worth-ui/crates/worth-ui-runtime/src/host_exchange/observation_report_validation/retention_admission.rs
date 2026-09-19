use worth_ui_host_contract::{
    UiHostObservationCoalescingIdentity, UiHostObservationFamily, UiHostObservationReport,
    UiHostObservationSequence, UiHostObservationSequenceRange,
};

use super::basis_admission::UiBasisAdmittedObservationBatch;
use super::state::{
    UiHostObservationBatchFingerprint, UiHostObservationPartition, UiRetainedHostObservationReport,
    UiRetainedObservationBasis,
};
use super::{
    UiDuplicateHostObservationBatch, UiHostObservationBatchDisposition,
    UiHostObservationDisposition, UiHostObservationFrameRelation, UiHostObservationReportDenial,
    UiHostObservationReportOutcome, UiHostObservationReportValidation,
    UiValidatedHostObservationBatch, UiValidatedHostObservationReport,
};

impl UiHostObservationReportValidation {
    pub(super) fn retain_covered_batch(
        &mut self,
        admitted: UiBasisAdmittedObservationBatch,
    ) -> Result<UiHostObservationReportOutcome, UiHostObservationReportDenial> {
        let (batch, relation, observation_basis) = admitted.into_parts();
        let core = batch.core();
        let binding = core.binding();
        let frame = core.frame();
        let mut partition = self
            .partitions
            .get(&binding)
            .cloned()
            .unwrap_or_else(UiHostObservationPartition::empty);
        let previous = UiHostObservationRetentionBasis::from_partition(&partition);
        let mut basis_changes = UiObservationBasisChanges::default();
        let validated = retain_batch_reports(
            &mut partition,
            batch.reports(),
            UiHostObservationRetentionMutation {
                relation,
                frame,
                host_coalescing: host_coalescing_admission(&batch),
                basis_changes: &mut basis_changes,
            },
        );
        let family = capacity_family(batch.disposition(), batch.reports());
        let admitted = self.admit_retention_capacity(&partition, previous, family)?;
        partition.remember_batch(UiHostObservationBatchFingerprint {
            sequences: core.sequences(),
            integrity: batch.integrity(),
        });
        self.commit_observation_basis(frame, observation_basis, basis_changes);
        self.commit_partition(binding, partition, admitted);
        self.last_sequence = Some(core.sequences().last());
        Ok(UiHostObservationReportOutcome::Validated(
            UiValidatedHostObservationBatch::new(core, relation, batch.disposition(), validated),
        ))
    }

    pub(super) fn duplicate_covered_batch(
        &self,
        batch: &super::sequence_coverage::UiSequenceCoveredObservationBatch,
    ) -> Option<UiHostObservationReportOutcome> {
        let core = batch.core();
        let retained = self
            .partitions
            .get(&core.binding())
            .is_some_and(|partition| partition.duplicate(core, batch.integrity()));
        let quarantined = self.quarantine_fingerprints.iter().any(|candidate| {
            candidate.sequences == core.sequences() && candidate.integrity == batch.integrity()
        });
        (retained || quarantined).then(|| {
            UiHostObservationReportOutcome::Duplicate(UiDuplicateHostObservationBatch::new(
                core.sequences(),
                batch.integrity(),
            ))
        })
    }

    fn admit_retention_capacity(
        &self,
        partition: &UiHostObservationPartition,
        previous: UiHostObservationRetentionBasis,
        family: UiHostObservationFamily,
    ) -> Result<UiAdmittedHostObservationRetention, UiHostObservationReportDenial> {
        enforce_local_capacity(partition, self.capacity, family)?;
        let global_reports = self.global_reports - previous.report_count + partition.reports.len();
        let global_bytes = self.global_bytes - previous.byte_count + partition.byte_count;
        if global_reports > self.capacity.global_reports()
            || global_bytes > self.capacity.global_bytes()
        {
            return Err(UiHostObservationReportDenial::GlobalCapacityExceeded(
                family,
            ));
        }
        Ok(UiAdmittedHostObservationRetention {
            global_reports,
            global_bytes,
        })
    }

    fn commit_partition(
        &mut self,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        partition: UiHostObservationPartition,
        admitted: UiAdmittedHostObservationRetention,
    ) {
        self.global_reports = admitted.global_reports;
        self.global_bytes = admitted.global_bytes;
        self.partitions.insert(binding, partition);
    }

    fn commit_observation_basis(
        &mut self,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        lease: crate::mounting::UiMountedObservationBasisLease,
        changes: UiObservationBasisChanges,
    ) {
        for removed in changes.removed {
            let remove_entry = {
                let retained = self
                    .observation_bases
                    .get_mut(&removed)
                    .expect("every retained report frame owns observation basis evidence");
                retained.retained_reports = retained
                    .retained_reports
                    .checked_sub(1)
                    .expect("observation basis report accounting includes the removed report");
                retained.retained_reports == 0
            };
            if remove_entry {
                self.observation_bases.remove(&removed);
            }
        }
        if changes.added == 0 {
            return;
        }
        match self.observation_bases.entry(frame) {
            std::collections::btree_map::Entry::Occupied(mut occupied) => {
                occupied.get_mut().retained_reports = occupied
                    .get()
                    .retained_reports
                    .checked_add(changes.added)
                    .expect("retained observation report count fits usize");
            }
            std::collections::btree_map::Entry::Vacant(vacant) => {
                vacant.insert(UiRetainedObservationBasis {
                    retained_reports: changes.added,
                    lease,
                });
            }
        }
    }
}

#[derive(Clone, Copy)]
struct UiHostObservationRetentionBasis {
    report_count: usize,
    byte_count: usize,
}

#[derive(Clone, Copy)]
struct UiAdmittedHostObservationRetention {
    global_reports: usize,
    global_bytes: usize,
}

#[derive(Clone, Copy)]
struct UiHostCoalescingAdmission {
    survivor: UiHostObservationSequence,
    replaced: UiHostObservationSequenceRange,
}

#[derive(Default)]
struct UiObservationBasisChanges {
    added: usize,
    removed: Vec<worth_ui_host_contract::UiMountedFrameIdentity>,
}

impl UiObservationBasisChanges {
    fn remove(
        &mut self,
        removed: worth_ui_host_contract::UiMountedFrameIdentity,
        candidate: worth_ui_host_contract::UiMountedFrameIdentity,
    ) {
        if removed == candidate && self.added > 0 {
            self.added -= 1;
        } else {
            self.removed.push(removed);
        }
    }
}

struct UiHostObservationRetentionMutation<'a> {
    relation: UiHostObservationFrameRelation,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    host_coalescing: Option<UiHostCoalescingAdmission>,
    basis_changes: &'a mut UiObservationBasisChanges,
}

impl UiHostObservationRetentionBasis {
    fn from_partition(partition: &UiHostObservationPartition) -> Self {
        Self {
            report_count: partition.reports.len(),
            byte_count: partition.byte_count,
        }
    }
}

fn capacity_family(
    disposition: UiHostObservationBatchDisposition,
    reports: &[UiHostObservationReport],
) -> UiHostObservationFamily {
    match disposition {
        UiHostObservationBatchDisposition::Complete => reports
            .last()
            .expect("exact complete coverage contains a report")
            .family(),
        UiHostObservationBatchDisposition::Coalesced { family, .. }
        | UiHostObservationBatchDisposition::Overflow { family, .. } => family,
    }
}

fn host_coalescing_admission(
    batch: &super::sequence_coverage::UiSequenceCoveredObservationBatch,
) -> Option<UiHostCoalescingAdmission> {
    match (batch.disposition(), batch.host_survivor()) {
        (UiHostObservationBatchDisposition::Coalesced { replaced, .. }, Some(survivor)) => {
            Some(UiHostCoalescingAdmission { survivor, replaced })
        }
        _ => None,
    }
}

fn retain_batch_reports(
    partition: &mut UiHostObservationPartition,
    reports: &[UiHostObservationReport],
    mut mutation: UiHostObservationRetentionMutation<'_>,
) -> Vec<UiValidatedHostObservationReport> {
    reports
        .iter()
        .cloned()
        .map(|report| {
            let replaced = mutation
                .host_coalescing
                .filter(|admission| admission.survivor == report.sequence())
                .map(|admission| admission.replaced);
            retain_report(partition, report, replaced, &mut mutation)
        })
        .collect()
}

fn retain_report(
    partition: &mut UiHostObservationPartition,
    report: UiHostObservationReport,
    host_replaced: Option<UiHostObservationSequenceRange>,
    mutation: &mut UiHostObservationRetentionMutation<'_>,
) -> UiValidatedHostObservationReport {
    let coalescing_identity = report.coalescing_identity();
    let replaced = partition.reports.back().and_then(|previous| {
        (coalescing_identity.is_some()
            && previous.coalescing_identity == coalescing_identity
            && previous.relation == mutation.relation)
            .then(|| previous.replaced_range())
    });
    let disposition = if let Some(replaced) = replaced {
        let previous = partition
            .reports
            .pop_back()
            .expect("coalescing candidate is retained");
        partition.byte_count -= previous.encoded_len;
        mutation
            .basis_changes
            .remove(previous.frame, mutation.frame);
        let first = host_replaced
            .map(|range| range.first().min(replaced.first()))
            .unwrap_or(replaced.first());
        let last = host_replaced
            .map(|range| range.last().max(previous.report.report().sequence()))
            .unwrap_or(previous.report.report().sequence());
        UiHostObservationDisposition::Coalesced {
            replaced: UiHostObservationSequenceRange::new(first, last),
        }
    } else {
        host_replaced
            .map(|replaced| UiHostObservationDisposition::Coalesced { replaced })
            .unwrap_or(UiHostObservationDisposition::Retained)
    };
    let validated = UiValidatedHostObservationReport::new(report, disposition);
    push_retained(partition, validated.clone(), coalescing_identity, mutation);
    validated
}

fn push_retained(
    partition: &mut UiHostObservationPartition,
    report: UiValidatedHostObservationReport,
    coalescing_identity: Option<UiHostObservationCoalescingIdentity>,
    mutation: &mut UiHostObservationRetentionMutation<'_>,
) {
    let encoded_len = report.report().encoded_len();
    partition.byte_count += encoded_len;
    partition
        .reports
        .push_back(UiRetainedHostObservationReport {
            report,
            relation: mutation.relation,
            frame: mutation.frame,
            coalescing_identity,
            encoded_len,
        });
    mutation.basis_changes.added = mutation
        .basis_changes
        .added
        .checked_add(1)
        .expect("batch report count fits usize");
}

fn enforce_local_capacity(
    partition: &UiHostObservationPartition,
    capacity: super::UiHostObservationCapacity,
    family: UiHostObservationFamily,
) -> Result<(), UiHostObservationReportDenial> {
    if partition.reports.len() > capacity.local_reports()
        || partition.byte_count > capacity.local_bytes()
    {
        return Err(UiHostObservationReportDenial::LocalCapacityExceeded(family));
    }
    Ok(())
}

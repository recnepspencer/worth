//! One request settles the wave; member outcomes travel to owner completion.
use super::super::execution_denial::{checkpoint_denial, lease_denial};
use super::super::{QueryRequestExecution, WorthQueryAdvancementDenial};
use super::prepared_request::{ComputedSlot, PreparedCompute};
use super::WorthQueryMemberExecution;
use worth_execution::{
    ChargedBytes, ExecutionMap, KeylessPartition, MapKernelFailure, MapOutcome, MapStop,
};
use worth_foundational::PartitionIdentity;

// A member outcome cannot stop the map; only its request can.
enum NoMemberFailure {}
impl ChargedBytes for NoMemberFailure {
    fn additional_charged_bytes(&self) -> u64 {
        match *self {}
    }
}

pub(in crate::domain_computation::primary_graph) struct PreparedMember {
    pub requests: Vec<PreparedCompute>,
}
impl ChargedBytes for PreparedMember {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { requests } = self;
        requests.additional_charged_bytes()
    }
}
pub(in crate::domain_computation::primary_graph) struct ComputedMember {
    slots: Vec<ComputedSlot>,
    // Release the payload before its owner-side request hold.
    custody: Option<super::super::request_execution::QueryMemoryReservation>,
}
impl ChargedBytes for ComputedMember {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { slots, custody: _ } = self;
        slots.additional_charged_bytes()
    }
}

/// Interruption already present at dispatch has width-independent settlement.
/// An interruption arriving during compute can be seen by an earlier running
/// member: only the canonical-prefix safety law is promised across widths.
pub(in crate::domain_computation::primary_graph) fn dispatch(
    execution: &QueryRequestExecution<'_>,
    members: Vec<PreparedMember>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    if members.iter().all(|member| member.requests.is_empty()) {
        return members
            .into_iter()
            .map(|_| {
                Ok(ComputedMember {
                    slots: Vec::new(),
                    custody: None,
                })
            })
            .collect();
    }
    let wave = execution.wave();
    let partitions = members
        .into_iter()
        .enumerate()
        .map(|(rank, member)| {
            let capacity = member.requests.iter().fold(
                (member.requests.len() * size_of::<ComputedSlot>()) as u64,
                |sum, request| sum.saturating_add(request.result_capacity()),
            );
            (
                PartitionIdentity::new(rank as u64 + 1),
                KeylessPartition {
                    value: member,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: capacity,
                },
            )
        })
        .collect();
    let map = match ExecutionMap::<_, u64>::from_keyless_partitions(partitions) {
        Ok(map) => map,
        Err(worth_execution::MapMemoryOverflow) => {
            return vec![Err(WorthQueryAdvancementDenial::Resource(
                super::super::WorthQueryManagedComputationResourceDenial::CapacityOverflow,
            ))]
        }
    };
    let request = match wave.request().execution_request() {
        Ok(request) => request,
        Err(denial) => return vec![Err(WorthQueryAdvancementDenial::Resource(denial))],
    };
    let outcome = request.in_scope(|lease| {
        let kernel = |input: PreparedMember,
                      context: &mut worth_execution::MapKernelContext<'_, '_>|
         -> Result<_, MapKernelFailure<NoMemberFailure>> {
            let member = wave.member();
            let mut slots = Vec::with_capacity(input.requests.len());
            for request in input.requests {
                // A prior request's untaken cause cannot reach a later
                // request, even when that request returns its own error.
                context.checkpoint(0)?;
                let context = std::cell::RefCell::new(&mut *context);
                let mut charge = |units| context.borrow_mut().checkpoint(units);
                let mut take = || context.borrow_mut().take_nested_computation_stop();
                let slot = request.run(WorthQueryMemberExecution::for_kernel(
                    &member,
                    &mut charge,
                    &mut take,
                ));
                slots.push(slot);
            }
            let result = ComputedMember {
                slots,
                custody: None,
            };
            Ok(result)
        };
        let outcome = match super::super::request_execution::certified_wave_seed_for_test() {
            Some(seed) => {
                let (outcome, order) = map.run_owned_perturbed_for_test(lease, seed, kernel);
                #[cfg(test)]
                tests::seeded_dispatch::record(order);
                #[cfg(not(test))]
                drop(order);
                outcome
            }
            None => map.run_owned(lease, kernel),
        };
        retain_outcome(execution, outcome)
    });
    match outcome {
        Ok(outcome) => outcome,
        Err(denial) => vec![Err(super::super::request_execution::advancement_denial(
            denial,
        ))],
    }
}

fn retain_outcome(
    execution: &QueryRequestExecution<'_>,
    outcome: MapOutcome<ComputedMember, NoMemberFailure>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    match outcome {
        MapOutcome::Complete { values, report: _ } => {
            retain_members(execution, values.into_iter().map(Ok).collect())
        }
        MapOutcome::Stopped {
            completed_prefix,
            boundary: _,
            reason,
            report: _,
        } => {
            let mut settled: Vec<_> = completed_prefix.into_iter().map(Ok).collect();
            let failed = match reason {
                MapStop::Failure { identity: _, cause: MapKernelFailure::Domain(impossible) } => match impossible {},
                MapStop::Failure { identity: _, cause: MapKernelFailure::Stop(stop) } => Err(from_checkpoint(stop)),
                MapStop::Failure { identity: _, cause: MapKernelFailure::Panic } => Err(WorthQueryAdvancementDenial::Panicked),
                MapStop::Failure { identity: _, cause: MapKernelFailure::ResultCapacityExceeded } => Err(
                    WorthQueryAdvancementDenial::Resource(super::super::WorthQueryManagedComputationResourceDenial::ResultCapacityExceeded)),
                MapStop::WorkExhausted { identity: _ } => Err(WorthQueryAdvancementDenial::Resource(
                    super::super::WorthQueryManagedComputationResourceDenial::WorkExhausted)),
                MapStop::Admission(denial) => Err(WorthQueryAdvancementDenial::Resource(lease_denial(denial))),
            };
            settled.push(failed);
            retain_members(execution, settled)
        }
    }
}
fn retain_members(
    execution: &QueryRequestExecution<'_>,
    members: Vec<Result<ComputedMember, WorthQueryAdvancementDenial>>,
) -> Vec<Result<ComputedMember, WorthQueryAdvancementDenial>> {
    let mut kept = Vec::with_capacity(members.len());
    for result in members {
        let result = result.and_then(|mut member| {
            member.custody = Some(
                execution
                    .reserve(member.slots.additional_charged_bytes())
                    .map_err(WorthQueryAdvancementDenial::Resource)?,
            );
            Ok(member)
        });
        let stopped = result.is_err();
        kept.push(result);
        if stopped {
            break;
        }
    }
    kept
}

impl ComputedMember {
    pub(in crate::domain_computation::primary_graph) fn finish<T>(
        self,
        complete: impl FnOnce(Vec<ComputedSlot>) -> T,
    ) -> T {
        let Self { slots, custody } = self;
        let result = complete(slots);
        drop(custody);
        result
    }
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn into_slots(self) -> Vec<ComputedSlot> {
        self.finish(|slots| slots)
    }
}

fn from_checkpoint(stop: worth_execution::MapKernelStop) -> WorthQueryAdvancementDenial {
    match checkpoint_denial(stop) {
        super::WorthQueryManagedComputationCheckpointDenial::Resource(denial) => {
            WorthQueryAdvancementDenial::Resource(denial)
        }
        super::WorthQueryManagedComputationCheckpointDenial::Interrupted(denial) => {
            WorthQueryAdvancementDenial::Interrupted(denial)
        }
        super::WorthQueryManagedComputationCheckpointDenial::NestedPatternStopped => {
            WorthQueryAdvancementDenial::NestedStopped
        }
    }
}

#[cfg(test)]
mod tests;

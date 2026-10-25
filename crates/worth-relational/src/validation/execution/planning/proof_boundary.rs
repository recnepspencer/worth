use std::collections::BTreeSet;

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::authority::commit::preparation::proofs::locality::{
    PreparationPartitionScope, PreparationReadSetApproximation,
};
use crate::validation::engine::{
    InvariantPlanScopeClass, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};

pub(crate) fn planned_proof_boundary_summary(
    planned: &PreparedInvariantExecution<'_>,
) -> InvariantProofBoundarySummary {
    let mut widened_causes = Vec::new();
    let mut touched_partitions = BTreeSet::new();
    let mut saw_touched_only = false;

    for packet in &planned.packets {
        collect_partition_scope_evidence(packet, &mut widened_causes, &mut touched_partitions);
        collect_read_set_evidence(packet, &mut widened_causes, &mut saw_touched_only);
    }

    InvariantProofBoundarySummary::new(
        proof_scope_class(&widened_causes, saw_touched_only),
        widened_causes,
        planned.packets.len(),
        touched_partitions.len(),
    )
}

fn collect_partition_scope_evidence(
    packet: &crate::authority::commit::preparation::InvariantWorkPacket<'_>,
    widened_causes: &mut Vec<InvariantScopeWideningCause>,
    touched_partitions: &mut BTreeSet<crate::identity::data::PartitionId>,
) {
    match &packet.locality.partition_scope {
        PreparationPartitionScope::AllObserved => {
            if !widened_causes.contains(&InvariantScopeWideningCause::AllObservedPartitionScope) {
                widened_causes.push(InvariantScopeWideningCause::AllObservedPartitionScope);
            }
        }
        PreparationPartitionScope::TouchedPartitions(partitions) => {
            touched_partitions.extend(partitions.iter().copied());
        }
    }
}

fn collect_read_set_evidence(
    packet: &crate::authority::commit::preparation::InvariantWorkPacket<'_>,
    widened_causes: &mut Vec<InvariantScopeWideningCause>,
    saw_touched_only: &mut bool,
) {
    match packet.locality.read_set_approximation {
        PreparationReadSetApproximation::FullObservedScan => {
            if !widened_causes.contains(&InvariantScopeWideningCause::FullObservedReadSet) {
                widened_causes.push(InvariantScopeWideningCause::FullObservedReadSet);
            }
        }
        PreparationReadSetApproximation::TouchedOnly => {
            *saw_touched_only = true;
        }
        PreparationReadSetApproximation::SharedCommittedRead => {}
    }
}

fn proof_scope_class(
    widened_causes: &[InvariantScopeWideningCause],
    saw_touched_only: bool,
) -> InvariantPlanScopeClass {
    if !widened_causes.is_empty() {
        InvariantPlanScopeClass::BroaderScope
    } else if saw_touched_only {
        InvariantPlanScopeClass::TouchedScope
    } else {
        InvariantPlanScopeClass::PartitionScope
    }
}

/// Completed stage keys retain the same actual partition slices as their plans.
/// Compose their evidence without summing overlapping partition counts.
pub(crate) fn composed_proof_boundary_summary<'a, E>(
    stages: impl IntoIterator<Item = &'a crate::validation::execution::InvariantStageOutput>,
    mut check: impl FnMut(u64, u64) -> Result<(), E>,
) -> Result<InvariantProofBoundarySummary, E> {
    let mut touched = BTreeSet::new();
    let mut saw_touched_only = false;
    let mut seen = [false; 2];
    let mut ordered = [None; 2];
    let mut cause_count = 0;
    let mut packet_count = 0;
    for stage in stages {
        packet_count += stage.proof_boundary.packet_count();
        saw_touched_only |=
            stage.proof_boundary.scope_class() == InvariantPlanScopeClass::TouchedScope;
        for &cause in stage.proof_boundary.widened_causes() {
            check(1, 0)?;
            let index = match cause {
                InvariantScopeWideningCause::AllObservedPartitionScope => 0,
                InvariantScopeWideningCause::FullObservedReadSet => 1,
            };
            if !seen[index] {
                seen[index] = true;
                ordered[cause_count] = Some(cause);
                cause_count += 1;
            }
        }
        for envelope in &stage.envelopes {
            for &partition in envelope.reduction_key.partition_scope.iter() {
                check(1, 0)?;
                if !touched.contains(&partition) {
                    // Same owned BTreeSet node payload/link convention as checked planning.
                    check(
                        0,
                        (std::mem::size_of::<crate::identity::data::PartitionId>()
                            + 4 * std::mem::size_of::<usize>()) as u64,
                    )?;
                    touched.insert(partition);
                }
            }
        }
    }
    check(
        0,
        (cause_count * std::mem::size_of::<InvariantScopeWideningCause>()) as u64,
    )?;
    let mut widened = Vec::with_capacity(cause_count);
    widened.extend(ordered.into_iter().flatten());
    Ok(InvariantProofBoundarySummary::new(
        proof_scope_class(&widened, saw_touched_only),
        widened,
        packet_count,
        touched.len(),
    ))
}

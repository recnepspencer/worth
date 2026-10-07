use super::super::*;
use super::materialization::ProjectedMaterializationBasis;

pub(super) struct StagingActionBasis {
    pub(super) actions: Vec<RecoveryStagingAction>,
    pub(super) allocated_targets: Vec<PhysicalRedoTargetIdentity>,
}

pub(super) fn derive(
    redo: &ImmutablePhysicalRedoPlan,
    materialization: &ProjectedMaterializationBasis,
    staging_generation: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<StagingActionBasis, ExecutionBasisDenial> {
    let count = redo
        .resolved_decisions()
        .filter(|decision| decision.kind() == PhysicalRedoDecisionKind::Apply)
        .count();
    let mut grouped = allowance.reserve(count)?;
    for (ordinal, decision) in redo.resolved_decisions().enumerate() {
        if decision.kind() != PhysicalRedoDecisionKind::Apply {
            continue;
        }
        let target = decision.target();
        let step = step(decision)?;
        grouped.push((target.identity(), ordinal, target, step));
    }
    grouped.sort_unstable_by_key(|row| (row.0, row.1));
    let group_count = grouped
        .iter()
        .enumerate()
        .filter(|(index, row)| *index == 0 || grouped[*index - 1].0 != row.0)
        .count();
    let mut allocated_targets = allowance.reserve(group_count)?;
    let mut actions = allowance.reserve(group_count)?;
    let mut start = 0;
    while start < grouped.len() {
        let identity = grouped[start].0;
        let source = grouped[start].2;
        let end = start + grouped[start..].partition_point(|row| row.0 == identity);
        if grouped[start..end].iter().any(|row| row.2 != source)
            || materialization
                .frames
                .binary_search_by_key(&identity, |row| row.0)
                .is_err()
        {
            return Err(ExecutionBasisDenial::Invalid);
        }
        let mut steps = allowance.reserve(end - start)?;
        steps.extend(grouped[start..end].iter().map(|row| row.3.clone()));
        actions.push(RecoveryStagingAction {
            ordinal: actions.len() as u64,
            steps: allowance.into_box(steps)?,
            source: source.clone(),
            destination_generation: staging_generation,
        });
        allocated_targets.push(identity);
        start = end;
    }
    let scratch = PlanningResidentAllowance::vector_bytes(&grouped)?;
    drop(grouped);
    allowance.release(scratch);
    Ok(StagingActionBasis {
        actions,
        allocated_targets,
    })
}

fn step(
    decision: worth_store_recovery_physics::PhysicalRedoDecisionView<'_>,
) -> Result<RecoveryStagingRedoStep, ExecutionBasisDenial> {
    Ok(RecoveryStagingRedoStep {
        operation: decision.operation(),
        record_index: decision.record_index(),
        target_index: decision.target_index(),
        record_lsn: decision.record().lsn().get(),
        prior: match decision.prior() {
            PhysicalRedoDecisionPrior::Page(prior) => prior,
            PhysicalRedoDecisionPrior::OperationFate(_) => {
                return Err(ExecutionBasisDenial::Invalid)
            }
        },
    })
}

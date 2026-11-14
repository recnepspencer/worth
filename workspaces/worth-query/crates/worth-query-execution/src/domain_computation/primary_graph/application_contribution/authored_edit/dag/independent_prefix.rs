//! Narrow only a stable independent prefix of complete actual scalar programs.
use super::*;
pub(super) fn refine<S: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &[CapturedAuthoredEditStage<S>],
    nodes: &[Node<'_>],
    edges: &mut Vec<(usize, usize, Option<DeclaredEditDependency>)>,
    context: &AuthoredEditStageContext<'_, '_, '_, S>,
    unseen: &UnseenAuthoredEnvelope<'_>,
) -> Result<(), DeclaredEditOrderRefusal> {
    let bytes = stages.len().checked_mul(size_of::<Option<super::super::planning::Footprint>>() + size_of::<bool>())
        .and_then(|bytes| u64::try_from(bytes).ok()).ok_or_else(|| DeclaredEditOrderRefusal::Execution(
            crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::AllocationDenied, "authored prefix plan")
        ))?;
    let _memory = execution.allocation_policy().reserve_memory(bytes).map_err(|cause| DeclaredEditOrderRefusal::Execution(
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied("authored prefix plan", cause)
    ))?;

    for (ordinal, stage) in stages.iter().enumerate() {
        if !edges
            .iter()
            .any(|(writer, reader, _)| *writer >= stages.len() && *reader == ordinal)
        {
            stage
                .executor
                .plan(context, unseen)
                .map_err(|cause| DeclaredEditOrderRefusal::Planning(cause))?;
        }
    }
    let footprints = stages
        .iter()
        .map(|stage| stage.executor.footprint())
        .collect::<Vec<_>>();
    let mut eligible = footprints.iter().map(Option::is_some).collect::<Vec<_>>();
    loop {
        let mut changed = false;
        for reader in 0..stages.len() {
            if !eligible[reader] {
                continue;
            }
            for writer in 0..nodes.len() {
                if writer == reader {
                    continue;
                }
                let intersects = if writer < stages.len() && eligible[writer] {
                    footprints[reader]
                        .as_ref()
                        .expect("eligible read")
                        .intersects(
                            footprints[writer].as_ref().expect("eligible write"),
                            execution,
                        )
                        .map_err(DeclaredEditOrderRefusal::Execution)?
                } else {
                    overlap(&nodes[reader], &nodes[writer], &stages[0].schema).is_some()
                };
                if intersects {
                    eligible[reader] = false;
                    stages[reader].executor.discard_plan();
                    changed = true;
                    break;
                }
            }
        }
        if !changed {
            break;
        }
    }
    edges.retain(|(writer, reader, _)| {
        !(*writer < stages.len()
            && *reader < stages.len()
            && eligible[*writer]
            && eligible[*reader])
    });
    Ok(())
}

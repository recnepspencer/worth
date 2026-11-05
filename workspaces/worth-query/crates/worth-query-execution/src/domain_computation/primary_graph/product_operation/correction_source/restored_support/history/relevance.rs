//! Outside-gate historical narrowing preserves every ordinary-selected occurrence.
use super::super::super::{
    WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedProductOperation,
};
use super::super::{admit_work, ordered_visit, WorthQueryCorrectionRestoredSupportDenial as Stop};
use super::{Occurrences, SelectedHistory};
use crate::domain_computation::primary_graph::{
    access_strategy::realized_footprint::ManagedHistoryRelevanceDenial,
    application_attempt::WorthQueryApplicationObservedFact as Fact,
    application_checkpoint::facts::FactPayload,
};
use worth_execution::ExecutionRequest;
use worth_query_installation::facade::ApplicationSchema;

pub(in crate::domain_computation::primary_graph::product_operation::correction_source::restored_support) fn narrow<
    Schema: ApplicationSchema,
>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    execution: ExecutionRequest<'_, '_>,
    facts: &FactPayload,
    selected: SelectedHistory,
) -> Result<Occurrences, Denial> {
    let SelectedHistory {
        mut ordinary,
        managed,
    } = selected;
    admit_work(execution, managed.len() as u64).map_err(Denial::RestoredSupport)?;
    for (commit, occurrence) in &managed {
        ordered_visit(execution, ordinary.len()).map_err(Denial::RestoredSupport)?;
        if ordinary.contains_key(commit) {
            continue;
        }
        admit_work(execution, facts.len() as u64).map_err(Denial::RestoredSupport)?;
        for fact in facts.iter() {
            execution
                .consult()
                .map_err(|cause| Denial::RestoredSupport(Stop::Request(cause)))?;
            let Fact::ManagedIndexSource { observation } = fact else {
                continue;
            };
            let impact = target
                .application()
                .primary_provider()
                .graph
                .with_runtime_mut(&execution, |runtime| {
                    observation.history_impact(runtime, &execution, occurrence)
                })
                .map_err(denial)?;
            // This call may acquire the last strong opaque share. No runtime,
            // attempt or World gate spans acquisition, provider work or disposal.
            if observation
                .history_relevant(&execution, impact)
                .map_err(denial)?
            {
                ordered_visit(execution, ordinary.len()).map_err(Denial::RestoredSupport)?;
                ordinary
                    .insert_cloned_native_for_request(commit, occurrence, execution)
                    .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
                break;
            }
        }
    }
    Ok(ordinary)
}
fn denial(cause: ManagedHistoryRelevanceDenial) -> Denial {
    Denial::RestoredSupport(match cause {
        ManagedHistoryRelevanceDenial::Native(cause) => Stop::Native(cause),
        ManagedHistoryRelevanceDenial::Provider(cause) => Stop::ManagedProvider(cause),
    })
}

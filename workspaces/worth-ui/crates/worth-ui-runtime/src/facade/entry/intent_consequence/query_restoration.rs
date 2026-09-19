use crate::runtime::intent_execution::UiIntentConsequenceHandoff;

pub(super) fn restore_query_from_batch(
    handoff: &mut UiIntentConsequenceHandoff,
    batch: crate::runtime::observation::UiIntentConsequenceObservationBatch,
) {
    let (_, query, projection) = batch.into_parts();
    if let Some(query) = query {
        handoff.restore_query_consequence(query);
    }
    if let Some(projection) = projection {
        handoff.restore_query_projection(projection);
    }
}

pub(super) fn restore_query_from_facts(
    handoff: &mut UiIntentConsequenceHandoff,
    facts: Box<[crate::fact_contract::UiProducedFact]>,
) {
    handoff.restore_query_from_facts(facts);
}

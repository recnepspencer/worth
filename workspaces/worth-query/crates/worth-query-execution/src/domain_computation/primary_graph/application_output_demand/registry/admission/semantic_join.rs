//! Select the newest semantic source accepted by demand admission.
use super::*;

pub(in super::super) fn accepts_semantic_join(record: &DemandRecord) -> bool {
    match &record.state {
        DemandState::Failed(_) => false,
        DemandState::Output(output) => !matches!(
            output.advancement(),
            super::super::WorthQueryOutputAdvancement::Stopped { .. }
        ),
        _ => true,
    }
}

pub(in super::super) fn newest_semantic_key(
    state: &super::super::DemandRegistryState,
    requested: &WorthQueryOutputDemandKey,

    accepts: impl Fn(&DemandRecord) -> bool,
) -> Result<Option<WorthQueryOutputDemandKey>, WorthQueryOutputDemandDenial> {
    let mut newest = None;
    for (key, record) in super::super::refreshed_rejoin::occurrence_rows(&state.records, requested)
    {
        if key.same_semantic_source(requested)
            && accepts(record)
            && newest
                .as_ref()
                .is_none_or(|prior: &WorthQueryOutputDemandKey| {
                    key.source.observation_generation() > prior.source.observation_generation()
                })
        {
            newest = Some(key.clone());
        }
    }
    Ok(newest)
}

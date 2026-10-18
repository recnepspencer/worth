//! A fresh caller can retain a successor source without replacing the original Ready registration.
use super::super::{
    DemandRecord, DemandState, SourceEpoch, WorthQueryOutputAdvancement,
    WorthQueryOutputCheckpoint, WorthQueryOutputDemandKey,
};
use std::{any::Any, sync::Arc};

/// This permits source custody placement, never currentness or publication.
/// Every pin is already installed by this registry on the actual native Ready.
pub(super) fn validate_original_ready(
    record: &DemandRecord,
    key: &WorthQueryOutputDemandKey,
    successor: &SourceEpoch,
    producer: &Arc<dyn Any + Send + Sync>,
) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial> {
    let refuse = |subject| {
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
        subject,
    )
    };
    if key.source.replacement_order(successor) != Some(std::cmp::Ordering::Less) {
        return Err(refuse(
            "required output successor is not ordered after its Ready source",
        ));
    }
    if record.settlements.is_empty()
        || (record.prerequisites.is_empty() && record.checkpoint_prerequisites.is_none())
    {
        return Err(refuse(
            "required output successor has no posted consumer custody",
        ));
    }
    if record.work_membership.is_none() {
        return Err(refuse(
            "required output successor has no retained work membership",
        ));
    }
    let Some(original) = record.readmission_source.as_ref() else {
        return Err(refuse(
            "required output successor has no original readmission registration",
        ));
    };
    let installed = original
        .producer
        .get()
        .is_some_and(|entry| Arc::ptr_eq(entry, producer));
    if !installed {
        return Err(refuse(
            "required output successor differs from its installed producer backing",
        ));
    }
    if !matches!(&record.state, DemandState::Output(output)
        if matches!(output.advancement(), WorthQueryOutputAdvancement::Idle)
            && matches!(&output.checkpoint, Some(WorthQueryOutputCheckpoint::Ready(completion))
                if completion.producer_mode().is_some()))
    {
        return Err(refuse("required output successor has no idle native Ready"));
    }
    Ok(())
}

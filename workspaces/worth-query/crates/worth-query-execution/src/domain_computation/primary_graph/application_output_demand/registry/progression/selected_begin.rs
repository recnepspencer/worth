//! Meter the required wave's first registry transition before it changes a row.

use super::selected_finish::PreparedSelectedSchedulingFinish;
use super::*;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

pub(super) fn begin_admitted<'a>(
    registry: &'a WorthQueryOutputDemandRegistry,
    interest: &'a WorthQueryOutputDemandInterest,
) -> Result<
    (
        WorthQueryOutputDemandAdvanceAdmission,
        Option<PreparedSelectedSchedulingFinish<'a>>,
    ),
    WorthQueryOutputDemandDenial,
> {
    // This envelope covers the ordered-lookup refusal. Selected claim
    // overflow uses an allocation-free empty subject in the shared core.

    if !std::sync::Arc::ptr_eq(&registry.state, &interest.owner.state) {
        return Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::ForeignDemand,
            "",
        ));
    }
    let mut state = registry
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut record = state
        .records
        .get_mut(&interest.key)
        .expect("live required Interest retains its owner record");
    // Shared core writes the state and returned admission, while Option::take
    // initializes both a vacant slot and its moved result. This conservative
    // envelope covers each possible branch before the first mutation.

    let member = if matches!(record.state, DemandState::Admitted) {
        Some(
            record
                .work_membership
                .as_ref()
                .cloned()
                .ok_or_else(empty_begin_work_denial)?,
        )
    } else {
        None
    };
    let result = begin_record(&mut record, "");
    let finish = if matches!(result, WorthQueryOutputDemandAdvanceAdmission::Schedule(_)) {
        Some(PreparedSelectedSchedulingFinish::new(
            registry,
            interest,
            member.expect("admitted required row retains its member"),
        ))
    } else {
        None
    };
    Ok((result, finish))
}

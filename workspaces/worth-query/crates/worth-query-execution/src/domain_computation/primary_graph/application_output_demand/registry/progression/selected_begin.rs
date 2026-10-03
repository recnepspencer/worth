//! Meter the required wave's first registry transition before it changes a row.

use super::selected_finish::PreparedSelectedSchedulingFinish;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

pub(super) fn begin_admitted<'a>(
    registry: &'a WorthQueryOutputDemandRegistry,
    interest: &'a WorthQueryOutputDemandInterest,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    (
        WorthQueryOutputDemandAdvanceAdmission,
        Option<PreparedSelectedSchedulingFinish<'a>>,
    ),
    WorthQueryOutputDemandDenial,
> {
    // This envelope covers the ordered-lookup refusal. Selected claim
    // overflow uses an allocation-free empty subject in the shared core.
    const TERMINAL: usize = 55;
    let backing = TERMINAL
        .checked_add(std::mem::size_of::<String>())
        .ok_or_else(empty_begin_work_denial)?;
    admission
        .charge_external_work(u64::try_from(backing).map_err(|_| empty_begin_work_denial())?)
        .map_err(|_| empty_begin_work_denial())?;
    admission
        .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_begin_work_denial())?)
        .map_err(empty_begin_preflight_denial)?;
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
    state.charge_record_lookup(&interest.key, admission)?;
    state.charge_maximum_record_lookup(&interest.key, admission)?;
    state.charge_maximum_record_lookup(&interest.key, admission)?;
    let record = state
        .records
        .get_mut(&interest.key)
        .expect("live required Interest retains its owner record");
    // Shared core writes the state and returned admission, while Option::take
    // initializes both a vacant slot and its moved result. This conservative
    // envelope covers each possible branch before the first mutation.
    let fixed = std::mem::size_of::<Option<WorthQueryPerformedOutputDemandSource>>()
        .checked_mul(2)
        .and_then(|bytes| {
            std::mem::size_of::<Option<WorthQueryOutputCheckpoint>>()
                .checked_mul(2)
                .and_then(|checkpoint| bytes.checked_add(checkpoint))
        })
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<ReadyCompletion>()))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Option<[u8; 32]>>()))
        .and_then(|bytes| {
            std::mem::size_of::<WorthQueryOutputDemandAdvanceAdmission>()
                .checked_mul(2)
                .and_then(|result| bytes.checked_add(result))
        })
        .and_then(|bytes| {
            std::mem::size_of::<DemandState>()
                .checked_mul(2)
                .and_then(|state| bytes.checked_add(state))
        })
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<WorthQueryOutputDemandDenial>()))
        .and_then(|bytes| bytes.checked_add(8))
        .ok_or_else(empty_begin_work_denial)?;
    admission
        .charge_external_work(u64::try_from(fixed).map_err(|_| empty_begin_work_denial())?)
        .map_err(|_| empty_begin_work_denial())?;
    let finish_work = std::mem::size_of::<DemandState>()
        .checked_mul(2)
        .and_then(|work| {
            std::mem::size_of::<Option<WorthQueryPerformedOutputDemandSource>>()
                .checked_mul(2)
                .and_then(|source| work.checked_add(source))
        })
        .and_then(|work| work.checked_add(std::mem::size_of::<WorthQueryOutputDemandDenial>()))
        .and_then(|work| {
            work.checked_add(std::mem::size_of::<
                std::sync::Arc<required_work::RequiredWorkMembership>,
            >())
        })
        .and_then(|work| {
            std::mem::size_of::<Vec<PerformedOutputObligation>>()
                .checked_mul(2)
                .and_then(|obligations| work.checked_add(obligations))
        })
        .and_then(|work| work.checked_add(32))
        .ok_or_else(empty_begin_work_denial)?;
    admission
        .charge_external_work(u64::try_from(finish_work).map_err(|_| empty_begin_work_denial())?)
        .map_err(|_| empty_begin_work_denial())?;
    let existing_denial = match &record.state {
        DemandState::Failed(denial) => Some(denial),
        DemandState::Output(output) => match &output.advancement {
            WorthQueryOutputAdvancement::Stopped { denial, .. } => Some(denial),
            _ => None,
        },
        _ => None,
    };
    if let Some(denial) = existing_denial {
        let copy = denial
            .subject()
            .len()
            .checked_add(std::mem::size_of::<WorthQueryOutputDemandDenial>())
            .ok_or_else(empty_begin_work_denial)?;
        admission
            .charge_external_work(u64::try_from(copy).map_err(|_| empty_begin_work_denial())?)
            .map_err(|_| empty_begin_work_denial())?;
        let backing = denial
            .subject()
            .len()
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(empty_begin_work_denial)?;
        admission
            .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_begin_work_denial())?)
            .map_err(empty_begin_preflight_denial)?;
    }
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
    let result = begin_record(record, "");
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

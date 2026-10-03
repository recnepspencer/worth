//! Costs reached only by required Fresh source admission.

use worth_relational::facade::mvcc::CompanionPreflightStop;

use crate::domain_computation::primary_graph::{
    application_output_demand::WorthQueryOutputDemandKey,
    application_query::{WorthQueryObservedEpochStop, WorthQueryObservedSourceEpoch},
    output_lineage::invalidation::InvalidationEditAdmission,
};

use super::super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(super) fn preclaim_family_subject(
    family: &str,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(|_| empty_work())?;
    let backing = family
        .len()
        .checked_add(std::mem::size_of::<String>())
        .ok_or_else(empty_capacity)?;
    admission
        .charge_external_work(u64::try_from(family.len()).map_err(|_| empty_work())?)
        .map_err(|_| empty_work())?;
    admission
        .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_capacity())?)
        .map_err(empty_stop)
}

pub(super) fn preclaim_temporary_key(
    producer: &str,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(2)
        .map_err(|_| empty_work())?;
    let initialized = producer
        .len()
        .checked_add(std::mem::size_of::<WorthQueryObservedSourceEpoch>())
        .and_then(|work| work.checked_add(std::mem::size_of::<WorthQueryOutputDemandKey>()))
        .and_then(|work| work.checked_add(2))
        .ok_or_else(empty_work)?;
    admission
        .charge_external_work(u64::try_from(initialized).map_err(|_| empty_work())?)
        .map_err(|_| empty_work())?;
    // The temporary key and source epoch remain live while registry admission
    // reserves its independently retained key. Both physical owners count.
    let backing = producer
        .len()
        .checked_add(std::mem::size_of::<WorthQueryOutputDemandKey>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<WorthQueryObservedSourceEpoch>()))
        .ok_or_else(empty_capacity)?;
    admission
        .admit_read_scratch(u64::try_from(backing).map_err(|_| empty_capacity())?)
        .map_err(empty_stop)
}

pub(super) fn epoch_denial(
    stop: WorthQueryObservedEpochStop<CompanionPreflightStop>,
) -> WorthQueryOutputDemandDenial {
    match stop {
        WorthQueryObservedEpochStop::AccountingOverflow => empty_work(),
        WorthQueryObservedEpochStop::Admission(stop) => empty_stop(stop),
    }
}

fn empty_work() -> WorthQueryOutputDemandDenial {
    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_capacity() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}

fn empty_stop(stop: CompanionPreflightStop) -> WorthQueryOutputDemandDenial {
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkCounterOverflow => empty_work(),
        _ => empty_capacity(),
    }
}

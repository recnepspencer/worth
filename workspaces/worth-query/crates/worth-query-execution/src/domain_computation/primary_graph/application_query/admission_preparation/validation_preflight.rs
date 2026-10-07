//! Resource admission for fresh installed Query validation on retained reads.

use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::{
    InvalidationEditAdmission, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryApplicationQueryControls,
};

pub(super) fn admit_source_readmission_validation<
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
>(
    query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    access: &WorthQueryApplicationQueryAccessContext<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Scope,
    >,
    controls: &WorthQueryApplicationQueryControls<'_, Schema>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    let name = query.name();
    let principal_binding = access.principal().binding();
    let scope_name = access.scope().entity_name();
    let subject_bytes = name
        .len()
        .max(principal_binding.len())
        .max(scope_name.len())
        .max(controls.lane().as_str().len());
    // The source selector owner has already funded one query-name denial.
    // Installation owns its framed seal and HMAC cost. Its failure can hold
    // the installed denial and translated Query denial together, so this
    // phase funds the other possible subject.
    let authority_work = query.validation_work_bound().ok_or_else(work_denial)?;
    let comparisons = 22usize
        .checked_add(32)
        .and_then(|n| n.checked_add(scope_name.len().max(query.scope_entity().len())))
        .and_then(|n| n.checked_add(subject_bytes))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or_else(work_denial)?;
    let comparisons = authority_work
        .checked_add(comparisons)
        .ok_or_else(work_denial)?;
    admission
        .charge_external_work(comparisons)
        .map_err(|_| work_denial())?;
    let subject_bytes = u64::try_from(subject_bytes).map_err(|_| work_denial())?;
    admission
        .admit_read_scratch(subject_bytes)
        .map_err(resource_denial)?;
    Ok(())
}

fn work_denial() -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(
        WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
        String::new(),
    )
}

fn resource_denial(stop: CompanionPreflightStop) -> WorthQueryApplicationQueryAdmissionDenial {
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::WorkLimitExceeded,
    };
    WorthQueryApplicationQueryAdmissionDenial::new(kind, String::new())
}

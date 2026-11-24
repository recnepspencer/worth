use std::convert::Infallible;
use std::mem::size_of;

use super::RelationalAuthorizationObservationDenial;

/// Borrowed preparation authority. The observer never creates an allowance.
pub trait RelationalAuthorizationObservationAdmission {
    type Stop;
    fn prepare(&mut self, work: u64, preparation_bytes: u64) -> Result<(), Self::Stop>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum RelationalAuthorizationBudgetedObservationStop<Stop> {
    Execution(worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause),
    Native(RelationalAuthorizationObservationDenial),
    Admission(Stop),
    AccountingOverflow,
    ExactBasisRequired,
}

pub type ObservationResult<T, A> = Result<
    T,
    RelationalAuthorizationBudgetedObservationStop<
        <A as RelationalAuthorizationObservationAdmission>::Stop,
    >,
>;

pub fn prepare<A: RelationalAuthorizationObservationAdmission>(
    admission: &mut A,
    work: u64,
    bytes: u64,
) -> ObservationResult<(), A> {
    admission
        .prepare(work, bytes)
        .map_err(RelationalAuthorizationBudgetedObservationStop::Admission)
}

pub(super) fn array<A: RelationalAuthorizationObservationAdmission, T>(
    admission: &mut A,
    entries: usize,
) -> ObservationResult<(), A> {
    let bytes = entries
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
    prepare(admission, entries as u64, bytes)
}

pub(super) fn ordered_insert<A: RelationalAuthorizationObservationAdmission, K>(
    admission: &mut A,
    entries: usize,
    comparison_width: usize,
    owned_key_bytes: usize,
) -> ObservationResult<(), A> {
    // std BTree nodes hold at most eleven keys. Reserve every possible split
    // along the path, including a new root, before editing the temporary set.
    let levels = if entries == 0 {
        0
    } else {
        entries.ilog2() as usize + 1
    };
    let comparisons = levels
        .checked_mul(11)
        .and_then(|visits| visits.checked_mul(comparison_width))
        .and_then(|visits| visits.checked_add(1))
        .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
    let node_bytes = size_of::<K>()
        .checked_mul(11)
        .and_then(|bytes| bytes.checked_add(size_of::<usize>() * 14))
        .and_then(|bytes| bytes.checked_mul(levels + 2))
        .and_then(|bytes| bytes.checked_add(owned_key_bytes))
        .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
    prepare(admission, comparisons as u64, node_bytes as u64)
}

pub(super) fn ordered_lookup<A: RelationalAuthorizationObservationAdmission>(
    admission: &mut A,
    entries: usize,
    comparison_width: usize,
) -> ObservationResult<(), A> {
    let levels = if entries == 0 {
        0
    } else {
        entries.ilog2() as usize + 1
    };
    let work = levels
        .checked_mul(11)
        .and_then(|work| work.checked_mul(comparison_width))
        .and_then(|work| work.checked_add(1))
        .ok_or(RelationalAuthorizationBudgetedObservationStop::AccountingOverflow)?;
    prepare(admission, work as u64, 0)
}

pub struct UnrestrictedObservation;
impl RelationalAuthorizationObservationAdmission for UnrestrictedObservation {
    type Stop = Infallible;
    fn prepare(&mut self, _: u64, _: u64) -> Result<(), Self::Stop> {
        Ok(())
    }
}

pub fn legacy_stop(
    stop: RelationalAuthorizationBudgetedObservationStop<Infallible>,
) -> RelationalAuthorizationObservationDenial {
    match stop {
        RelationalAuthorizationBudgetedObservationStop::Execution(cause) => {
            RelationalAuthorizationObservationDenial::Execution(cause)
        }
        RelationalAuthorizationBudgetedObservationStop::Native(denial) => denial,
        RelationalAuthorizationBudgetedObservationStop::Admission(impossible) => {
            match impossible {}
        }
        RelationalAuthorizationBudgetedObservationStop::AccountingOverflow => {
            RelationalAuthorizationObservationDenial::PreparationAccountingOverflow
        }
        RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired => {
            RelationalAuthorizationObservationDenial::SnapshotUnavailable
        }
    }
}

pub struct RequestObservation<'request, 'execution, 'authority>(
    pub &'request worth_execution::ExecutionRequest<'execution, 'authority>,
);
impl RelationalAuthorizationObservationAdmission for RequestObservation<'_, '_, '_> {
    type Stop = worth_relational_contracts::internal_port::execution::denial_cause::RelationalExecutionDenialCause;
    fn prepare(&mut self, _: u64, _: u64) -> Result<(), Self::Stop> {
        self.0.consult().map_err(Into::into)
    }
}

/// Preserve native execution stops while rejecting a reconstructed basis separately.
pub(super) fn borrowed_denial<S>(
    denial: crate::branch_port::visibility::materialization::read_records::projection::borrowed_records::RelationalBorrowedRecordReadDenial,
) -> RelationalAuthorizationBudgetedObservationStop<S> {
    match denial {
        crate::branch_port::visibility::materialization::read_records::projection::borrowed_records::RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
            RelationalAuthorizationBudgetedObservationStop::ExactBasisRequired
        }
        crate::branch_port::visibility::materialization::read_records::projection::borrowed_records::RelationalBorrowedRecordReadDenial::Execution(cause) => {
            RelationalAuthorizationBudgetedObservationStop::Execution(cause)
        }
    }
}

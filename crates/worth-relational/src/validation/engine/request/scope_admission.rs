//! Original-request work and retained custody for relation witness preparation.
use super::PreparedRelationIntegrityScopes;
use crate::storage::substrate::maintenance::{ChangeWork, MaintenanceRefusal};
use crate::transactions::data::TransactionCommitError;
use std::cell::{Cell, RefCell};

type Scopes = Option<PreparedRelationIntegrityScopes>;
pub(super) fn prepare(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    operation: impl FnOnce(
        &dyn Fn(u64, u64) -> Result<(), TransactionCommitError>,
    ) -> Result<Scopes, TransactionCommitError>,
) -> Result<Scopes, TransactionCommitError> {
    let memory: RefCell<Option<worth_execution::ExecutionMemoryReservation>> = RefCell::new(None);
    let extent = Cell::new(0_u64);
    let result = ChangeWork::run_phase(execution, "relation_source_witness", |work| {
        let work = RefCell::new(work);
        operation(&|units, bytes| {
            work.borrow_mut().prepare(units, 0).map_err(refusal)?;
            if bytes > 0 {
                let total = extent.get().saturating_add(bytes);
                let mut memory = memory.borrow_mut();
                if let Some(ticket) = memory.as_mut() {
                    ticket.grow_for_allocation(total).map_err(|denial| {
                        TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
                            crate::transactions::data::ConflictClass::ExecutionAllocationDenied { denial },
                        ))
                    })?;
                } else {
                    let total = if extent.get()==0 {
                        total.saturating_add((std::mem::size_of::<worth_execution::ExecutionMemoryReservation>() + 2 * std::mem::size_of::<usize>()) as u64)
                    } else {total};
                    *memory = execution.reserve_retained_memory(total)
                        .map_err(|cause| TransactionCommitError::execution(cause.into()))?;
                    extent.set(total);
                    return Ok(());
                }
                extent.set(total);
            }
            Ok(())
        }).map_err(ScopeFailure::Preparation)
    })
    .map_err(|error| match error {
        ScopeFailure::Work(error) => refusal(error),
        ScopeFailure::Preparation(error) => error,
    })?;
    Ok(result.map(|scopes| scopes.with_memory(memory.into_inner())))
}

fn refusal(error: MaintenanceRefusal) -> TransactionCommitError {
    match error {
        MaintenanceRefusal::Execution(cause) => {
            TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
                kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
                partition_identity: None,
            })
        }
        MaintenanceRefusal::MissingPredecessor | MaintenanceRefusal::UnavailableChange => {
            // Proof: ChangeWork admits work only; it never dispatches a
            // maintainer or projects a predecessor/change in this driver.
            unreachable!("a request work scope cannot refuse a maintenance predecessor")
        }
    }
}

enum ScopeFailure {
    Work(MaintenanceRefusal),
    Preparation(TransactionCommitError),
}
impl From<MaintenanceRefusal> for ScopeFailure {
    fn from(error: MaintenanceRefusal) -> Self {
        Self::Work(error)
    }
}

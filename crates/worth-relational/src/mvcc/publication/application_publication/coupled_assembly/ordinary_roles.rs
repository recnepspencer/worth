//! Real allocation bindings resolve ordinary output correspondence without ownership.
use super::*;

impl RelationalCoupledProducerPosture {
    /// Allocation bindings, not an origin, resolve an ordinary writer's local creates.
    pub(super) fn resolve_ordinary_roles(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        created: &crate::transactions::data::CommitCreatedEntityBindings,
    ) -> Result<
        Option<worth_execution::ExecutionArray<crate::identity::data::EntityId>>,
        TransactionCommitError,
    > {
        let Self::Ordinary { roles, .. } = self else {
            return Ok(None);
        };
        resolve_applied_roles(execution, roles, created).map(Some)
    }
}

pub(super) fn resolve_applied_roles(
    execution: &ExecutionRequest<'_, '_>,
    roles: &[RequiredOutputRoleTarget],
    created: &crate::transactions::data::CommitCreatedEntityBindings,
) -> Result<worth_execution::ExecutionArray<crate::identity::data::EntityId>, TransactionCommitError>
{
    let mut ids = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, roles.len())
        .map_err(array_stop)?;
    execution
        .run_native_with_checkpoint(|context| {
            for role in roles {
                let id = match &role.target {
                    crate::transactions::data::EntityReference::Existing(id) => {
                        checkpoint(context, 1).map_err(custody::execution_stop)?;
                        *id
                    }
                    crate::transactions::data::EntityReference::Created(key) => {
                        checkpoint(
                            context,
                            created.resolve_work_bound(key).ok_or_else(|| {
                                origin_denial("applied output allocation is missing")
                            })?,
                        )
                        .map_err(custody::execution_stop)?;
                        created
                            .resolve(key)
                            .ok_or_else(|| origin_denial("applied output allocation is missing"))?
                    }
                };
                ids.push(id).map_err(array_stop)?;
            }
            Ok::<_, TransactionCommitError>(())
        })
        .map_err(custody::execution_stop)?
        .0?;
    ids.seal().map_err(array_stop)
}

fn array_stop(cause: worth_execution::ExecutionRequestArrayDenial) -> TransactionCommitError {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            custody::execution_stop(cause)
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(denial) => {
            TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::ExecutionAllocationDenied { denial },
            ))
        }
    }
}

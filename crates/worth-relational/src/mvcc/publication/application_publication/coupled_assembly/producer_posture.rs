//! A real producer can participate without claiming publication ownership.
use crate::execution::request_stage::checkpoint;
use crate::mvcc::publication::application_publication::custody;
use crate::mvcc::publication::application_publication::output_meaning::{
    RequiredOutputPublicationMeaning, RequiredOutputRoleTarget,
};
use crate::mvcc::publication::application_publication::{
    RelationalInstalledApplicationPublicationAuthority, RelationalRequiredOutputRouteAbsence,
    RequiredOutputPublication,
};
use crate::transactions::data::outcomes::transaction_commit_error::origin_denial;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

/// Untrusted posture selectors with the installed producer's concrete admission.
/// Native admits these against C before applying any participant commands to D.
#[derive(Debug)]
pub enum RelationalCoupledProducerPosture {
    Publication(RequiredOutputPublication),
    Ordinary {
        admission: worth_proof::AdmittedRequiredOutputPublication,
        absence: RelationalRequiredOutputRouteAbsence,
        meaning: RequiredOutputPublicationMeaning,
        roles: Vec<RequiredOutputRoleTarget>,
    },
}
impl RelationalCoupledProducerPosture {
    pub(super) fn roles(&self) -> &[RequiredOutputRoleTarget] {
        match self {
            Self::Publication(
                RequiredOutputPublication::Birth { roles, .. }
                | RequiredOutputPublication::Preserve { roles, .. },
            )
            | Self::Ordinary { roles, .. } => roles,
            _ => unreachable!("Native admitted a producer participant"),
        }
    }
    pub fn producing_meaning(&self) -> Option<&RequiredOutputPublicationMeaning> {
        match self {
            Self::Publication(publication) => publication.producing_meaning(),
            Self::Ordinary { meaning, .. } => Some(meaning),
        }
    }
    pub(crate) fn lineage_attribution(&self) -> Option<(&str, [u8; 32])> {
        match self {
            Self::Publication(publication) => publication.lineage_attribution(),
            Self::Ordinary { admission, .. } => {
                Some((admission.producer(), *admission.dependency_identity()))
            }
        }
    }
    pub(crate) fn admitted_by(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
    ) -> bool {
        match self {
            Self::Publication(publication) => publication.admitted_by(installed),
            Self::Ordinary { admission, .. } => {
                installed.authority.admits_required_output(admission)
            }
        }
    }
    pub(crate) fn owned(&self) -> Option<&RequiredOutputPublication> {
        match self {
            Self::Publication(publication) => Some(publication),
            Self::Ordinary { .. } => None,
        }
    }
    pub(super) fn roles_mut(&mut self) -> &mut Vec<RequiredOutputRoleTarget> {
        match self {
            Self::Publication(
                RequiredOutputPublication::Birth { roles, .. }
                | RequiredOutputPublication::Preserve { roles, .. },
            )
            | Self::Ordinary { roles, .. } => roles,
            _ => unreachable!("Native admitted a producer participant"),
        }
    }
}

impl RelationalCoupledProducerPosture {
    /// Allocation bindings, not an origin, resolve an ordinary writer's local creates.
    pub(super) fn resolve_ordinary_roles(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        created: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
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
    created: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
) -> Result<worth_execution::ExecutionArray<crate::identity::data::EntityId>, TransactionCommitError>
{
    let mut ids = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, roles.len())
        .map_err(array_stop)?;
    execution
        .run_native_with_checkpoint(|context| {
            for role in roles {
                let id = match &role.target {
                    crate::transactions::data::primitives::EntityReference::Existing(id) => {
                        checkpoint(context, 1).map_err(custody::execution_stop)?;
                        *id
                    }
                    crate::transactions::data::primitives::EntityReference::Created(key) => {
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
            TransactionCommitError::conflict(crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                crate::transactions::data::outcomes::conflict_class::ConflictClass::ExecutionAllocationDenied { denial },
            ))
        }
    }
}

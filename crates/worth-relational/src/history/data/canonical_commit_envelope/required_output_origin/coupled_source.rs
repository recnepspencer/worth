//! Install the authored prefix's actual posture without inventing a retirement.
use super::*;
use crate::{
    history::data::RequiredOutputPublicationPositions,
    mvcc::{CoupledSource, RelationalAuthoredRetirement, RelationalCoupledAuthoredPublication},
    storage::overlay::WorkingState,
    transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings, RecordRef},
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[allow(clippy::too_many_arguments)]
pub(super) fn install_empty(
    envelope: &mut CanonicalCommitEnvelope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    group: &RelationalCoupledAuthoredPublication,
    lease: Option<&ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    let custody = &group.binding.owners;
    if let (
        CoupledSource::Retirement {
            anchor,
            birth,
            co_anchors,
            roles,
        },
        Some(admission),
    ) = (&custody.source, &group.binding.retirement_admission)
    {
        return super::authored_cascade::install(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            admission,
            RelationalAuthoredRetirement {
                anchor: *anchor,
                owned_members: false,
                birth: *birth,
                co_anchors: co_anchors.clone(),
                roles: roles.clone(),
                before: custody.before.clone(),
                publications: custody.retirements.clone(),
            },
            lease,
            validated_history,
            context,
        );
    }
    Ok(basis.inner.root.output_publication_positions.clone())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    group: &RelationalCoupledAuthoredPublication,
    lease: Option<&ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<coupled_authored::CanonicalCoupledAuthoredSources, TransactionCommitError> {
    use coupled_authored::CanonicalCoupledAuthoredSources as Sources;
    let custody = &group.binding.owners;
    match &custody.source {
        CoupledSource::Edit(source) if group.stages.is_empty() => Ok(Sources::Edit(*source)),
        CoupledSource::Edit(source) => Ok(Sources::Stages(
            std::iter::once(*source)
                .chain(group.stages.iter().map(|stage| stage.source))
                .collect(),
        )),
        CoupledSource::Restoration { plan, .. } => {
            super::authored_restoration::install(
                envelope,
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed,
                group.binding.retirement_admission.as_deref(),
                plan,
                lease,
                validated_history,
                context,
            )?;
            Ok(Sources::Restoration(
                envelope
                    .authored_cascade_restoration
                    .take()
                    .expect("Native installed authored source restoration"),
            ))
        }
        CoupledSource::Retirement {
            anchor,
            birth,
            co_anchors,
            ..
        } => {
            if group.binding.retirement_admission.is_some() {
                install_empty(
                    envelope,
                    execution,
                    runtime,
                    basis,
                    created,
                    created_relations,
                    state,
                    installed,
                    group,
                    lease,
                    validated_history,
                    context,
                )?;
            } else {
                require_authored_sources(envelope, *anchor, co_anchors, context)?;
                envelope.authored_cascade_retirement =
                    Some(Arc::new(CanonicalAuthoredRetirement {
                        anchor: *anchor,
                        owned_members: false,
                        birth: *birth,
                        co_anchors: co_anchors.clone(),
                        publications: Vec::new(),
                    }));
            }
            Ok(Sources::Retirement(
                envelope
                    .authored_cascade_retirement
                    .take()
                    .expect("Native installed authored source retirement"),
            ))
        }
    }
}

fn require_authored_sources(
    envelope: &CanonicalCommitEnvelope,
    anchor: crate::identity::data::EntityId,
    co_anchors: &[(
        crate::identity::data::EntityId,
        crate::history::data::CommitId,
    )],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    use crate::publication::{
        data::{AuthoredRetirementRefusal as Cause, PublicationError},
        patch::data::RecordStructuralChange as Change,
    };
    let anchors = std::iter::once(anchor)
        .chain(co_anchors.iter().map(|(id, _)| *id))
        .collect::<Vec<_>>();
    for id in &anchors {
        crate::execution::request_stage::checkpoint(
            context,
            envelope.patch.authoritative_record_patches.len() as u64 + 1,
        )
        .map_err(origin_request)?;
        if !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|p| p.target == RecordRef::Entity(*id) && p.structural_change == Change::Deleted)
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(Cause::MissingMember, RecordRef::Entity(*id)),
            ));
        }
    }
    for patch in &envelope.patch.authoritative_record_patches {
        crate::execution::request_stage::checkpoint(context, anchors.len() as u64 + 1)
            .map_err(origin_request)?;
        if let RecordRef::Entity(id) = patch.target
            && patch.structural_change == Change::Deleted
            && !anchors.contains(&id)
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(Cause::ExtraEntity, RecordRef::Entity(id)),
            ));
        }
    }
    Ok(())
}

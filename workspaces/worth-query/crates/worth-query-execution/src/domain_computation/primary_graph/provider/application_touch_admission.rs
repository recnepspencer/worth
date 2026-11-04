//! Admission of Relational validated candidate touches against one installed contract.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::{
    WorthQueryOperationGraphReadContract, WorthQueryOperationReadTouchOverlapIndex,
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::mvcc::{
    ValidatedMutationTouch, ValidatedMutationTouchProjectionWork, ValidatedRelationalProposal,
};

mod installed_touches;
mod struct_aspect;
use installed_touches::resolve_installed_touches;
#[cfg(test)]
mod tests;

use super::super::schema_layout::WorthQueryPrimaryGraphLayout;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryInstalledTouchAdmissionEvidence {
    projection_work: ValidatedMutationTouchProjectionWork,
    validated_application_touches: usize,
    installed_touch_scopes: usize,
    read_touch_overlaps: usize,
}

impl WorthQueryInstalledTouchAdmissionEvidence {
    pub(super) const fn projection_work(self) -> ValidatedMutationTouchProjectionWork {
        self.projection_work
    }

    pub(super) const fn validated_application_touches(self) -> usize {
        self.validated_application_touches
    }

    pub(super) const fn installed_touch_scopes(self) -> usize {
        self.installed_touch_scopes
    }

    pub(super) const fn read_touch_overlaps(self) -> usize {
        self.read_touch_overlaps
    }
}

/// One thing a candidate authored, in the form the installed touch contract
/// can be compared against.
///
/// Every variant here is an authoring act, because that is all a touch
/// contract can declare. A candidate touch that authors nothing â€” a
/// revalidation demand â€” resolves to no variant at all, and on a kind this
/// application governs it is refused: an operation cannot put its own records
/// in front of rules under a contract that never admitted the effect.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ResolvedApplicationTouch {
    CreateEntity(KindId),
    DeleteEntity(KindId),
    WriteEntityField(KindId, AspectFieldLocator),
    LinkRelation(KindId),
    InitializeRelationField(KindId, AspectFieldLocator),
    UnlinkRelation(KindId),
    RemoveEntityAspect(KindId, worth_foundational::facade::AspectKey),
}

pub(super) fn admit_validated_application_touches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: &ValidatedRelationalProposal,
    layout: &WorthQueryPrimaryGraphLayout,
    graph_reads: &WorthQueryOperationGraphReadContract,
    touch_contract: &WorthQueryOperationTouchContract,
    overlap: &WorthQueryOperationReadTouchOverlapIndex,
) -> Result<WorthQueryInstalledTouchAdmissionEvidence, TouchAdmissionFailure> {
    execution.in_scope(|_| admit_in_scope(execution, candidate, layout, graph_reads, touch_contract, overlap))
        .map_err(|stop| TouchAdmissionFailure::Projection(
            worth_relational::facade::mvcc::ValidatedMutationTouchProjectionError::Execution(
                worth_relational::facade::transactions::CommitExecutionDenial {
                    kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(stop.into()),
                    partition_identity: None,
                })))?
}
fn admit_in_scope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: &ValidatedRelationalProposal,
    layout: &WorthQueryPrimaryGraphLayout,
    graph_reads: &WorthQueryOperationGraphReadContract,
    touch_contract: &WorthQueryOperationTouchContract,
    overlap: &WorthQueryOperationReadTouchOverlapIndex,
) -> Result<WorthQueryInstalledTouchAdmissionEvidence, TouchAdmissionFailure> {
    validate_overlap_basis(graph_reads, touch_contract, overlap)?;
    let installed = resolve_installed_touches(layout, touch_contract)?;
    let validated = if candidate.has_coupled_authored_publication() {
        candidate.authored_mutation_touches(execution)
    } else {
        candidate.application_mutation_touches(execution)
    }
    .map_err(TouchAdmissionFailure::Projection)?;
    admit_projected_touches(
        execution,
        &validated,
        layout,
        touch_contract,
        overlap,
        installed,
    )
}

pub(super) fn admit_coupled_contribution_touches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    validated: &worth_relational::facade::mvcc::ValidatedMutationTouches<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    graph_reads: &WorthQueryOperationGraphReadContract,
    touch_contract: &WorthQueryOperationTouchContract,
    overlap: &WorthQueryOperationReadTouchOverlapIndex,
) -> Result<WorthQueryInstalledTouchAdmissionEvidence, TouchAdmissionFailure> {
    validate_overlap_basis(graph_reads, touch_contract, overlap)?;
    let installed = resolve_installed_touches(layout, touch_contract)?;
    admit_projected_touches(
        execution,
        validated,
        layout,
        touch_contract,
        overlap,
        installed,
    )
}

fn admit_projected_touches(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    validated: &worth_relational::facade::mvcc::ValidatedMutationTouches<'_>,
    layout: &WorthQueryPrimaryGraphLayout,
    touch_contract: &WorthQueryOperationTouchContract,
    overlap: &WorthQueryOperationReadTouchOverlapIndex,
    installed: BTreeMap<ResolvedApplicationTouch, usize>,
) -> Result<WorthQueryInstalledTouchAdmissionEvidence, TouchAdmissionFailure> {
    let mut validated_application_touches = 0usize;
    let mut read_touch_overlaps = 0usize;
    for actual in validated.touches() {
        let Some(key) = validated_application_touch(actual, layout)? else {
            continue;
        };
        let touch_index = installed.get(&key).copied().ok_or(())?;
        validated_application_touches = validated_application_touches.checked_add(1).ok_or(())?;
        read_touch_overlaps = read_touch_overlaps
            .checked_add(overlap.touch_overlap_count(touch_index).ok_or(())?)
            .ok_or(())?;
    }
    for actual in validated.struct_touches() {
        if !layout.is_application_entity_kind(actual.kind()) {
            continue;
        }
        let index = struct_aspect::admit(execution, actual, layout, touch_contract)?;
        validated_application_touches = validated_application_touches.checked_add(1).ok_or(())?;
        read_touch_overlaps = read_touch_overlaps
            .checked_add(overlap.touch_overlap_count(index).ok_or(())?)
            .ok_or(())?;
    }
    Ok(WorthQueryInstalledTouchAdmissionEvidence {
        projection_work: validated.work(),
        validated_application_touches,
        installed_touch_scopes: touch_contract.scopes().len(),
        read_touch_overlaps,
    })
}

fn validate_overlap_basis(
    graph_reads: &WorthQueryOperationGraphReadContract,
    touches: &WorthQueryOperationTouchContract,
    overlap: &WorthQueryOperationReadTouchOverlapIndex,
) -> Result<(), ()> {
    let reads = graph_reads
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes());
    if overlap.touches() != touches.scopes()
        || reads.clone().count() != overlap.reads().len()
        || !reads
            .zip(overlap.reads())
            .all(|(left, right)| left == right)
    {
        return Err(());
    }
    Ok(())
}

fn validated_application_touch(
    touch: &ValidatedMutationTouch,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<Option<ResolvedApplicationTouch>, ()> {
    match touch {
        ValidatedMutationTouch::CreateEntity { kind } => {
            application_entity_touch(layout, *kind, ResolvedApplicationTouch::CreateEntity(*kind))
        }
        ValidatedMutationTouch::DeleteEntity { kind } => {
            application_entity_touch(layout, *kind, ResolvedApplicationTouch::DeleteEntity(*kind))
        }
        ValidatedMutationTouch::WriteEntityField { kind, locator } => application_entity_touch(
            layout,
            *kind,
            ResolvedApplicationTouch::WriteEntityField(*kind, locator.clone()),
        ),
        ValidatedMutationTouch::LinkRelation { kind } => {
            application_relation_touch(layout, *kind, ResolvedApplicationTouch::LinkRelation(*kind))
        }
        ValidatedMutationTouch::InitializeRelationField { kind, locator } => {
            application_relation_touch(
                layout,
                *kind,
                ResolvedApplicationTouch::InitializeRelationField(*kind, locator.clone()),
            )
        }
        ValidatedMutationTouch::UnlinkRelation { kind } => application_relation_touch(
            layout,
            *kind,
            ResolvedApplicationTouch::UnlinkRelation(*kind),
        ),
        ValidatedMutationTouch::RevalidateEntity { kind } => (!layout
            .is_application_entity_kind(*kind))
        .then_some(None)
        .ok_or(()),
        ValidatedMutationTouch::RevalidateRelation { kind } => (!layout
            .is_application_relation_kind(*kind))
        .then_some(None)
        .ok_or(()),
        ValidatedMutationTouch::UnrepresentableEntityMutation { kind } => (!layout
            .is_application_entity_kind(*kind))
        .then_some(None)
        .ok_or(()),
        ValidatedMutationTouch::UnrepresentableRelationMutation { kind } => (!layout
            .is_application_relation_kind(*kind))
        .then_some(None)
        .ok_or(()),
    }
}

fn application_entity_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    kind: KindId,
    touch: ResolvedApplicationTouch,
) -> Result<Option<ResolvedApplicationTouch>, ()> {
    Ok(layout.is_application_entity_kind(kind).then_some(touch))
}

fn application_relation_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    kind: KindId,
    touch: ResolvedApplicationTouch,
) -> Result<Option<ResolvedApplicationTouch>, ()> {
    Ok(layout.is_application_relation_kind(kind).then_some(touch))
}

#[derive(Debug)]
pub(super) enum TouchAdmissionFailure {
    Contract,
    Projection(worth_relational::facade::mvcc::ValidatedMutationTouchProjectionError),
}
impl From<()> for TouchAdmissionFailure {
    fn from(_: ()) -> Self {
        Self::Contract
    }
}
impl TouchAdmissionFailure {
    pub(super) fn into_invariant_failure(
        self,
    ) -> crate::domain_computation::WorthQueryInvariantExecutionFailure {
        use worth_relational::facade::mvcc::ValidatedMutationTouchProjectionError as Error;
        match self {
            Self::Projection(Error::AuthoredRestoration(error)) =>
                crate::domain_computation::WorthQueryInvariantExecutionFailure::authored_retirement(error),
            Self::Projection(Error::Execution(denial)) =>
                super::invariant_execution_failure::map_execution_failure(denial),
            Self::Projection(Error::Allocation(cause)) => {
                match cause.classify() {
                    worth_execution::ExecutionAllocationDenialClass::Request(stop) => {
                        let cause = worth_relational::facade::transactions::RelationalExecutionDenialCause::from(stop);
                        super::invariant_execution_failure::map_execution_failure(
                                worth_relational::facade::transactions::CommitExecutionDenial {
                                    kind: worth_relational::facade::transactions::CommitExecutionDenialKind::Cause(cause),
                                    partition_identity: None,
                                })
                    }
                    worth_execution::ExecutionAllocationDenialClass::Physical(_) =>
                        crate::domain_computation::WorthQueryInvariantExecutionFailure::physical_allocation(
                            cause, "native mutation-touch backing refused"),
                }
            }
            Self::Contract | Self::Projection(Error::MissingEntityKind(_) | Error::MissingRelationKind(_)) =>
                super::invariant_execution::touch_failure(),
        }
    }
}

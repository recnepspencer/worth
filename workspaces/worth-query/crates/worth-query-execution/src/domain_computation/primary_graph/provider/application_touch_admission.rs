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

mod struct_aspect;
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
    UnlinkRelation(KindId),
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
    let validated = candidate
        .mutation_touches(execution)
        .map_err(TouchAdmissionFailure::Projection)?;
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

fn resolve_installed_touches(
    layout: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryOperationTouchContract,
) -> Result<BTreeMap<ResolvedApplicationTouch, usize>, ()> {
    let mut resolved = BTreeMap::new();
    for (index, scope) in contract.scopes().iter().enumerate() {
        let key = resolve_installed_touch(layout, scope)?;
        if resolved.insert(key, index).is_some() {
            return Err(());
        }
    }
    Ok(resolved)
}

fn resolve_installed_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    scope: &WorthQueryOperationTouchScope,
) -> Result<ResolvedApplicationTouch, ()> {
    match scope {
        WorthQueryOperationTouchScope::CreateEntity(scope) => layout
            .entity_kind(scope.entity())
            .map(ResolvedApplicationTouch::CreateEntity)
            .ok_or(()),
        WorthQueryOperationTouchScope::DeleteEntity(scope) => layout
            .entity_kind(scope.entity())
            .map(ResolvedApplicationTouch::DeleteEntity)
            .ok_or(()),
        WorthQueryOperationTouchScope::WriteField(scope) => {
            if layout.aspect_contract(scope.entity(), scope.contract().key())
                != Some(scope.contract())
                || scope.field_path().fields().len() != 1
            {
                return Err(());
            }
            let field = &scope.field_path().fields()[0];
            let kind = layout.entity_kind(scope.entity()).ok_or(())?;
            let locator = layout
                .field_locator(
                    scope.entity(),
                    scope.contract().key().as_str(),
                    field.as_str(),
                )
                .cloned()
                .ok_or(())?;
            if locator.field_path() != scope.field_path() {
                return Err(());
            }
            Ok(ResolvedApplicationTouch::WriteEntityField(kind, locator))
        }
        WorthQueryOperationTouchScope::LinkRelation(scope) => {
            resolve_relation_touch(layout, scope, true)
        }
        WorthQueryOperationTouchScope::UnlinkRelation(scope) => {
            resolve_relation_touch(layout, scope, false)
        }
        WorthQueryOperationTouchScope::DeclaredDomain(_) => Err(()),
    }
}

fn resolve_relation_touch(
    layout: &WorthQueryPrimaryGraphLayout,
    scope: &worth_query_installation::facade::WorthQueryOperationRelationTouchScope,
    link: bool,
) -> Result<ResolvedApplicationTouch, ()> {
    let relation = layout.relation(scope.relation()).ok_or(())?;
    if layout.entity_kind(scope.from()) != Some(relation.from)
        || layout.entity_kind(scope.to()) != Some(relation.to)
    {
        return Err(());
    }
    Ok(if link {
        ResolvedApplicationTouch::LinkRelation(relation.kind)
    } else {
        ResolvedApplicationTouch::UnlinkRelation(relation.kind)
    })
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

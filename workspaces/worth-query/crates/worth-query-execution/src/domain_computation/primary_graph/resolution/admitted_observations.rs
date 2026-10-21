//! Fresh principal observations at one issued native root and one request meter.

use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::AuthoritativeRecordAspectState;
use worth_foundational::facade::ContractValidatedAspectValue;
use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;
use worth_relational::facade::runtime::RelationalAdjacencyVisit;
use worth_relational::facade::runtime::RelationalEntityMetadata;
use worth_relational::facade::runtime::RelationalFieldRevision;
use worth_relational::facade::runtime::VisibilityProjectionView;
use worth_relational::facade::storage::RecordLifecycleState;

use super::super::observations::WorthQueryPrincipalMappingObservation;
use super::super::observations::WorthQueryPrincipalTargetObservation;

use super::super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::admitted_lookup::CheckedMappingCandidate;

#[derive(Debug)]
pub(super) enum AdmittedPrincipalObservationStop {
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

type ObservationResult<T> = Result<T, AdmittedPrincipalObservationStop>;

fn stale() -> AdmittedPrincipalObservationStop {
    AdmittedPrincipalObservationStop::Semantic(WorthQueryPrincipalResolutionDenialKind::Refused(
        PrincipalRefusal::StalePrincipalProof,
    ))
}

fn authoritative_locator(planned: &AspectFieldLocator) -> ObservationResult<AspectFieldLocator> {
    Ok(AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        planned.aspect().aspect_key().clone(),
        planned.field_path().clone(),
    ))
}

fn revision(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    entity: EntityId,
    planned: &AspectFieldLocator,
) -> ObservationResult<RelationalFieldRevision> {
    let locator = authoritative_locator(planned)?;
    view.entity_field_revision(execution, entity, &locator)
        .map_err(|cause| {
            AdmittedPrincipalObservationStop::Semantic(
                super::super::resolution_denial::relational_read_denial(
                    cause,
                    "principal revision",
                )
                .kind(),
            )
        })?
        .ok_or_else(stale)
}

fn selected_value(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    entity: EntityId,
    expected_kind: KindId,
    locator: &AspectFieldLocator,
    wrong_kind: WorthQueryPrincipalResolutionDenialKind,
) -> ObservationResult<(RelationalEntityMetadata, AspectValue)> {
    execution.consult().map_err(|cause| {
        AdmittedPrincipalObservationStop::Semantic(
            super::super::resolution_denial::relational_read_denial(
                cause.into(),
                "principal selected value",
            )
            .kind(),
        )
    })?;
    let result = view
        .with_exact_entity_state(entity, |metadata, authoritative| {
            if metadata.kind_id != expected_kind {
                return Err(AdmittedPrincipalObservationStop::Semantic(wrong_kind));
            }
            if metadata.lifecycle != RecordLifecycleState::Live {
                return Ok(None);
            }
            let Some(authoritative) = authoritative else {
                return Ok(None);
            };
            Ok(selected_field_value(authoritative, locator)?.map(|value| (metadata, value)))
        })
        .map_err(|_| stale())?;
    result.unwrap_or(Ok(None))?.ok_or_else(stale)
}

fn selected_field_value(
    authoritative: &AuthoritativeRecordAspectState,
    locator: &AspectFieldLocator,
) -> ObservationResult<Option<AspectValue>> {
    let field = locator.field_path().fields().first().ok_or_else(stale)?;
    let aspect = locator.aspect().aspect_key();
    let Some(value) = authoritative.get(aspect) else {
        return Ok(None);
    };
    selected_field_from_aspect(value, field)
}

fn selected_field_from_aspect(
    value: &ContractValidatedAspectValue,
    field: &FieldKey,
) -> ObservationResult<Option<AspectValue>> {
    let value = match value.view() {
        ContractValidatedAspectValueView::Scalar(_) => None,
        ContractValidatedAspectValueView::Struct(value) => value.get(field),
    };
    let Some(value) = value else {
        return Ok(None);
    };
    Ok(Some(value.clone()))
}

pub(super) fn observe_mapping(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    candidate: &CheckedMappingCandidate<'_, '_, '_>,
) -> ObservationResult<WorthQueryPrincipalMappingObservation> {
    let view = candidate.view();
    let mapping = candidate.mapping();
    let layout = candidate.resolution().layout;
    let expected_identity = candidate.resolution().expected_identity;
    let identity_revision = revision(execution, view, mapping, &layout.identity_locator)?;
    let status_revision = revision(execution, view, mapping, &layout.status_locator)?;
    // The immediately preceding Native indexed lookup compared the actual
    // identity at this same issued view. Re-read live kind and status, while
    // carrying that checked value into freshness without a second identity get.
    let (metadata, status) = selected_value(
        execution,
        view,
        mapping,
        layout.mapping_kind,
        &layout.status_locator,
        WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof),
    )?;
    let AspectValue::Bool(enabled) = status else {
        return Err(stale());
    };

    Ok(WorthQueryPrincipalMappingObservation {
        entity_id: metadata.entity_id,
        kind_id: metadata.kind_id,
        identity: expected_identity.clone(),
        enabled,
        identity_revision,
        status_revision,
    })
}

pub(super) fn observe_target(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    mapping: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
) -> ObservationResult<WorthQueryPrincipalTargetObservation> {
    let mut selected = None;
    view.try_visit_adjacency_ids(
        execution,
        mapping,
        layout.relation_kind,
        RelationalAdjacencyDirection::Outgoing,
        |visit| {
            match visit {
                RelationalAdjacencyVisit::Prepare { .. } => {}
                RelationalAdjacencyVisit::List => {}
                RelationalAdjacencyVisit::Relation(id) => {
                    let Some(relation) = view.exact_relation_metadata(id).map_err(|_| stale())?
                    else {
                        return Ok(());
                    };
                    if relation.kind_id == layout.relation_kind
                        && relation.source == mapping
                        && relation.lifecycle == RecordLifecycleState::Live
                        && selected.replace(relation).is_some()
                    {
                        return Err(AdmittedPrincipalObservationStop::Semantic(
                            WorthQueryPrincipalResolutionDenialKind::Refused(
                                PrincipalRefusal::AmbiguousPrincipalTarget,
                            ),
                        ));
                    }
                }
            }
            Ok(())
        },
    )
    .map_err(|cause| {
        AdmittedPrincipalObservationStop::Semantic(
            super::super::resolution_denial::relational_read_denial(cause, "principal adjacency")
                .kind(),
        )
    })??;
    let relation = selected.ok_or(AdmittedPrincipalObservationStop::Semantic(
        WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::MissingPrincipalTarget),
    ))?;
    let (principal, principal_identity) = selected_value(
        execution,
        view,
        relation.target,
        layout.principal_kind,
        &layout.principal_identity_locator,
        WorthQueryPrincipalResolutionDenialKind::Refused(
            PrincipalRefusal::WrongPrincipalTargetKind,
        ),
    )?;
    let principal_identity_revision = revision(
        execution,
        view,
        relation.target,
        &layout.principal_identity_locator,
    )?;
    let adjacency_revision = view
        .exact_adjacency_structural_revision(
            mapping,
            layout.relation_kind,
            RelationalAdjacencyDirection::Outgoing,
        )
        .map_err(|_| stale())?;
    Ok(WorthQueryPrincipalTargetObservation {
        relation_id: relation.relation_id,
        relation_kind: relation.kind_id,
        source: relation.source,
        target: relation.target,
        principal_kind: principal.kind_id,
        principal_identity,
        principal_identity_revision,
        adjacency_revision,
    })
}

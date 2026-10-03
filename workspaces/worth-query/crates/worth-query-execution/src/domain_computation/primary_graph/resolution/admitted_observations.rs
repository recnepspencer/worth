//! Fresh principal observations at one issued native root and one request meter.

use worth_foundational::facade::{
    AspectFieldLocator, AspectValue, AuthoritativeRecordAspectState, ContractValidatedAspectValue,
    ContractValidatedAspectValueView, FieldKey, LocatorAuthority,
};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::{
    RelationalAdjacencyDirection, RelationalAdjacencyVisit, RelationalEntityMetadata,
    RelationalFieldRevision, VisibilityProjectionView,
};
use worth_relational::facade::storage::RecordLifecycleState;

use super::super::observations::{
    WorthQueryPrincipalMappingObservation, WorthQueryPrincipalTargetObservation,
};
use super::super::output_lineage::invalidation::InvalidationEditAdmission;
use super::super::schema_layout::WorthQueryPrimaryPrincipalBindingLayout;
use super::super::WorthQueryPrincipalResolutionDenialKind;
use super::admitted_lookup::CheckedMappingCandidate;

#[derive(Debug)]
pub(super) enum AdmittedPrincipalObservationStop {
    Admission(CompanionPreflightStop),
    Semantic(WorthQueryPrincipalResolutionDenialKind),
}

type ObservationResult<T> = Result<T, AdmittedPrincipalObservationStop>;

fn stale() -> AdmittedPrincipalObservationStop {
    AdmittedPrincipalObservationStop::Semantic(
        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
    )
}

fn prepare(
    admission: &mut InvalidationEditAdmission,
    work: u64,
    bytes: u64,
) -> ObservationResult<()> {
    admission
        .admit_read_scratch(bytes)
        .and_then(|()| admission.charge_external_work(work))
        .map_err(AdmittedPrincipalObservationStop::Admission)
}

fn ordered_text_lookup_work(count: usize, text: &str) -> u64 {
    let mut levels = 1_u64;
    let mut minimum = 1_usize;
    while count > minimum {
        minimum = minimum.saturating_mul(6).saturating_add(5);
        levels = levels.saturating_add(1);
        if minimum == usize::MAX {
            break;
        }
    }
    // A single ordered get cannot compare more distinct keys than the map
    // actually contains, even when the conservative height bound is larger.
    levels
        .saturating_mul(11)
        .min(u64::try_from(count).unwrap_or(u64::MAX))
        .saturating_mul(
            u64::try_from(text.len())
                .unwrap_or(u64::MAX)
                .saturating_add(1),
        )
}

fn authoritative_locator(
    planned: &AspectFieldLocator,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<AspectFieldLocator> {
    let fields = planned.field_path().fields();
    prepare(
        admission,
        u64::try_from(fields.len()).map_err(|_| {
            AdmittedPrincipalObservationStop::Admission(CompanionPreflightStop::WorkCounterOverflow)
        })?,
        0,
    )?;
    let field_text = fields
        .iter()
        .try_fold(0_usize, |total, field| {
            total.checked_add(field.as_str().len())
        })
        .ok_or(AdmittedPrincipalObservationStop::Admission(
            CompanionPreflightStop::WorkCounterOverflow,
        ))?;
    let text = field_text
        .checked_add(planned.aspect().aspect_key().as_str().len())
        .ok_or(AdmittedPrincipalObservationStop::Admission(
            CompanionPreflightStop::WorkCounterOverflow,
        ))?;
    let backing = fields
        .len()
        .checked_mul(std::mem::size_of::<worth_foundational::facade::FieldKey>())
        .and_then(|slots| slots.checked_add(text))
        .ok_or(AdmittedPrincipalObservationStop::Admission(
            CompanionPreflightStop::PreparationMemoryCounterOverflow,
        ))?;
    prepare(
        admission,
        u64::try_from(text.saturating_add(fields.len())).unwrap_or(u64::MAX),
        u64::try_from(backing).unwrap_or(u64::MAX),
    )?;
    Ok(AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        planned.aspect().aspect_key().clone(),
        planned.field_path().clone(),
    ))
}

fn revision(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    planned: &AspectFieldLocator,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<RelationalFieldRevision> {
    let locator = authoritative_locator(planned, admission)?;
    view.entity_field_revision_admitted(entity, &locator, |work| {
        admission.charge_external_work(work)
    })
    .map_err(AdmittedPrincipalObservationStop::Admission)?
    .ok_or_else(stale)
}

fn selected_value(
    view: &VisibilityProjectionView<'_>,
    entity: EntityId,
    expected_kind: KindId,
    locator: &AspectFieldLocator,
    wrong_kind: WorthQueryPrincipalResolutionDenialKind,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<(RelationalEntityMetadata, AspectValue)> {
    let Some(read_work) = view.exact_entity_state_read_work_bound() else {
        return Err(stale());
    };
    prepare(admission, read_work, 0)?;
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
            Ok(selected_field_value(authoritative, locator, admission)?
                .map(|value| (metadata, value)))
        })
        .map_err(|_| stale())?;
    result.unwrap_or(Ok(None))?.ok_or_else(stale)
}

fn selected_field_value(
    authoritative: &AuthoritativeRecordAspectState,
    locator: &AspectFieldLocator,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<Option<AspectValue>> {
    prepare(admission, 3, 0)?;
    let field = locator.field_path().fields().first().ok_or_else(stale)?;
    let aspect = locator.aspect().aspect_key();
    prepare(
        admission,
        ordered_text_lookup_work(authoritative.aspects().len(), aspect.as_str()),
        0,
    )?;
    let Some(value) = authoritative.get(aspect) else {
        return Ok(None);
    };
    selected_field_from_aspect(value, field, admission)
}

fn selected_field_from_aspect(
    value: &ContractValidatedAspectValue,
    field: &FieldKey,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<Option<AspectValue>> {
    prepare(admission, 2, 0)?;
    let value = match value.view() {
        ContractValidatedAspectValueView::Scalar(_) => None,
        ContractValidatedAspectValueView::Struct(value) => {
            prepare(admission, 1, 0)?;
            prepare(
                admission,
                ordered_text_lookup_work(value.len(), field.as_str()),
                0,
            )?;
            value.get(field)
        }
    };
    let Some(value) = value else {
        return Ok(None);
    };
    prepare(
        admission,
        u64::try_from(value.semantic_byte_width()).unwrap_or(u64::MAX),
        u64::try_from(value.owned_allocation_capacity_bytes()).unwrap_or(u64::MAX),
    )?;
    Ok(Some(value.clone()))
}

pub(super) fn observe_mapping(
    candidate: &CheckedMappingCandidate<'_, '_, '_, '_>,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<WorthQueryPrincipalMappingObservation> {
    let entry_work = std::mem::size_of::<EntityId>()
        .checked_add(4)
        .and_then(|work| u64::try_from(work).ok())
        .ok_or(AdmittedPrincipalObservationStop::Admission(
            CompanionPreflightStop::WorkCounterOverflow,
        ))?;
    prepare(admission, entry_work, 0)?;
    let view = candidate.view();
    let mapping = candidate.mapping();
    let layout = candidate.resolution().layout;
    let expected_identity = candidate.resolution().expected_identity;
    let identity_revision = revision(view, mapping, &layout.identity_locator, admission)?;
    let status_revision = revision(view, mapping, &layout.status_locator, admission)?;
    // The immediately preceding Native indexed lookup compared the actual
    // identity at this same issued view. Re-read live kind and status, while
    // carrying that checked value into freshness without a second identity get.
    let (metadata, status) = selected_value(
        view,
        mapping,
        layout.mapping_kind,
        &layout.status_locator,
        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
        admission,
    )?;
    let AspectValue::Bool(enabled) = status else {
        return Err(stale());
    };
    prepare(admission, 2, 0)?;
    let copy_work = expected_identity
        .semantic_byte_width()
        .checked_add(std::mem::size_of::<AspectValue>())
        .and_then(|work| u64::try_from(work).ok())
        .ok_or({
            AdmittedPrincipalObservationStop::Admission(CompanionPreflightStop::WorkCounterOverflow)
        })?;
    let copy_bytes =
        u64::try_from(expected_identity.owned_allocation_capacity_bytes()).map_err(|_| {
            AdmittedPrincipalObservationStop::Admission(
                CompanionPreflightStop::PreparationMemoryCounterOverflow,
            )
        })?;
    prepare(admission, copy_work, copy_bytes)?;
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
    view: &VisibilityProjectionView<'_>,
    mapping: EntityId,
    layout: &WorthQueryPrimaryPrincipalBindingLayout,
    admission: &mut InvalidationEditAdmission,
) -> ObservationResult<WorthQueryPrincipalTargetObservation> {
    let mut selected = None;
    view.try_visit_adjacency_ids(
        mapping,
        layout.relation_kind,
        RelationalAdjacencyDirection::Outgoing,
        |visit| {
            match visit {
                RelationalAdjacencyVisit::Prepare { work, bytes } => {
                    prepare(admission, work, bytes)?;
                }
                RelationalAdjacencyVisit::List => {}
                RelationalAdjacencyVisit::Relation(id) => {
                    let work = view
                        .exact_relation_metadata_read_work_bound()
                        .ok_or_else(stale)?;
                    prepare(admission, work, 0)?;
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
                            WorthQueryPrincipalResolutionDenialKind::AmbiguousPrincipalTarget,
                        ));
                    }
                }
            }
            Ok(())
        },
    )?;
    let relation = selected.ok_or(AdmittedPrincipalObservationStop::Semantic(
        WorthQueryPrincipalResolutionDenialKind::MissingPrincipalTarget,
    ))?;
    let (principal, principal_identity) = selected_value(
        view,
        relation.target,
        layout.principal_kind,
        &layout.principal_identity_locator,
        WorthQueryPrincipalResolutionDenialKind::WrongPrincipalTargetKind,
        admission,
    )?;
    let principal_identity_revision = revision(
        view,
        relation.target,
        &layout.principal_identity_locator,
        admission,
    )?;
    let adjacency_revision = view
        .exact_adjacency_structural_revision_admitted(
            mapping,
            layout.relation_kind,
            RelationalAdjacencyDirection::Outgoing,
            |work| admission.charge_external_work(work),
        )
        .map_err(AdmittedPrincipalObservationStop::Admission)?
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

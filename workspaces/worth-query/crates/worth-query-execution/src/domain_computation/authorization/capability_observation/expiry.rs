use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};

use super::*;

pub(super) fn observe_grant_expiry(
    relational: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    installed: &WorthQueryInstalledCapabilityPlan,
    grant: worth_relational::facade::identity::EntityId,
) -> Result<worth_foundational::facade::AspectValue, WorthQueryOperationAuthorizationDenial> {
    let locator = &installed.request().not_after;
    let field = locator
        .field_path()
        .fields()
        .first()
        .cloned()
        .ok_or_else(|| invalid_policy(installed.contract().name()))?;
    let scope = ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
        locator.aspect().aspect_key().clone(),
        [field.clone()],
    )]);
    relational
        .read_truth()
        .project_snapshot(snapshot)
        .and_then(|view| {
            view.entity_record_with_projection_scope(grant, scope, |record| {
                (record.kind_id() == installed.grant_kind()
                    && record.lifecycle()
                        == worth_relational::facade::storage::RecordLifecycleState::Live)
                    .then(|| {
                        record
                            .aspect_field_value(locator.aspect().aspect_key(), &field)
                            .cloned()
                    })
                    .flatten()
            })
        })
        .ok_or_else(|| {
            WorthQueryOperationAuthorizationDenial::new(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                installed.contract().name(),
            )
        })
}

pub(super) fn add_observation_work(
    work: &mut worth_relational::facade::authorization::RelationalAuthorizationObservationCounters,
) {
    work.entity_records_inspected = work.entity_records_inspected.saturating_add(1);
    work.predicate_fields_inspected = work.predicate_fields_inspected.saturating_add(1);
}

pub(super) fn observation_work(
) -> worth_relational::facade::authorization::RelationalAuthorizationObservationCounters {
    let mut work = Default::default();
    add_observation_work(&mut work);
    work
}

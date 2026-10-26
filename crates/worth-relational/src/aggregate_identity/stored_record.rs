//! Shared row grammar/definition/mint validation, accepting active and retired rows.
use super::funding::{AggregateWork, BuildResult, PlanDenial};
use super::observations::Acquisition;
use super::record::{AggregateRecord, AGGREGATE_ASPECT};
use crate::identity::data::EntityId;
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::validation::engine::state_view::InvariantStateView;
use worth_foundational::facade::{AspectKey, ContractValidatedAspectValueView};

pub(super) fn read(
    view: &InvariantStateView<'_>,
    entity: EntityId,
    contract: AggregateIdentityContractDeclaration,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<AggregateRecord> {
    work.retain(super::retention_extent::stored_row())?;
    let Some(metadata) = view.entity_metadata(entity) else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    work.acquired(Acquisition::Entity)?;
    if metadata.kind_id != contract.storage.aggregate_kind {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    let Some(state) = view.entity_aspect_state(entity) else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    work.acquired(Acquisition::Aspect)?;
    let Some(aspect) =
        state.get(&AspectKey::new(AGGREGATE_ASPECT).expect("fixed aggregate aspect"))
    else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    let ContractValidatedAspectValueView::Struct(fields) = aspect.view() else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    let Some(row) =
        AggregateRecord::decode(|locator| fields.get(locator.field_path().fields().first()?))
    else {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    };
    if row.definition != contract.definition
        || row.identity != super::transition::mint(row.definition, row.born, row.seed)
    {
        return work.invalid(PlanDenial::InvalidStoredIdentity);
    }
    // The scalar decoder retains the row's existing lifecycle posture. Active
    // anchor/membership requirements belong only to read_stored_group.
    Ok(row)
}

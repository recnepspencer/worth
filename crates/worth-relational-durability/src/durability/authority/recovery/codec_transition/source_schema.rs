//! Source-carrier readmission and supported consequence admission before restore.
use crate::durability::data::DurableCheckpoint;
use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::indexes::data::definition::{DerivedIndexDefinition, DerivedIndexKind};
use contracts::schema::codec_field_comparison::CodecFieldLocus;
use contracts::schema::data::{RelationalSchemaRegistry, RelationalSchemaRegistryFieldsPort as _};
use worth_foundational::FoundationalBranchTarget;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::validation::custom_registry::FrozenCustomInvariantRegistry;

pub mod relation_consequences;
pub use relation_consequences::admit_retired_relation_consequences;

pub fn readmit_current_source_schema(
    checkpoint: &DurableCheckpoint,
    main_branch: &contracts::history::data::coordinates::BranchId,
    available: &contracts::content::registry::RelationalContentFormatRegistry,
) -> Result<
    RelationalSchemaRegistry,
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    for cell in checkpoint
        .branch_cells
        .iter()
        .filter(|cell| &cell.branch_id == main_branch)
    {
        let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
            continue;
        };
        let root = checkpoint
            .branch_roots
            .iter()
            .find(|root| root.commit_id.0 == target.selected_commit_id())
            .ok_or_else(|| schema_denial("current source root image is unavailable".into()))?;
        let carrier = checkpoint
            .branch_root_schema_images
            .iter()
            .find(|carrier| carrier.carrier_digest() == root.schema_carrier_digest)
            .ok_or_else(|| schema_denial("current source schema carrier is unavailable".into()))?;
        if carrier.schema_root() != *target.roots().schema_root() {
            return Err(schema_denial(
                "current source schema carrier differs from its exact target".into(),
            )
            .into());
        }
        let (mut registry, _) = carrier.readmit_registry().map_err(schema_denial)?;
        (*registry.owner_content_formats_mut()) =
            (*registry.owner_content_formats()).bind_recorded(available)?;
        return Ok(registry);
    }
    Err(schema_denial("codec transition requires a committed current source schema".into()).into())
}

/// Select the schema role from this image's authenticated current component,
/// never from the union of known schema carriers or an inline-root fallback.
pub fn readmit_interim_current_source_schema(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    origin: &crate::durability::authority::CapturedNativeInterimOrigin,
    checkpoint: &DurableCheckpoint,
    main_branch: &contracts::history::data::coordinates::BranchId,
    available: &contracts::content::registry::RelationalContentFormatRegistry,
) -> Result<
    RelationalSchemaRegistry,
    contracts::durability::data::recovery_stop::RelationalRecoveryStop,
> {
    use contracts::durability::checkpoint_admission::refusal::RelationalCheckpointAdmissionRefusal as Refusal;
    use contracts::durability::data::recovery_stop::RelationalRecoveryStop as Stop;
    let directory = crate::durability::authority::checkpoint_complete::interim::origin_readmission::decoded_origin_directory(
        origin, checkpoint, request,
    )?;
    let manifest = &directory.manifest;
    let mut selected = None;
    for cell in manifest.branch_data.branch_cells.iter() {
        work(request, 1 + cell.branch_id.0.len() as u64)?;
        if &cell.branch_id == main_branch {
            selected = Some(cell);
            break;
        }
    }
    let cell = selected.ok_or(Stop::AdmissionImage(Refusal::MissingRootImage))?;
    let FoundationalBranchTarget::Basis(target) = cell.observation.target() else {
        return Err(schema_denial(
            "codec transition requires a committed current source schema".into(),
        )
        .into());
    };
    let mut selected = None;
    for root in manifest.roots.iter() {
        work(request, 1)?;
        if root.commit_id.0 == target.selected_commit_id() {
            selected = Some(root);
            break;
        }
    }
    let root = selected.ok_or(Stop::AdmissionImage(Refusal::MissingRootImage))?;
    let mut lifecycle = None;
    for life in manifest.branch_data.branch_lifecycle.iter() {
        work(request, 1 + life.branch_id.0.len() as u64)?;
        if &life.branch_id == main_branch {
            lifecycle = Some(life.posture);
            break;
        }
    }
    let posture = match lifecycle.ok_or(Stop::AdmissionImage(Refusal::InvalidProjectedMeaning))? {
        worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Live => {
            worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture::Live
        }
        worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Archived => {
            worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture::Archived
        }
        worth_relational_branch::internal_port::branch::lifecycle::RelationalBranchLifecyclePosture::Deleting => {
            worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisPosture::Deleting
        }
    };
    let mut matched = false;
    for component in &root.recorded_components {
        work(
            request,
            1 + component.branch_id().0.len() as u64 + target.parent_commit_ids().len() as u64,
        )?;
        if component.branch_id() == main_branch
            && component.reference().generation() == cell.observation.generation()
            && component.truth_version() == cell.truth_version
            && component.posture() == posture
            && component.schema_commitment() == *target.roots().schema_root()
            && let FoundationalBranchTarget::Basis(recorded) = component.reference().target()
            && recorded.selected_commit_id() == target.selected_commit_id()
            && recorded.version_id() == target.version_id()
            && recorded.parent_commit_ids() == target.parent_commit_ids()
            && recorded.roots() == target.roots()
        {
            matched = true;
            break;
        }
    }
    if !matched {
        return Err(Stop::AdmissionImage(Refusal::InvalidProjectedMeaning));
    }
    let mut selected = None;
    for carrier in manifest.schemas.iter() {
        work(request, 1)?;
        if carrier.carrier_digest() == root.schema_carrier_digest {
            selected = Some(carrier);
            break;
        }
    }
    let carrier = selected.ok_or(Stop::AdmissionImage(Refusal::InvalidProjectedMeaning))?;
    if carrier.schema_root() != *target.roots().schema_root() {
        return Err(Stop::AdmissionImage(Refusal::InvalidProjectedMeaning));
    }
    // The existing source schema constructor and executable format binding stay
    // identical to original-frame admission; only the authoritative selector differs.
    let (mut registry, _) = carrier.readmit_registry().map_err(schema_denial)?;
    (*registry.owner_content_formats_mut()) =
        (*registry.owner_content_formats()).bind_recorded(available)?;
    Ok(registry)
}
fn work(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    units: u64,
) -> Result<(), contracts::durability::data::recovery_stop::RelationalRecoveryStop> {
    contracts::durability::data::recovery_stop::account_work(request, units)
}

pub fn admit_supported_consequences<'a>(
    registrations: &FrozenCustomInvariantRegistry,
    indexes: impl IntoIterator<Item = &'a DerivedIndexDefinition>,
    changes: &[CodecFieldLocus],
) -> Result<(), String> {
    for invariant in registrations.iter() {
        let access = &invariant.descriptor().operational.access;
        if changes.iter().any(|change| {
            access.read_entity_kinds.contains(&change.kind_id)
                || access.affected_entity_kinds.contains(&change.kind_id)
        }) {
            return Err(
                "codec transition with custom invariant consequences is unsupported".into(),
            );
        }
    }
    for group in registrations.source_inventory().iter() {
        let access = group.access_contract();
        if changes.iter().any(|change| {
            access.read_entity_kinds.contains(&change.kind_id)
                || access.affected_entity_kinds.contains(&change.kind_id)
        }) {
            return Err("codec transition with managed Source consequences is unsupported".into());
        }
    }
    for index in indexes {
        let unsupported = match &index.kind {
            DerivedIndexKind::RelatedEntityOrdering {
                child_kind,
                ordering,
                ..
            } => changes.iter().any(|change| {
                change.kind_id == *child_kind
                    && ordering.iter().any(|field| {
                        let locator = field.locator();
                        locator.aspect().aspect_key() == &change.aspect_key
                            && locator.field_path().fields().first() == Some(&change.field_key)
                    })
            }),
            DerivedIndexKind::RelationJoin(join) => changes.iter().any(|change| {
                change.kind_id == join.shared_entity_kind()
                    || change.kind_id == join.left().external_entity_kind()
                    || change.kind_id == join.right().external_entity_kind()
            }),
            DerivedIndexKind::EntityField { .. } | DerivedIndexKind::RelationField { .. } => false,
        };
        if unsupported {
            return Err(
                "codec transition has an unsupported ordering or join index consequence".into(),
            );
        }
    }
    Ok(())
}

pub fn admit_single_codec_head(checkpoint: &DurableCheckpoint) -> Result<(), String> {
    if checkpoint.branch_cells.len() != 1 {
        return Err("codec transition requires one current branch; other source-contract heads cannot be adopted partially".into());
    }
    Ok(())
}

pub fn schema_denial(detail: String) -> DurabilityError {
    DurabilityError::new(RecoveryFailureClass::SchemaMismatch, detail)
}

#[cfg(test)]
pub mod tests;

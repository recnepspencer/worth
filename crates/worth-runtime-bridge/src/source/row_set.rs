use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectMask, ContractValidatedAspectArtifact,
    ProjectionMask,
};

use crate::relational_source::identity_parts::{
    RelationalBridgeRecordIdentityKind, RelationalBridgeRecordIdentityParts,
};
use crate::snapshot::{
    contract_validated_scalar_aspect_value, BridgeSnapshotReadError,
    MaterializedTruthViewObservation, SnapshotReadRequest, SnapshotReadTarget,
};

mod digest_basis;
mod native_projection_basis;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BridgeRowIdentity(Arc<str>);

impl BridgeRowIdentity {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeMaterializedFieldProjection {
    aspect_locator: AspectLocator,
    field_locator: Option<AspectFieldLocator>,
    projection_mask: AspectMask<ProjectionMask>,
    field_identity: BridgeMaterializedFieldIdentity,
    canonical_basis: Arc<str>,
}

impl BridgeMaterializedFieldProjection {
    fn from_snapshot_target(target: &SnapshotReadTarget) -> Self {
        let locator_basis = native_projection_basis::row_field_projection_locator_canonical_basis(
            target.aspect_locator(),
            target.field_locator().cloned(),
        );
        let mask_basis = native_projection_basis::row_field_projection_mask_canonical_basis(
            target.aspect_locator(),
            target.projection_mask(),
        );
        let field_identity = BridgeMaterializedFieldIdentity::from_native_projection_basis(
            &locator_basis,
            &mask_basis,
        );
        let canonical_basis =
            native_projection_basis::materialized_field_projection_canonical_basis(
                &locator_basis,
                &mask_basis,
                &field_identity,
            );
        Self {
            aspect_locator: target.aspect_locator().clone(),
            field_locator: target.field_locator().cloned(),
            projection_mask: target.projection_mask().clone(),
            field_identity,
            canonical_basis: canonical_basis.into(),
        }
    }

    pub fn aspect_key(&self) -> &AspectKey {
        self.aspect_locator.aspect_key()
    }

    pub fn aspect_locator(&self) -> &AspectLocator {
        &self.aspect_locator
    }

    pub fn field_locator(&self) -> Option<&AspectFieldLocator> {
        self.field_locator.as_ref()
    }

    pub fn projection_mask(&self) -> &AspectMask<ProjectionMask> {
        &self.projection_mask
    }

    pub fn field_identity(&self) -> &BridgeMaterializedFieldIdentity {
        &self.field_identity
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BridgeMaterializedFieldIdentity(Arc<str>);

impl BridgeMaterializedFieldIdentity {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    fn from_native_projection_basis(
        locator_basis: &worth_foundational::facade::CanonicalBasisReadyArtifact,
        mask_basis: &worth_foundational::facade::CanonicalBasisReadyArtifact,
    ) -> Self {
        native_projection_basis::row_field_projection_identity_from_basis(locator_basis, mask_basis)
    }

    fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeMaterializedFieldValue {
    projection: BridgeMaterializedFieldProjection,
    validated_value: ContractValidatedAspectArtifact,
    validated_value_canonical_basis: Arc<str>,
}

impl BridgeMaterializedFieldValue {
    pub fn projection(&self) -> &BridgeMaterializedFieldProjection {
        &self.projection
    }

    pub fn scalar_value(&self) -> Option<&worth_foundational::facade::AspectValue> {
        contract_validated_scalar_aspect_value(&self.validated_value)
    }

    pub fn validated_value(&self) -> &ContractValidatedAspectArtifact {
        &self.validated_value
    }

    pub fn validated_value_canonical_basis(&self) -> &str {
        self.validated_value_canonical_basis.as_ref()
    }

    fn from_validated_record(
        projection: BridgeMaterializedFieldProjection,
        record: &crate::snapshot::ValidatedSnapshotReadRecord,
    ) -> Option<Self> {
        let validated_value = record.validated_value_posture()?.clone();
        let validated_value_canonical_basis =
            crate::snapshot::validated_value_basis::validated_snapshot_read_value_canonical_basis(
                &validated_value,
            );
        Some(Self {
            projection,
            validated_value,
            validated_value_canonical_basis: validated_value_canonical_basis.into(),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeMaterializedRowArtifact {
    row_identity: BridgeRowIdentity,
    fields: BTreeMap<BridgeMaterializedFieldIdentity, BridgeMaterializedFieldValue>,
}

impl BridgeMaterializedRowArtifact {
    pub fn row_identity(&self) -> &BridgeRowIdentity {
        &self.row_identity
    }

    pub fn fields(
        &self,
    ) -> &BTreeMap<BridgeMaterializedFieldIdentity, BridgeMaterializedFieldValue> {
        &self.fields
    }

    pub(crate) fn whole_aspect_fields_for_key<'a>(
        &'a self,
        aspect_key: &'a AspectKey,
    ) -> impl Iterator<Item = &'a BridgeMaterializedFieldValue> + 'a {
        self.fields().values().filter(move |field| {
            field.projection().aspect_key() == aspect_key
                && field.projection().field_locator().is_none()
                && field.projection().projection_mask().is_whole_aspect()
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeMaterializedRowSetDigest(Arc<str>);

impl BridgeMaterializedRowSetDigest {
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BridgeMaterializedRowSetArtifact {
    truth_view_digest: Arc<str>,
    basis_snapshot_identity: crate::snapshot::TruthSnapshotIdentity,
    rows: Vec<BridgeMaterializedRowArtifact>,
    digest: BridgeMaterializedRowSetDigest,
}

impl BridgeMaterializedRowSetArtifact {
    pub fn truth_view_digest(&self) -> &str {
        self.truth_view_digest.as_ref()
    }

    pub fn basis_snapshot_identity(&self) -> &crate::snapshot::TruthSnapshotIdentity {
        &self.basis_snapshot_identity
    }

    pub fn rows(&self) -> &[BridgeMaterializedRowArtifact] {
        &self.rows
    }

    pub fn digest(&self) -> &BridgeMaterializedRowSetDigest {
        &self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeRowSetMaterializationError {
    SnapshotReadContractFailure {
        error: BridgeSnapshotReadError,
    },
    DuplicateMaterializedField {
        row_identity: String,
        field_identity: String,
    },
}

pub fn materialize_bridge_row_set(
    observation: &MaterializedTruthViewObservation,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeMaterializedRowSetArtifact, BridgeRowSetMaterializationError> {
    let result = observation
        .read_planned_packet(execution)
        .map_err(|error| BridgeRowSetMaterializationError::SnapshotReadContractFailure { error })?;
    let mut rows: BTreeMap<
        Arc<str>,
        BTreeMap<BridgeMaterializedFieldIdentity, BridgeMaterializedFieldValue>,
    > = BTreeMap::new();

    for (read, record) in observation
        .read_packet()
        .reads()
        .iter()
        .zip(result.records().iter())
    {
        let row_identity = Arc::from(row_identity_for_read(read));
        let field_projection =
            BridgeMaterializedFieldProjection::from_snapshot_target(read.target());
        let Some(field_value) =
            BridgeMaterializedFieldValue::from_validated_record(field_projection, record)
        else {
            continue;
        };
        let field_identity = field_value.projection().field_identity().clone();
        let row_fields = rows.entry(Arc::clone(&row_identity)).or_default();
        if row_fields
            .insert(field_identity.clone(), field_value)
            .is_some()
        {
            return Err(
                BridgeRowSetMaterializationError::DuplicateMaterializedField {
                    row_identity: row_identity.to_string(),
                    field_identity: field_identity.as_str().to_string(),
                },
            );
        }
    }

    let rows = rows
        .into_iter()
        .map(|(row_identity, fields)| BridgeMaterializedRowArtifact {
            row_identity: BridgeRowIdentity::new(row_identity),
            fields,
        })
        .collect::<Vec<_>>();

    Ok(BridgeMaterializedRowSetArtifact {
        truth_view_digest: Arc::from(observation.planned().digest().to_string()),
        basis_snapshot_identity: result.snapshot_identity().clone(),
        digest: digest_basis::row_set_digest_from_materialized_rows(
            observation.planned().digest(),
            result.snapshot_identity(),
            &rows,
        ),
        rows,
    })
}

fn row_identity_for_read(read: &SnapshotReadRequest) -> String {
    read.relational_record_identity_parts()
        .map(relational_row_identity_label)
        .unwrap_or_else(|| read.entity_identity().to_string())
}

fn relational_row_identity_label(parts: RelationalBridgeRecordIdentityParts) -> String {
    let kind = match parts.kind() {
        RelationalBridgeRecordIdentityKind::Entity => "entity",
        RelationalBridgeRecordIdentityKind::Relation => "relation",
    };
    format!(
        "{kind}:{}:{}:{}",
        parts.partition_id(),
        parts.local_slot(),
        parts.generation()
    )
}

#[cfg(test)]
#[path = "row_set_tests.rs"]
mod tests;

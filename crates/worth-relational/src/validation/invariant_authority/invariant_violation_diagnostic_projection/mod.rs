use worth_foundational::facade::{
    prepare_aspect_mask_for_canonical_basis, AspectKey, AspectMask, AspectMaskLocator, AspectValue,
    CanonicalBasisReadyArtifact, CanonicalFieldPath, CanonicalizationRuleVersion, DiagnosticMask,
    FieldKey, LocatorAuthority,
};
use worth_proof::TransitionOutcome;

use crate::config::data::CascadeDeletePolicy;
use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::schema::data::{
    ContractId, EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope,
};
use crate::transactions::data::EntityReference;
use crate::validation::data::{
    CustomInvariantFailureIdentity, CustomInvariantFailurePhase, RelationCardinalityBoundary,
    RelationEndpointBoundary, ResultCustomInvariantFailureKind, StorageInconsistencyFailure,
    StorageInconsistencyLookup, StorageInconsistencyScan,
};

mod from_invariant_violation_fields;
mod typed_projection;

#[derive(Debug)]
pub(super) enum InvariantViolationDiagnosticProjection<'a> {
    None,
    MergedIntentLimit {
        merged_intent_count: usize,
        limit: usize,
    },
    SnapshotEntityLimit {
        version_id: VersionId,
        visible_entities: usize,
        limit: usize,
    },
    UniqueEntityField {
        aspect_field: AspectFieldDiagnosticProjection<'a>,
    },
    SidecarConsistency {
        partition_id: PartitionId,
        slot: usize,
        missing_label: &'a str,
    },
    RelationEndpointKindMismatch {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        source_kind_id: KindId,
        target_kind_id: KindId,
        boundary: RelationEndpointBoundary,
    },
    RelationEndpointKindSelfEdge {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        self_edge: bool,
    },
    RelationEndpointKindCrossContext {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source_partition_id: PartitionId,
        target_partition_id: PartitionId,
    },
    RelationCardinalityEndpoint {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        entity_id: &'a EntityReference,
        boundary: RelationCardinalityBoundary,
        count: usize,
        limit: u64,
    },
    RelationCardinalityPair {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        count: usize,
        limit: u64,
    },
    RelationUniqueness {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        scope: UniquenessScope,
        source: &'a EntityReference,
        target: &'a EntityReference,
        count: usize,
    },
    RelationSymmetry {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        mode: SymmetryMode,
    },
    RelationEndpointDeletionIntegrity {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        entity_id: EntityId,
        remaining_relation_endpoint_count: usize,
        mode: EndpointDeletionIntegrityMode,
        cascade_delete_policy: Option<CascadeDeletePolicy>,
    },
    StorageInconsistency {
        entity_id: Option<EntityId>,
        partition_id: Option<PartitionId>,
        slot: Option<usize>,
        field: Option<&'a FieldKey>,
        missing_label: Option<&'a str>,
        scan: Option<StorageInconsistencyScan>,
        lookup: Option<StorageInconsistencyLookup>,
        failure: Option<StorageInconsistencyFailure>,
    },
    RelationIntegrityScopeBudgetExceeded {
        limit_name: &'a str,
        limit: usize,
        observed: usize,
        relation_kind_count: usize,
        touched_entity_count: usize,
        deleted_entity_count: usize,
        scanned_relation_count: usize,
        planned_edge_count: usize,
    },
    CustomInvariantFailure {
        identity: &'a CustomInvariantFailureIdentity,
        phase: CustomInvariantFailurePhase,
        failure: ResultCustomInvariantFailureKind,
        detail: &'a str,
    },
    CustomInvariantViolation {
        identity: &'a crate::validation::data::CustomInvariantSemanticIdentity,
    },
    PartitionIsolation {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        relation_id: Option<RelationId>,
        source_partition_id: PartitionId,
        target_partition_id: PartitionId,
    },
    Acyclicity {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
    },
    ConnectivityMinimum {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        reachable_target_count: usize,
        minimum_reachable_targets: u32,
    },
}

#[derive(Debug)]
pub(super) struct AspectFieldDiagnosticProjection<'a> {
    aspect_key: &'a AspectKey,
    field_path: CanonicalFieldPath,
    diagnostic_mask: AspectMask<DiagnosticMask>,
    diagnostic_mask_locator: AspectMaskLocator<DiagnosticMask>,
    canonical_diagnostic_mask_basis: CanonicalBasisReadyArtifact,
    value: &'a AspectValue,
}

impl<'a> AspectFieldDiagnosticProjection<'a> {
    fn new(
        aspect_key: &'a AspectKey,
        field_path: CanonicalFieldPath,
        value: &'a AspectValue,
    ) -> Self {
        let diagnostic_mask = AspectMask::<DiagnosticMask>::new([field_path.clone()]);
        let diagnostic_mask_locator = AspectMaskLocator::diagnostic(
            LocatorAuthority::SupportOnly,
            aspect_key.clone(),
            &diagnostic_mask,
        );
        let canonical_diagnostic_mask_basis =
            canonical_diagnostic_mask_basis(aspect_key, &diagnostic_mask);

        Self {
            aspect_key,
            field_path,
            diagnostic_mask,
            diagnostic_mask_locator,
            canonical_diagnostic_mask_basis,
            value,
        }
    }

    fn to_diagnostic_value(&self) -> RelationalDiagnosticValue {
        RelationalDiagnosticValue::object([
            (
                "aspect_key",
                RelationalDiagnosticValue::AspectKey(self.aspect_key.clone()),
            ),
            (
                "field_path",
                RelationalDiagnosticValue::FieldPath(self.field_path.clone()),
            ),
            (
                "diagnostic_mask",
                RelationalDiagnosticValue::DiagnosticMask(self.diagnostic_mask.clone()),
            ),
            (
                "diagnostic_mask_locator",
                RelationalDiagnosticValue::DiagnosticMaskLocator(
                    self.diagnostic_mask_locator.clone(),
                ),
            ),
            (
                "canonical_diagnostic_mask_basis",
                RelationalDiagnosticValue::CanonicalBasis(
                    self.canonical_diagnostic_mask_basis.clone(),
                ),
            ),
            (
                "value",
                RelationalDiagnosticValue::AspectValue(self.value.clone()),
            ),
        ])
    }
}

fn canonical_diagnostic_mask_basis(
    aspect_key: &AspectKey,
    diagnostic_mask: &AspectMask<DiagnosticMask>,
) -> CanonicalBasisReadyArtifact {
    let version = CanonicalizationRuleVersion::new("worth.relational.invariant.diagnostic_mask.v1")
        .expect("diagnostic mask canonicalization version is static and non-empty");
    match prepare_aspect_mask_for_canonical_basis(
        version,
        aspect_key.clone(),
        diagnostic_mask.clone(),
    ) {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            panic!("invariant diagnostic mask basis denied for {aspect_key:?}: {denial:?}")
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => {
            panic!("invariant diagnostic mask basis did not become ready for {aspect_key:?}")
        }
    }
}

fn violation_diagnostic_object(
    violation_kind: &'static str,
    fields: impl IntoIterator<Item = (impl Into<String>, RelationalDiagnosticValue)>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object(
        [(
            "violation_kind".to_string(),
            RelationalDiagnosticValue::string(violation_kind),
        )]
        .into_iter()
        .chain(fields.into_iter().map(|(key, value)| (key.into(), value))),
    )
}

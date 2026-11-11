use serde::{Deserialize, Serialize};

use crate::history::data::BranchId;
use crate::schema::data::{SchemaId, SchemaVersionId};

use super::{
    default_boundary_visibility_for_continuation, DescriptorCanonicalBasisVersion,
    DescriptorSemanticsVersion, HistoricalInterpretationSensitivity, SchemaBoundaryFingerprint,
    SchemaBridgeabilityClassification, SchemaContinuationAdmissionObservation,
    SchemaContinuationClassification, SchemaDiffAtom, SchemaLineageOrderingSemantics,
    SchemaReconciliationClassification, SchemaReconciliationOrderingMode,
    SchemaReconciliationPolicy, SchemaStratum, SubscriberBoundaryVisibility,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedSchemaTransition {
    pub source_schema_id: SchemaId,
    pub source_schema_version_id: SchemaVersionId,
    pub target_schema_id: SchemaId,
    pub target_schema_version_id: SchemaVersionId,
    pub diff_atoms: Vec<SchemaDiffAtom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedSchemaTransition {
    pub proposed: ProposedSchemaTransition,
    pub continuation_admission_observation: SchemaContinuationAdmissionObservation,
    pub reconciliation: SchemaReconciliationClassification,
    pub continuation: SchemaContinuationClassification,
    pub bridgeability: SchemaBridgeabilityClassification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaBridgeDescriptor {
    pub boundary_fingerprint: SchemaBoundaryFingerprint,
    pub semantics_version: DescriptorSemanticsVersion,
    pub canonical_basis_version: DescriptorCanonicalBasisVersion,
    pub continuation: SchemaContinuationClassification,
    pub bridgeability: SchemaBridgeabilityClassification,
    pub boundary_visibility: SubscriberBoundaryVisibility,
    pub historical_interpretation: HistoricalInterpretationSensitivity,
    pub changed_strata: Vec<SchemaStratum>,
}

impl SchemaBridgeDescriptor {
    pub fn new(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        semantics_version: DescriptorSemanticsVersion,
        canonical_basis_version: DescriptorCanonicalBasisVersion,
        continuation: SchemaContinuationClassification,
        bridgeability: SchemaBridgeabilityClassification,
        historical_interpretation: HistoricalInterpretationSensitivity,
        changed_strata: Vec<SchemaStratum>,
    ) -> Self {
        Self::new_with_visibility(
            boundary_fingerprint,
            semantics_version,
            canonical_basis_version,
            continuation,
            bridgeability,
            default_boundary_visibility_for_continuation(continuation),
            historical_interpretation,
            changed_strata,
        )
    }

    pub fn new_with_visibility(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        semantics_version: DescriptorSemanticsVersion,
        canonical_basis_version: DescriptorCanonicalBasisVersion,
        continuation: SchemaContinuationClassification,
        bridgeability: SchemaBridgeabilityClassification,
        boundary_visibility: SubscriberBoundaryVisibility,
        historical_interpretation: HistoricalInterpretationSensitivity,
        changed_strata: Vec<SchemaStratum>,
    ) -> Self {
        Self {
            boundary_fingerprint,
            semantics_version,
            canonical_basis_version,
            continuation,
            bridgeability,
            boundary_visibility,
            historical_interpretation,
            changed_strata,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaContinuationDescriptor {
    pub boundary_fingerprint: SchemaBoundaryFingerprint,
    pub bridge: SchemaBridgeDescriptor,
    pub normalized_boundary_count: usize,
}

impl SchemaContinuationDescriptor {
    pub fn new(
        boundary_fingerprint: SchemaBoundaryFingerprint,
        bridge: SchemaBridgeDescriptor,
        normalized_boundary_count: usize,
    ) -> Self {
        Self {
            boundary_fingerprint,
            bridge,
            normalized_boundary_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaLineageArtifact {
    pub resulting_schema_id: SchemaId,
    pub resulting_schema_version_id: SchemaVersionId,
    pub parent_schema_ids: Vec<SchemaId>,
    pub parent_schema_version_ids: Vec<SchemaVersionId>,
    pub branch_context: Option<BranchId>,
    pub ordering_mode: SchemaReconciliationOrderingMode,
    pub ordering_semantics: SchemaLineageOrderingSemantics,
}

impl SchemaLineageArtifact {
    pub fn new(
        resulting_schema_id: SchemaId,
        resulting_schema_version_id: SchemaVersionId,
        parent_schema_ids: Vec<SchemaId>,
        parent_schema_version_ids: Vec<SchemaVersionId>,
        branch_context: Option<BranchId>,
        ordering_mode: SchemaReconciliationOrderingMode,
        ordering_semantics: SchemaLineageOrderingSemantics,
    ) -> Self {
        Self {
            resulting_schema_id,
            resulting_schema_version_id,
            parent_schema_ids,
            parent_schema_version_ids,
            branch_context,
            ordering_mode,
            ordering_semantics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaReconciliationDescriptor {
    pub semantics_version: DescriptorSemanticsVersion,
    pub canonical_basis_version: DescriptorCanonicalBasisVersion,
    pub classification: SchemaReconciliationClassification,
    pub policy: SchemaReconciliationPolicy,
    pub resulting_lineage: SchemaLineageArtifact,
}

impl SchemaReconciliationDescriptor {
    pub fn new(
        semantics_version: DescriptorSemanticsVersion,
        canonical_basis_version: DescriptorCanonicalBasisVersion,
        classification: SchemaReconciliationClassification,
        policy: SchemaReconciliationPolicy,
        resulting_lineage: SchemaLineageArtifact,
    ) -> Self {
        Self {
            semantics_version,
            canonical_basis_version,
            classification,
            policy,
            resulting_lineage,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoweredSchemaTransitionPlan {
    pub validated: ValidatedSchemaTransition,
    pub continuation_descriptor: SchemaContinuationDescriptor,
    pub reconciliation_descriptor: SchemaReconciliationDescriptor,
}

impl LoweredSchemaTransitionPlan {
    pub fn new(
        validated: ValidatedSchemaTransition,
        continuation_descriptor: SchemaContinuationDescriptor,
        reconciliation_descriptor: SchemaReconciliationDescriptor,
    ) -> Self {
        Self {
            validated,
            continuation_descriptor,
            reconciliation_descriptor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaTransitionArtifact {
    pub source_schema_id: SchemaId,
    pub source_schema_version_id: SchemaVersionId,
    pub target_schema_id: SchemaId,
    pub target_schema_version_id: SchemaVersionId,
    pub diff_atoms: Vec<SchemaDiffAtom>,
    pub continuation_descriptor: SchemaContinuationDescriptor,
    pub reconciliation_descriptor: SchemaReconciliationDescriptor,
}

impl SchemaTransitionArtifact {
    pub fn new(
        source_schema_id: SchemaId,
        source_schema_version_id: SchemaVersionId,
        target_schema_id: SchemaId,
        target_schema_version_id: SchemaVersionId,
        diff_atoms: Vec<SchemaDiffAtom>,
        continuation_descriptor: SchemaContinuationDescriptor,
        reconciliation_descriptor: SchemaReconciliationDescriptor,
    ) -> Self {
        Self {
            source_schema_id,
            source_schema_version_id,
            target_schema_id,
            target_schema_version_id,
            diff_atoms,
            continuation_descriptor,
            reconciliation_descriptor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaTransitionSummary {
    pub changed_atom_count: usize,
    pub changed_strata: Vec<SchemaStratum>,
    pub continuation: SchemaContinuationClassification,
    pub bridgeability: SchemaBridgeabilityClassification,
    pub reconciliation: SchemaReconciliationClassification,
    pub historical_interpretation: HistoricalInterpretationSensitivity,
}

impl SchemaTransitionSummary {
    pub fn from_artifact(artifact: &SchemaTransitionArtifact) -> Self {
        let changed_strata = observed_strata(artifact).collect();
        Self {
            changed_atom_count: artifact.diff_atoms.len(),
            changed_strata,
            continuation: artifact.continuation_descriptor.bridge.continuation,
            bridgeability: artifact.continuation_descriptor.bridge.bridgeability,
            reconciliation: artifact.reconciliation_descriptor.classification,
            historical_interpretation: artifact
                .continuation_descriptor
                .bridge
                .historical_interpretation,
        }
    }
}

impl SchemaTransitionSummary {
    /// Compare the SAME summary meaning without constructing a second vector.
    pub(crate) fn matches_artifact(&self, artifact: &SchemaTransitionArtifact) -> bool {
        self.changed_atom_count == artifact.diff_atoms.len()
            && self
                .changed_strata
                .iter()
                .copied()
                .eq(observed_strata(artifact))
            && self.continuation == artifact.continuation_descriptor.bridge.continuation
            && self.bridgeability == artifact.continuation_descriptor.bridge.bridgeability
            && self.reconciliation == artifact.reconciliation_descriptor.classification
            && self.historical_interpretation
                == artifact
                    .continuation_descriptor
                    .bridge
                    .historical_interpretation
    }
}
fn observed_strata(artifact: &SchemaTransitionArtifact) -> impl Iterator<Item = SchemaStratum> {
    use SchemaStratum::*;
    let mut present = [false; 8];
    for stratum in artifact.diff_atoms.iter().flat_map(|atom| &atom.strata) {
        let index = match stratum {
            StructuralShape => 0,
            ValueDomain => 1,
            EntityIdentitySemantics => 2,
            CorrespondenceSemantics => 3,
            LineageSemantics => 4,
            BehavioralSemantics => 5,
            PublicationContract => 6,
            SubscriberContract => 7,
        };
        present[index] = true;
    }
    // This is the enum's original Ord order, exactly the prior BTreeSet order.
    [
        StructuralShape,
        ValueDomain,
        EntityIdentitySemantics,
        CorrespondenceSemantics,
        LineageSemantics,
        BehavioralSemantics,
        PublicationContract,
        SubscriberContract,
    ]
    .into_iter()
    .zip(present)
    .filter_map(|(stratum, present)| present.then_some(stratum))
}

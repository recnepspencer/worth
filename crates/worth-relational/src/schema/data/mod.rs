mod aspect_contracts;
mod aspect_traces;
mod authority_snapshot;
mod continuity;
mod registry_errors;
mod relation_integrity;
mod structural_invariants;

use std::collections::BTreeMap;

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::identity::data::KindId;
use crate::merge::data::{AspectMergePolicyDeclaration, IdentityBasisDeclaration};
use serde::{Deserialize, Serialize};

pub use aspect_contracts::{
    AspectBinding, AspectContractPlanCatalog, AspectContractPlanRevision,
    DeclaredAspectContractBinding, KindAspectContractDeclarations, LoweredAspectContractBinding,
    LoweredAspectContractPlan, RelationalAspectChangeKind,
};
pub use aspect_traces::{
    AspectDeclarationTrace, AspectDeclarationTraceRow, AspectLoweringTrace, AspectLoweringTraceRow,
};
pub use authority_snapshot::{
    schema_authority_snapshot_digest_bytes, SchemaAuthorityKindSnapshot,
    SchemaAuthorityRelationSnapshot, SchemaAuthoritySnapshot,
};
pub use continuity::{
    runtime_descriptor_canonical_basis_policy, runtime_descriptor_semantics_policy,
    DescriptorCanonicalBasisSupportPolicy, DescriptorCanonicalBasisVersion,
    DescriptorSemanticsSupportPolicy, DescriptorSemanticsVersion, FreeFormSchemaDiffIntent,
    HistoricalInterpretationSensitivity, LoweredSchemaTransitionPlan, ProposedSchemaTransition,
    SchemaBoundaryFingerprint, SchemaBridgeDescriptor, SchemaBridgeabilityClassification,
    SchemaContinuationAdmissionObservation, SchemaContinuationClassification,
    SchemaContinuationDescriptor, SchemaDiffAtom, SchemaDiffDetail, SchemaElementKind,
    SchemaElementRef, SchemaLineageArtifact, SchemaLineageOrderingSemantics,
    SchemaPublicationImpact, SchemaReconciliationClassification, SchemaReconciliationDescriptor,
    SchemaReconciliationOrderingMode, SchemaReconciliationPolicy, SchemaStratum,
    SchemaSubscriberImpact, SchemaTransitionArtifact, SchemaTransitionBarrier,
    SchemaTransitionSummary, SubscriberBoundaryVisibility, ValidatedSchemaTransition,
};
pub use registry_errors::{SchemaRegistryError, SchemaRegistryErrorClass};
pub(crate) use relation_integrity::derive_relation_integrity_plan_revision;
pub use relation_integrity::{
    AggregateIdentityContractDeclaration, AggregateIdentityStorage, CardinalityContractDeclaration,
    ContractId, EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
    EndpointKindContractDeclaration, LoweredCardinalityMaximumContract,
    LoweredCardinalityMinimumContract, LoweredEndpointDeletionIntegrityContract,
    LoweredEndpointKindContract, LoweredRelationIntegrityPlan, LoweredSymmetryContract,
    LoweredUniquenessContract, MinimumCardinalityEnforcement, PairMinimumSemantics,
    RelationIntegrityDeclarations, RelationIntegrityPlanCatalog, RelationIntegrityPlanRevision,
    SymmetryContractDeclaration, SymmetryMode, UniquenessContractDeclaration, UniquenessScope,
};
pub use structural_invariants::{
    AcyclicityContractDeclaration, AllowedCycleClass, ConnectivityMinimumContractDeclaration,
    ConnectivityMinimumEnforcement, DirectedTraversalKind, LoweredAcyclicityContract,
    LoweredConnectivityMinimumContract, LoweredPartitionIsolationContract,
    PartitionIsolationContractDeclaration, PartitionIsolationMode,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersionId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityKindRegistration {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub aspect_contract_declarations: KindAspectContractDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationKindRegistration {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub cross_context_policy: CrossContextPolicy,
    pub cascade_delete_policy: CascadeDeletePolicy,
    pub aspect_contract_declarations: KindAspectContractDeclarations,
    pub relation_integrity: RelationIntegrityDeclarations,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindResolution {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RelationalSchemaRegistry {
    pub(crate) entity_kinds: BTreeMap<KindId, EntityKindRegistration>,
    pub(crate) relation_kinds: BTreeMap<KindId, RelationKindRegistration>,
}

impl RelationalSchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_entity_kind(
        mut self,
        registration: EntityKindRegistration,
    ) -> Result<Self, SchemaRegistryError> {
        if self.entity_kinds.contains_key(&registration.kind_id) {
            return Err(SchemaRegistryError::duplicate_entity_kind(
                registration.kind_id,
            ));
        }
        if self.relation_kinds.contains_key(&registration.kind_id) {
            return Err(SchemaRegistryError::entity_relation_kind_collision(
                registration.kind_id,
            ));
        }
        let registration = crate::schema::canonicalize_entity_registration(registration)?;
        self.entity_kinds.insert(registration.kind_id, registration);
        Ok(self)
    }

    pub fn register_relation_kind(
        mut self,
        registration: RelationKindRegistration,
    ) -> Result<Self, SchemaRegistryError> {
        if self.relation_kinds.contains_key(&registration.kind_id) {
            return Err(SchemaRegistryError::duplicate_relation_kind(
                registration.kind_id,
            ));
        }
        if self.entity_kinds.contains_key(&registration.kind_id) {
            return Err(SchemaRegistryError::entity_relation_kind_collision(
                registration.kind_id,
            ));
        }
        let registration = crate::schema::canonicalize_relation_registration(registration)?;
        crate::schema::aggregate_identity::validate_binding(&self, &registration)?;
        self.relation_kinds
            .insert(registration.kind_id, registration);
        Ok(self)
    }

    /// Extends this registry without allowing an existing kind authority to be
    /// replaced by the incoming registry.
    pub fn extend(
        mut self,
        additions: RelationalSchemaRegistry,
    ) -> Result<Self, SchemaRegistryError> {
        for registration in additions.entity_kinds.into_values() {
            self = self.register_entity_kind(registration)?;
        }
        for registration in additions.relation_kinds.into_values() {
            self = self.register_relation_kind(registration)?;
        }
        Ok(self)
    }

    pub fn resolve_entity(&self, kind_id: KindId) -> Result<KindResolution, SchemaRegistryError> {
        self.entity_kinds
            .get(&kind_id)
            .map(|registration| KindResolution {
                kind_id: registration.kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
            })
            .ok_or_else(|| SchemaRegistryError::unknown_entity_kind(kind_id))
    }

    pub fn resolve_relation(&self, kind_id: KindId) -> Result<KindResolution, SchemaRegistryError> {
        self.relation_kinds
            .get(&kind_id)
            .map(|registration| KindResolution {
                kind_id: registration.kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
            })
            .ok_or_else(|| SchemaRegistryError::unknown_relation_kind(kind_id))
    }

    pub fn relation_registration(
        &self,
        kind_id: KindId,
    ) -> Result<&RelationKindRegistration, SchemaRegistryError> {
        self.relation_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_relation_kind(kind_id))
    }

    pub fn entity_registration(
        &self,
        kind_id: KindId,
    ) -> Result<&EntityKindRegistration, SchemaRegistryError> {
        self.entity_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_entity_kind(kind_id))
    }

    pub fn entity_aspect_declaration_trace(
        &self,
        kind_id: KindId,
    ) -> Result<AspectDeclarationTrace, SchemaRegistryError> {
        let registration = self
            .entity_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_entity_kind(kind_id))?;
        Ok(registration
            .aspect_contract_declarations
            .declaration_trace(kind_id))
    }

    pub fn relation_aspect_declaration_trace(
        &self,
        kind_id: KindId,
    ) -> Result<AspectDeclarationTrace, SchemaRegistryError> {
        let registration = self
            .relation_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_relation_kind(kind_id))?;
        Ok(registration
            .aspect_contract_declarations
            .declaration_trace(kind_id))
    }

    pub fn entity_identity_declarations(
        &self,
        kind_id: KindId,
    ) -> Result<&[IdentityBasisDeclaration], SchemaRegistryError> {
        let registration = self
            .entity_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_entity_kind(kind_id))?;
        Ok(&registration
            .aspect_contract_declarations
            .identity_declarations)
    }

    pub fn relation_identity_declarations(
        &self,
        kind_id: KindId,
    ) -> Result<&[IdentityBasisDeclaration], SchemaRegistryError> {
        let registration = self
            .relation_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_relation_kind(kind_id))?;
        Ok(&registration
            .aspect_contract_declarations
            .identity_declarations)
    }

    pub fn entity_merge_policy_declarations(
        &self,
        kind_id: KindId,
    ) -> Result<&[AspectMergePolicyDeclaration], SchemaRegistryError> {
        let registration = self
            .entity_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_entity_kind(kind_id))?;
        Ok(&registration
            .aspect_contract_declarations
            .merge_policy_declarations)
    }

    pub fn relation_merge_policy_declarations(
        &self,
        kind_id: KindId,
    ) -> Result<&[AspectMergePolicyDeclaration], SchemaRegistryError> {
        let registration = self
            .relation_kinds
            .get(&kind_id)
            .ok_or_else(|| SchemaRegistryError::unknown_relation_kind(kind_id))?;
        Ok(&registration
            .aspect_contract_declarations
            .merge_policy_declarations)
    }

    pub fn authoritative_schema_basis(
        &self,
    ) -> Result<Option<(SchemaId, SchemaVersionId)>, SchemaRegistryError> {
        let mut registrations = self
            .entity_kinds
            .values()
            .map(|registration| (&registration.schema_id, registration.schema_version_id))
            .chain(
                self.relation_kinds
                    .values()
                    .map(|registration| (&registration.schema_id, registration.schema_version_id)),
            );
        let Some((schema_id, schema_version_id)) = registrations.next() else {
            return Ok(None);
        };
        for (next_schema_id, next_schema_version_id) in registrations {
            if next_schema_id != schema_id || next_schema_version_id != schema_version_id {
                return Err(SchemaRegistryError::inconsistent_schema_basis(format!(
                    "found mixed schema basis {:?}/{:?} and {:?}/{:?}",
                    schema_id, schema_version_id, next_schema_id, next_schema_version_id
                )));
            }
        }
        Ok(Some((schema_id.clone(), schema_version_id)))
    }

    pub fn authority_snapshot(&self) -> SchemaAuthoritySnapshot {
        let (primary_schema_id, primary_schema_version_id) = self
            .authoritative_schema_basis()
            .ok()
            .flatten()
            .map(|(schema_id, schema_version_id)| (Some(schema_id), Some(schema_version_id)))
            .unwrap_or((None, None));

        let entity_kinds = self
            .entity_kinds
            .iter()
            .map(|(kind_id, registration)| SchemaAuthorityKindSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
            })
            .collect();
        let relation_kinds = self
            .relation_kinds
            .iter()
            .map(|(kind_id, registration)| SchemaAuthorityRelationSnapshot {
                kind_id: *kind_id,
                kind_name: registration.kind_name.clone(),
                schema_id: registration.schema_id.clone(),
                schema_version_id: registration.schema_version_id,
                aspect_plan_revision: registration.aspect_contract_declarations.plan_revision,
                relation_integrity_plan_revision: derive_relation_integrity_plan_revision(
                    &registration.relation_integrity.endpoint_kind_contracts,
                    &registration.relation_integrity.cardinality_contracts,
                    &registration.relation_integrity.uniqueness_contracts,
                    &registration.relation_integrity.symmetry_contracts,
                    &registration
                        .relation_integrity
                        .endpoint_deletion_integrity_contracts,
                    &registration.relation_integrity.acyclicity_contracts,
                    &registration
                        .relation_integrity
                        .partition_isolation_contracts,
                    &registration
                        .relation_integrity
                        .connectivity_minimum_contracts,
                    registration.relation_integrity.aggregate_identity.as_ref(),
                ),
                cross_context_policy: registration.cross_context_policy,
                cascade_delete_policy: registration.cascade_delete_policy,
            })
            .collect();

        SchemaAuthoritySnapshot {
            primary_schema_id,
            primary_schema_version_id,
            entity_kinds,
            relation_kinds,
        }
    }

    pub fn authority_digest_bytes(&self) -> [u8; 32] {
        schema_authority_snapshot_digest_bytes(&self.authority_snapshot())
    }
}

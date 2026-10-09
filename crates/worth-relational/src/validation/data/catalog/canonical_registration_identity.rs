use sha2::{Digest, Sha256};

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::schema::data::{
    AllowedCycleClass, ConnectivityMinimumEnforcement, DirectedTraversalKind,
    EndpointDeletionIntegrityMode, LoweredAcyclicityContract, LoweredCardinalityMaximumContract,
    LoweredCardinalityMinimumContract, LoweredConnectivityMinimumContract,
    LoweredEndpointDeletionIntegrityContract, LoweredEndpointKindContract,
    LoweredPartitionIsolationContract, LoweredSymmetryContract, LoweredUniquenessContract,
    MinimumCardinalityEnforcement, PairMinimumSemantics, PartitionIsolationMode, SymmetryMode,
    UniquenessScope,
};
use crate::validation::data::{
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantRegistration,
    InvariantRule, InvariantRuleDescriptor, InvariantRuleId, InvariantSemanticsClass,
    NativeInvariantRuleId, RecordKindTag, SupportedExecutionPoints,
};

use super::canonical_registration_tags as tags;

pub(super) fn canonical_registration_bytes(registration: &InvariantRegistration) -> Vec<u8> {
    let mut bytes = RegistrationIdentityBytes::new();
    bytes.registration(registration);
    bytes.finish()
}

pub(super) fn canonical_catalog_registration_digest_hex(
    registrations: &[InvariantRegistration],
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"validation-invariant-catalog-v1");
    for registration in registrations {
        let registration_bytes = canonical_registration_bytes(registration);
        hasher.update((registration_bytes.len() as u32).to_le_bytes());
        hasher.update(registration_bytes);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct RegistrationIdentityBytes {
    bytes: Vec<u8>,
}

impl RegistrationIdentityBytes {
    fn new() -> Self {
        let mut bytes = Self { bytes: Vec::new() };
        bytes.string("validation-invariant-registration-v1");
        bytes
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn registration(&mut self, registration: &InvariantRegistration) {
        self.descriptor(&registration.descriptor);
        self.rule(&registration.rule);
        self.execution_point(registration.execution_point);
        self.failure_effect(registration.failure_effect);
    }

    fn descriptor(&mut self, descriptor: &InvariantRuleDescriptor) {
        self.rule_id(&descriptor.id);
        self.supported_execution_points(descriptor.execution_points);
        self.u32(descriptor.groups.mask());
        self.cost_class(descriptor.cost_class);
        self.failure_effect(descriptor.failure_effect);
        self.semantics_class(descriptor.semantics);
    }

    fn rule_id(&mut self, rule_id: &InvariantRuleId) {
        match rule_id {
            InvariantRuleId::Native(native) => {
                self.tag(1);
                self.native_rule_id(*native);
            }
            InvariantRuleId::Custom(custom) => {
                self.tag(2);
                self.string(custom.as_str());
            }
        }
    }

    fn rule(&mut self, rule: &InvariantRule) {
        match rule {
            InvariantRule::LiveRecordRequiresSidecar(kind) => {
                self.tag(1);
                self.record_kind(kind);
            }
            InvariantRule::MaxMergedIntents(limit) => {
                self.tag(2);
                self.usize(*limit);
            }
            // Rule-body tag 3 stays retired; surviving meanings keep their tags.
            InvariantRule::MaxSnapshotEntities(limit) => {
                self.tag(4);
                self.usize(*limit);
            }
            InvariantRule::UniqueEntityAspectField { field_locator } => {
                self.tag(5);
                self.string(field_locator.aspect().aspect_key().as_str());
                self.field_path(field_locator.field_path().fields());
            }
            InvariantRule::EndpointKindContract(contract) => {
                self.tag(6);
                self.endpoint_kind_contract(contract);
            }
            InvariantRule::CardinalityMaximumContract(contract) => {
                self.tag(7);
                self.cardinality_maximum_contract(contract);
            }
            InvariantRule::CardinalityMinimumContract(contract) => {
                self.tag(8);
                self.cardinality_minimum_contract(contract);
            }
            InvariantRule::UniquenessContract(contract) => {
                self.tag(9);
                self.uniqueness_contract(contract);
            }
            InvariantRule::SymmetryContract(contract) => {
                self.tag(10);
                self.symmetry_contract(contract);
            }
            InvariantRule::EndpointDeletionIntegrityContract(contract) => {
                self.tag(11);
                self.endpoint_deletion_integrity_contract(contract);
            }
            InvariantRule::AcyclicityContract(contract) => {
                self.tag(12);
                self.acyclicity_contract(contract);
            }
            InvariantRule::PartitionIsolationContract(contract) => {
                self.tag(13);
                self.partition_isolation_contract(contract);
            }
            InvariantRule::ConnectivityMinimumContract(contract) => {
                self.tag(14);
                self.connectivity_minimum_contract(contract);
            }
        }
    }

    fn endpoint_kind_contract(&mut self, contract: &LoweredEndpointKindContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.kind_ids(&contract.allowed_source_kinds);
        self.kind_ids(&contract.allowed_target_kinds);
        self.bool(contract.self_edges_allowed);
        self.cross_context_policy(contract.cross_context_policy);
        self.u128(contract.plan_revision.0);
    }

    fn cardinality_maximum_contract(&mut self, contract: &LoweredCardinalityMaximumContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.optional_u64(contract.source_max);
        self.optional_u64(contract.target_max);
        self.optional_u64(contract.pair_max);
        self.u128(contract.plan_revision.0);
    }

    fn cardinality_minimum_contract(&mut self, contract: &LoweredCardinalityMinimumContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.optional_u64(contract.source_min);
        self.optional_u64(contract.target_min);
        self.optional_u64(contract.pair_min);
        self.pair_minimum_semantics(contract.pair_min_semantics);
        self.kind_ids(&contract.candidate_source_kinds);
        self.kind_ids(&contract.candidate_target_kinds);
        self.minimum_cardinality_enforcement(contract.minimum_enforcement);
        self.u128(contract.plan_revision.0);
    }

    fn uniqueness_contract(&mut self, contract: &LoweredUniquenessContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.uniqueness_scope(contract.scope);
        self.u128(contract.plan_revision.0);
    }

    fn symmetry_contract(&mut self, contract: &LoweredSymmetryContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.symmetry_mode(contract.mode);
        self.u128(contract.plan_revision.0);
    }

    fn endpoint_deletion_integrity_contract(
        &mut self,
        contract: &LoweredEndpointDeletionIntegrityContract,
    ) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.endpoint_deletion_integrity_mode(contract.mode);
        self.cascade_delete_policy(contract.cascade_delete_policy);
        self.u128(contract.plan_revision.0);
    }

    fn acyclicity_contract(&mut self, contract: &LoweredAcyclicityContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.directed_traversal_kind(contract.traversal_direction);
        self.allowed_cycle_class(contract.allowed_cycle_class);
        self.u128(contract.plan_revision.0);
    }

    fn partition_isolation_contract(&mut self, contract: &LoweredPartitionIsolationContract) {
        self.contract_identity(
            contract.contract_id.as_str(),
            contract.relation_kind_id.as_u32(),
        );
        self.partition_isolation_mode(contract.isolation_mode);
        self.u128(contract.plan_revision.0);
    }

    fn connectivity_minimum_contract(&mut self, contract: &LoweredConnectivityMinimumContract) {
        self.string(contract.contract_id.as_str());
        self.kind_ids(&contract.source_kind_ids);
        self.u32(contract.relation_kind_id.as_u32());
        self.kind_ids(&contract.target_kind_ids);
        self.u32(contract.minimum_reachable_targets);
        self.connectivity_minimum_enforcement(contract.enforcement_boundary);
        self.u128(contract.plan_revision.0);
    }

    fn contract_identity(&mut self, contract_id: &str, relation_kind_id: u32) {
        self.string(contract_id);
        self.u32(relation_kind_id);
    }

    fn field_path(&mut self, fields: &[worth_foundational::facade::FieldKey]) {
        self.usize(fields.len());
        for field in fields {
            self.string(field.as_str());
        }
    }

    fn kind_ids(&mut self, kind_ids: &[crate::identity::data::KindId]) {
        self.usize(kind_ids.len());
        for kind_id in kind_ids {
            self.u32(kind_id.as_u32());
        }
    }

    fn supported_execution_points(&mut self, supported: SupportedExecutionPoints) {
        for point in [
            InvariantExecutionPoint::MutationSensitive,
            InvariantExecutionPoint::CommitBoundary,
            InvariantExecutionPoint::SnapshotPublication,
            InvariantExecutionPoint::CertificationBoundary,
            InvariantExecutionPoint::HarnessAudit,
            InvariantExecutionPoint::GraphComposition,
        ] {
            self.bool(supported.supports(point));
        }
    }

    fn native_rule_id(&mut self, value: NativeInvariantRuleId) {
        self.tag(tags::native_rule_id_tag(value));
    }

    fn record_kind(&mut self, value: &RecordKindTag) {
        self.tag(tags::record_kind_tag(value));
    }

    fn execution_point(&mut self, value: InvariantExecutionPoint) {
        self.tag(tags::execution_point_tag(value));
    }

    fn failure_effect(&mut self, value: InvariantFailureEffect) {
        self.tag(tags::failure_effect_tag(value));
    }

    fn cost_class(&mut self, value: InvariantCostClass) {
        self.tag(tags::cost_class_tag(value));
    }

    fn semantics_class(&mut self, value: InvariantSemanticsClass) {
        self.tag(tags::semantics_class_tag(value));
    }

    fn cross_context_policy(&mut self, value: CrossContextPolicy) {
        self.tag(tags::cross_context_policy_tag(value));
    }

    fn cascade_delete_policy(&mut self, value: CascadeDeletePolicy) {
        self.tag(tags::cascade_delete_policy_tag(value));
    }

    fn pair_minimum_semantics(&mut self, value: PairMinimumSemantics) {
        self.tag(tags::pair_minimum_semantics_tag(value));
    }

    fn minimum_cardinality_enforcement(&mut self, value: MinimumCardinalityEnforcement) {
        self.tag(tags::minimum_cardinality_enforcement_tag(value));
    }

    fn uniqueness_scope(&mut self, value: UniquenessScope) {
        self.tag(tags::uniqueness_scope_tag(value));
    }

    fn symmetry_mode(&mut self, value: SymmetryMode) {
        self.tag(tags::symmetry_mode_tag(value));
    }

    fn endpoint_deletion_integrity_mode(&mut self, value: EndpointDeletionIntegrityMode) {
        self.tag(tags::endpoint_deletion_integrity_mode_tag(value));
    }

    fn directed_traversal_kind(&mut self, value: DirectedTraversalKind) {
        self.tag(tags::directed_traversal_kind_tag(value));
    }

    fn allowed_cycle_class(&mut self, value: AllowedCycleClass) {
        self.tag(tags::allowed_cycle_class_tag(value));
    }

    fn partition_isolation_mode(&mut self, value: PartitionIsolationMode) {
        self.tag(tags::partition_isolation_mode_tag(value));
    }

    fn connectivity_minimum_enforcement(&mut self, value: ConnectivityMinimumEnforcement) {
        self.tag(tags::connectivity_minimum_enforcement_tag(value));
    }

    fn optional_u64(&mut self, value: Option<u64>) {
        match value {
            Some(value) => {
                self.tag(1);
                self.u64(value);
            }
            None => self.tag(0),
        }
    }

    fn bool(&mut self, value: bool) {
        self.tag(u8::from(value));
    }

    fn tag(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn usize(&mut self, value: usize) {
        self.u64(value as u64);
    }

    fn string(&mut self, value: &str) {
        self.u32(value.len() as u32);
        self.bytes.extend_from_slice(value.as_bytes());
    }
}

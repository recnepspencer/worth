use super::relational_provenance::provenance_packet;
use super::*;
use crate::facade::{
    RelationalBridgeBranchHeadLease, RelationalBridgeObservationLease, SnapshotReadContract,
};
use crate::harness::fixtures::RelationalFixture;
use crate::relational_source::relational_test_support::{
    aspect_key, bridge_entity_identity, field_key,
};
use worth_foundational::facade::{aspects, AspectBinding, AspectIdentity, ScalarAspectType};
use worth_relational::facade::{identity::EntityId, schema::DeclaredAspectContractBinding};

use crate::adapter::RelationalBridgeSource;
use crate::builder::{
    PresentCommittedPatchSource, PresentMappingRegistrations, PresentSignalSink,
    PresentSnapshotReadSource, PresentTruthBranchHeadSource,
};

use super::capture::PricingFaultSource;

type PricingBuilder = RuntimeBridgeBuilder<
    PresentCommittedPatchSource,
    PresentSnapshotReadSource,
    PresentSignalSink,
    PresentTruthBranchHeadSource,
    PresentMappingRegistrations,
>;

pub(in crate::harness::tests::pricing_shock) struct RelationalPricing {
    pub(in crate::harness::tests::pricing_shock::support) native: RelationalFixture,
    pub(in crate::harness::tests::pricing_shock::support) steel: EntityId,
    pub(in crate::harness::tests::pricing_shock::support) rubber: EntityId,
    pub steel_commit: TruthCommitIdentity,
    pub rubber_commit: TruthCommitIdentity,
    pub shock_commit: TruthCommitIdentity,
    pub(in crate::harness::tests::pricing_shock::support) steel_observation:
        RelationalBridgeObservationLease,
    pub(in crate::harness::tests::pricing_shock::support) main_head:
        RelationalBridgeBranchHeadLease,
    pub(in crate::harness::tests::pricing_shock::support) shock_head:
        RelationalBridgeBranchHeadLease,
}

impl RelationalPricing {
    pub fn into_scenario(self) -> crate::harness::fixtures::ScenarioRelationalTruth {
        crate::harness::fixtures::ScenarioRelationalTruth::new_with_heads(
            self.native,
            self.steel_commit,
            vec![self.steel_observation],
            vec![self.main_head, self.shock_head],
        )
    }

    pub fn steel_snapshot(&self) -> &TruthSnapshotIdentity {
        self.steel_observation.snapshot_identity()
    }

    pub fn main_snapshot(&self) -> &TruthSnapshotIdentity {
        self.main_head.snapshot_identity()
    }

    pub fn shock_snapshot(&self) -> &TruthSnapshotIdentity {
        self.shock_head.snapshot_identity()
    }

    pub fn rubber_read_packet(&self) -> SnapshotReadPacket {
        self.cost_packet(PricingMaterial::Rubber)
    }

    pub fn cost_packet(&self, material: PricingMaterial) -> SnapshotReadPacket {
        SnapshotReadPacket::new(vec![SnapshotReadRequest::for_relational_record(
            bridge_entity_identity(self.component(material)),
            SnapshotReadContract::new(pricing_aspect().contract),
        )])
    }

    pub(in crate::harness::tests::pricing_shock::support) fn component(
        &self,
        material: PricingMaterial,
    ) -> EntityId {
        match material {
            PricingMaterial::Steel => self.steel,
            PricingMaterial::Rubber => self.rubber,
            _ => panic!("the two-component pricing fixture does not author {material:?}"),
        }
    }

    pub fn rubber_provenance_packet(&self) -> SnapshotReadPacket {
        provenance_packet(bridge_entity_identity(self.rubber))
    }

    pub fn begin_preview(
        &self,
        runtime: &RuntimeBridge,
        identity: BridgePreviewSessionIdentity,
    ) -> BridgePreviewSessionHandle {
        runtime.begin_preview(
            identity,
            BridgePreviewBranchBinding::new(
                TruthBranchIdentity::from_relational_branch_id("pricing-shock"),
                BridgeSignalBranchIdentity::from_stable_name("signal:pricing-shock"),
            ),
            self.shock_snapshot().clone(),
        )
    }
}

pub(super) fn pricing_aspect() -> DeclaredAspectContractBinding {
    let shape = aspects()
        .struct_fields()
        .required("usd", ScalarAspectType::String)
        .finish()
        .unwrap();
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: field_key("cost"),
        },
        contract: aspects()
            .contract()
            .for_key(aspect_key("cost"))
            .identified_by(AspectIdentity(1))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    }
}

fn cost_mapping(entity: EntityId, scope: &str) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name(scope),
        TruthPatchScope::for_entity_field(
            MappingSelector::exact(bridge_entity_identity(entity).bridge_entity_identity()),
            aspect_key("cost"),
            field_key("usd"),
        ),
        SnapshotReadContract::new(pricing_aspect().contract),
        SignalInvalidationScope::admit_bridge_owned(scope),
        CoarseRoutingMode::Direct,
    )
}

impl RelationalPricing {
    pub fn mappings(&self) -> Vec<BridgeMappingRegistration> {
        vec![
            cost_mapping(self.steel, "price:bicycle"),
            cost_mapping(self.steel, "price:wheelbarrow"),
            cost_mapping(self.rubber, "price:scooter"),
        ]
    }

    fn builder<S: RelationalBridgeSource + Clone>(
        &self,
        source: S,
        sink: RecordingSignalBridgeSink,
        policy: BridgeRuntimePolicy,
    ) -> PricingBuilder {
        RuntimeBridgeBuilder::new()
            .with_truth_source(source.clone())
            .with_truth_branch_head_source(source)
            .with_compute_sink(sink)
            .with_policy(policy)
            .register_mapping(cost_mapping(self.steel, "price:bicycle"))
            .register_mapping(cost_mapping(self.steel, "price:wheelbarrow"))
            .register_mapping(cost_mapping(self.rubber, "price:scooter"))
    }

    pub fn runtime(&self, sink: RecordingSignalBridgeSink) -> RuntimeBridge {
        self.runtime_with_policy(sink, BridgeRuntimePolicy::development())
    }

    pub fn runtime_with_policy(
        &self,
        sink: RecordingSignalBridgeSink,
        policy: BridgeRuntimePolicy,
    ) -> RuntimeBridge {
        self.builder(self.native.source.clone(), sink, policy)
            .build()
            .expect("native pricing runtime")
    }

    pub fn runtime_with_historical_source(&self, declaration: SourceDeclaration) -> RuntimeBridge {
        self.builder(
            self.native.source.clone(),
            RecordingSignalBridgeSink::default(),
            BridgeRuntimePolicy::development(),
        )
        .with_source_adapter(self.native.source.clone())
        .register_source(declaration)
        .build()
        .expect("native pricing runtime with a historical source")
    }

    pub fn aspect_runtime(&self, policy: BridgeRuntimePolicy) -> RuntimeBridge {
        let mapping = cost_mapping(self.steel, "price:bicycle");
        let aspect = BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::admit_bridge_owned("pricing-steel-usd-field"),
            mapping.truth_scope().clone(),
            mapping.snapshot_read_contract().clone(),
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        );
        RuntimeBridgeBuilder::new()
            .with_truth_source(self.native.source.clone())
            .with_truth_branch_head_source(self.native.source.clone())
            .with_compute_sink(RecordingSignalBridgeSink::default())
            .with_policy(policy)
            .register_mapping(mapping)
            .register_aspect_mapping(aspect)
            .build()
            .expect("native pricing aspect runtime")
    }

    pub fn writeback_runtime(
        &self,
        policy: BridgeRuntimePolicy,
        authority: impl TruthWritebackAuthority,
    ) -> RuntimeBridge {
        self.builder(
            self.native.source.clone(),
            RecordingSignalBridgeSink::default(),
            policy,
        )
        .with_writeback_authority(authority)
        .build()
        .expect("native pricing runtime with writeback authority")
    }

    pub fn fault_runtime(&self, fault: PricingFault, policy: BridgeRuntimePolicy) -> RuntimeBridge {
        self.builder(
            PricingFaultSource(self.native.source.clone(), fault),
            RecordingSignalBridgeSink::default(),
            policy,
        )
        .build()
        .expect("native pricing runtime with a selected source fault")
    }

    pub fn cost_item(&self, material: PricingMaterial) -> BridgeCommittedPatchItem {
        BridgeCommittedPatchItem::with_relational_record_target(
            bridge_entity_identity(self.component(material)),
            crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                worth_foundational::facade::AspectLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    aspect_key("cost"),
                ),
                worth_foundational::facade::CanonicalFieldPath::single(field_key("usd")),
            ),
        )
    }

    pub fn fanout_runtime(&self, sink: RecordingSignalBridgeSink) -> RuntimeBridge {
        let mut builder = RuntimeBridgeBuilder::new()
            .with_truth_source(self.native.source.clone())
            .with_truth_branch_head_source(self.native.source.clone())
            .with_compute_sink(sink)
            .register_mapping(cost_mapping(self.steel, "price:product-000"));
        for product in 1..100 {
            builder = builder.register_mapping(cost_mapping(
                self.steel,
                &format!("price:product-{product:03}"),
            ));
        }
        builder.build().expect("native fanout runtime")
    }
}

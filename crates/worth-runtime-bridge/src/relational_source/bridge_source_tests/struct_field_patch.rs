use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, PartitionId},
    symbols::ClientKey,
    transactions::{
        CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

use super::support::RelationalFixture;
use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeContinuityAuthorityBasis,
    BridgeLineageContext, BridgeMappingId, BridgeMappingRegistration, CoarseRoutingMode,
    MappingSelector, RelationalBridgeObservationLease, SignalInvalidationScope,
    SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind, TruthBranchIdentity,
    TruthCommitIdentity, TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};
use crate::relational_source::relational_test_support::{
    aspect_key, changed_entities, entity_summary_struct_aspect, field_key, fork_branch,
    single_string_aspect_field_patch, string_aspect_field_patch,
    test_owner_begin_transaction_for_branch, test_owner_begin_transaction_for_main,
    AspectSchemaFixture, ENTITY_KIND,
};

pub struct ProfileTruth {
    pub native: RelationalFixture,
    pub commit: TruthCommitIdentity,
    pub observation: RelationalBridgeObservationLease,
    branch: TruthBranchIdentity,
    entity: EntityId,
}

impl ProfileTruth {
    pub fn new(execution: ExecutionRequest<'_, '_>) -> Self {
        Self::on_branch_with_field(execution, "main", "title")
    }

    pub fn on_branch_with_field(
        execution: ExecutionRequest<'_, '_>,
        branch: &str,
        field: &str,
    ) -> Self {
        let main = BranchId("main".to_owned());
        let branch_id = BranchId(branch.to_owned());
        let native = RelationalFixture::attach(
            AspectSchemaFixture {
                entity_aspects: vec![entity_summary_struct_aspect(
                    aspect_key("profile"),
                    field_key("profile"),
                )],
                ..AspectSchemaFixture::default()
            }
            .build_runtime(),
            "model",
        );
        let runtime = &native.runtime;
        let mut create = test_owner_begin_transaction_for_main(runtime);
        create
            .push_batch(
                WorkerIntentBatch::new("create-profile").push(MutationIntent::Create(
                    CreateIntent::Entity(EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: ENTITY_KIND,
                        client_key: ClientKey::raw("profile"),
                        fields: string_aspect_field_patch([
                            (aspect_key("profile"), field_key("title"), "before"),
                            (aspect_key("profile"), field_key("status"), "open"),
                        ]),
                    }),
                )),
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let entity = changed_entities(&create.commit(&execution, runtime).unwrap())[0];
        if branch_id != main {
            fork_branch(&execution, runtime, branch_id.clone(), &main);
        }
        let commit = publish_update(execution, &native, &branch_id, entity, field, "alice");
        let selected_branch = runtime.branch_identity(&branch_id).unwrap();
        let (_, basis) = native
            .source
            .observe_branch_basis(&selected_branch)
            .unwrap();
        let observation = native
            .source
            .retain_branch_basis_for_bridge(&basis)
            .unwrap();
        Self {
            native,
            commit,
            observation,
            branch: TruthBranchIdentity::from_relational_branch_id(branch),
            entity,
        }
    }

    pub fn update_field(
        &self,
        execution: ExecutionRequest<'_, '_>,
        field: &str,
        value: &str,
    ) -> (TruthCommitIdentity, RelationalBridgeObservationLease) {
        let branch = BranchId(self.branch.relational_branch_id().unwrap().to_owned());
        let commit = publish_update(execution, &self.native, &branch, self.entity, field, value);
        let branch_identity = self.native.runtime.branch_identity(&branch).unwrap();
        let (_, basis) = self
            .native
            .source
            .observe_branch_basis(&branch_identity)
            .unwrap();
        (
            commit,
            self.native
                .source
                .retain_branch_basis_for_bridge(&basis)
                .unwrap(),
        )
    }

    pub fn into_scenario(self) -> crate::harness::fixtures::ScenarioRelationalTruth {
        crate::harness::fixtures::ScenarioRelationalTruth::new(
            self.native,
            self.commit,
            vec![self.observation],
            None,
        )
    }

    pub fn field() -> FieldKey {
        field_key("title")
    }

    pub fn mapping() -> BridgeMappingRegistration {
        let key = aspect_key("profile");
        let path = CanonicalFieldPath::single(Self::field());
        BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("profile-title"),
            TruthPatchScope::for_target(
                MappingSelector::any(),
                key.clone(),
                TruthPatchTargetSelector::entity_field_path(path),
            ),
            SnapshotReadContract::new(
                entity_summary_struct_aspect(key, field_key("profile")).contract,
            ),
            SignalInvalidationScope::from_stable_name("signal.profile"),
            CoarseRoutingMode::Direct,
        )
    }

    pub fn widening_mapping() -> BridgeMappingRegistration {
        let mapping = Self::mapping();
        BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("profile-surface-widening"),
            TruthPatchScope::for_target(
                MappingSelector::any(),
                aspect_key("profile"),
                TruthPatchTargetSelector::any(),
            ),
            mapping.snapshot_read_contract().clone(),
            SignalInvalidationScope::from_stable_name("signal.profile.widening"),
            CoarseRoutingMode::Direct,
        )
    }

    pub fn aspect_mapping() -> BridgeAspectRegistration {
        Self::aspect_mapping_with_kind("profile-title-field", SubscriptionSliceKind::SignalField)
    }

    pub fn aspect_mapping_with_kind(
        registration: &str,
        kind: SubscriptionSliceKind,
    ) -> BridgeAspectRegistration {
        let mapping = Self::mapping();
        BridgeAspectRegistration::new(
            BridgeAspectRegistrationId::from_stable_name(registration),
            mapping.truth_scope().clone(),
            mapping.snapshot_read_contract().clone(),
            TruthDeltaSurfaceKind::EntityField,
            kind,
            SliceWideningPolicy::Disallow,
        )
    }

    pub fn lineage_context(&self) -> BridgeLineageContext {
        BridgeLineageContext::new(BridgeContinuityAuthorityBasis::new(
            self.branch.clone(),
            self.observation.snapshot_identity().clone(),
        ))
    }
}

fn publish_update(
    execution: ExecutionRequest<'_, '_>,
    native: &RelationalFixture,
    branch: &BranchId,
    entity: EntityId,
    field: &str,
    value: &str,
) -> TruthCommitIdentity {
    let runtime = &native.runtime;
    let mut update = test_owner_begin_transaction_for_branch(runtime, branch.clone());
    update
        .push_batch(
            WorkerIntentBatch::new("update-profile").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: single_string_aspect_field_patch(
                        aspect_key("profile"),
                        field_key(field),
                        value,
                    ),
                }),
            )),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let commit_id = update
        .commit(&execution, &runtime)
        .unwrap()
        .commit
        .commit_id;
    TruthCommitIdentity::from_relational_commit_id(commit_id.0)
}

#[test]
fn a_committed_struct_field_patch_publishes_one_field_precise_target() {
    use crate::facade::{CommittedPatchSource, RelationalCommittedPatchRequest};
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = ProfileTruth::new(execution);
    let envelope = truth
        .native
        .source
        .load_committed_patch(
            RelationalCommittedPatchRequest::new(truth.commit.clone()),
            execution,
        )
        .unwrap();

    let items = envelope.patch_body().canonical_items();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].surface_kind(), TruthDeltaSurfaceKind::EntityField);
    assert_eq!(
        items[0]
            .field_locator()
            .expect("field-precise bridge target")
            .field_path()
            .fields(),
        &[ProfileTruth::field()]
    );
}

#[test]
fn coarse_and_fine_routes_share_the_native_field_slice() {
    use super::support::TestSink;
    use crate::facade::{BridgeRouteRequest, RuntimeBridgeBuilder};
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = ProfileTruth::new(execution);
    let exact = ProfileTruth::mapping();
    let coarse = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("coarse-field"),
        TruthPatchScope::for_target(
            MappingSelector::any(),
            aspect_key("profile"),
            TruthPatchTargetSelector::any(),
        ),
        exact.snapshot_read_contract().clone(),
        exact.signal_scope().clone(),
        CoarseRoutingMode::Direct,
    );

    let route = |registration| {
        let bridge = RuntimeBridgeBuilder::new()
            .with_relational_source(truth.native.source.clone())
            .with_signal_sink(TestSink)
            .register_mapping(registration)
            .register_aspect_mapping(ProfileTruth::aspect_mapping())
            .build()
            .unwrap();
        let plan = bridge
            .plan_committed_patch(
                BridgeRouteRequest::for_commit(truth.commit.clone()),
                execution,
            )
            .unwrap();
        bridge.deliver_invalidation(plan, execution).unwrap()
    };
    let fine_result = route(exact);
    let coarse_result = route(coarse);
    let fine = fine_result.result_summary();
    let coarse = coarse_result.result_summary();
    assert_eq!(fine.subscription_slice_count(), 1);
    assert_eq!(coarse.subscription_slice_count(), 1);
    assert_eq!(
        fine.subscription_slice_identity(),
        coarse.subscription_slice_identity()
    );
}

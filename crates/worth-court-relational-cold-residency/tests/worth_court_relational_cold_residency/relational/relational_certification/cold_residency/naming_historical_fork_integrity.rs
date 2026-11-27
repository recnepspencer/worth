//! Cold-residency courts for historical-fork `CHK24` integrity and mixed
//! unnamed/named fork reopen:
//! - Tampered CHK24 emptying a historical fork's naming/index/group root
//!   after its parent advances past the fork cutoff refuses `CorruptCheckpoint`.
//! - Unnamed entity at C1 + named entities & group at C2 -> ACK -> historical
//!   fork at C1 -> child checkpoint & reopen preserves exact C1 state.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::config::{
    CascadeDeletePolicy, RelationalRuntimeProfile, ResidencyBudget,
};
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalCheckpointArtifactPolicy,
    RelationalCheckpointMember, RelationalDurableCheckpointSource, RelationalDurableColdSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::lineage::{
    InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry, LineageRecordTarget,
    PersistentNameOfRecord, PersistentNameSubjectKind,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry,
};
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, RecordRef, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, observe_branch, publish_group_partition,
    ENTITY_SCHEME, GROUP_SCHEME,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;

fn tamper_branch_root(
    c: &RelationalNativeCheckpoint,
    target_branch: &BranchId,
    empty_all: bool,
) -> RelationalNativeCheckpoint {
    let body = c.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let branch_rows = rows(field(naming, "branches"));
    let nil = rmp_serde::to_vec(&Option::<u8>::None).unwrap();
    let empty_pages = array(&[]);
    let mut edited: Vec<Vec<u8>> = Vec::with_capacity(branch_rows.len());
    let mut found = false;
    for row in branch_rows {
        let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
        if &name == target_branch {
            let r = if empty_all {
                let r = replace_field(row, "naming", &nil);
                let r = replace_field(&r, "index", &nil);
                let r = replace_field(&r, "groups", &nil);
                replace_field(&r, "pages", &empty_pages)
            } else {
                replace_field(row, "groups", &nil)
            };
            edited.push(r);
            found = true;
        } else {
            edited.push(row.to_vec());
        }
    }
    assert!(found, "target branch root must exist in manifest");
    let refs: Vec<&[u8]> = edited.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(naming, "branches", &array(&refs));
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_body = replace_field(body, "manifest", &new_manifest);
    RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat())
}

#[test]
fn tampered_chk24_emptying_historical_fork_after_parent_advances_refuses_corrupt_checkpoint() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    // Parent creates named entities + group at C1.
    let (_, e1, n1) =
        create_named_entities(request, &producer, &main, &["p0-a".into(), "p0-b".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e1[0],
        "p0-g1",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let basis_c1 = observe_branch(&producer, &main);

    // Parent advances ALL 3 components (naming, index, groups) past C1 at C2.
    let (_, e2, n2) = create_named_entities(request, &producer, &main, &["p0-c".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e2[0],
        "p0-g2",
        vec![vec![
            (n1[0], AggregateMemberRole::Anchor),
            (n1[1], AggregateMemberRole::Ordinary),
            (n2[0], AggregateMemberRole::Ordinary),
        ]],
    );

    // Fork historical child at C1 (strictly below parent's C2 head).
    let child = BranchId("hist-child-c1".into());
    fork_retained_at(request, &producer, &basis_c1, &child).unwrap();
    let (ck, source) = capture_external_checkpoint(request, &producer);

    // Both emptying the entire child root AND dropping only child's groups must refuse.
    for empty_all in [true, false] {
        let tampered = tamper_branch_root(&ck, &child, empty_all);
        let mut target = durable_named_runtime(&medium, u64::MAX / 4);
        let input = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                ck.covered_position().unwrap(),
                ck.covered_branch_transitions().unwrap(),
                tampered,
            )),
            Vec::new(),
            Vec::new(),
            source.clone(),
        );
        let err = match target.durability().recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        ) {
            Err(err) => err,
            Ok(plan) => target
                .durability_recovery()
                .recover(request, plan)
                .expect_err("emptying historical fork root after parent advances must refuse"),
        };
        assert!(
            matches!(
                err,
                RelationalRecoveryStop::Durability(ref d)
                    if d.class == RecoveryFailureClass::CorruptCheckpoint
            ),
            "empty_all={empty_all}: expected CorruptCheckpoint, got {err:?}"
        );
    }
}

fn mixed_named_unnamed_schema() -> RelationalSchemaRegistry {
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let base = fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "unnamed_entity".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::new(
                fixture.entity_aspects.clone(),
            ),
        })
        .unwrap();
    let mut schemes = InstalledPersistentNameSchemeRegistry::new();
    schemes
        .register(
            InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
                ENTITY_SCHEME,
                PersistentNameSubjectKind::Entity,
                "test.entity",
                ["authored"],
                false,
                false,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    base.install_persistent_name_schemes(
        schemes,
        BTreeMap::from([(
            (PersistentNameSubjectKind::Entity, KindId(1)),
            ENTITY_SCHEME.into(),
        )]),
    )
    .unwrap()
}

pub(super) fn durable_mixed_runtime(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(mixed_named_unnamed_schema())
        .residency_budget(ResidencyBudget {
            head_working_set_bytes: u64::MAX / 4,
            historical_working_set_bytes: u64::MAX / 4,
            max_in_flight_fill_bytes: u64::MAX / 4,
            streaming_window_bytes: 64 * 1024,
        })
        .build_durable(medium.attachment())
        .unwrap()
}

#[test]
fn unnamed_entity_at_c1_then_named_and_group_at_c2_historical_fork_survives_reopen() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_mixed_runtime(&medium);
    let main = producer.main_branch_identity().branch_id().clone();

    // C1: create ONLY an unnamed entity (KindId(3) has no persistent name scheme).
    let basis0 = observe_branch(&producer, &main);
    let mut tx = producer
        .begin_branch_transaction(&basis0, RelationalTransactionIntent::ordinary())
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("unnamed-c1").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(3),
                client_key: "unnamed-1".into(),
                fields: field_patch("name", "name", "unnamed-1"),
            },
        ))),
        request.allocation_policy(),
    )
    .unwrap();
    let c1_res = tx.commit(request, &producer).unwrap();
    release_test_commit_snapshot(&producer, &c1_res);
    let unnamed_entity: EntityId = c1_res
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .unwrap();
    let basis_c1 = observe_branch(&producer, &main);

    // C2: create named entities + group on `main`, then checkpoint & ACK.
    let (_, e2, n2) =
        create_named_entities(request, &producer, &main, &["n2-a".into(), "n2-b".into()]);
    publish_group_partition(
        request,
        &producer,
        &main,
        e2[0],
        "g2",
        vec![vec![
            (n2[0], AggregateMemberRole::Anchor),
            (n2[1], AggregateMemberRole::Ordinary),
        ]],
    );
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let ck1 = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(source.clone()),
            },
        )
        .unwrap();
    medium.file_checkpoint(ck1.clone());
    request
        .run_native_with_checkpoint(|ctx| ck1.acknowledge_selected(request, ctx))
        .unwrap()
        .0
        .unwrap();

    // Historical fork at C1 after C2 ACK, then checkpoint child & reopen.
    let child = BranchId("unnamed-c1-child".into());
    fork_retained_at(request, &producer, &basis_c1, &child).unwrap();
    let ck1_members: Vec<_> = ck1
        .members()
        .iter()
        .map(RelationalCheckpointMember::locator)
        .collect();
    let ck2 = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &ck1_members,
                source: Some(source),
            },
        )
        .unwrap();
    medium.file_checkpoint(ck2.clone());
    request
        .run_native_with_checkpoint(|ctx| ck2.acknowledge_selected(request, ctx))
        .unwrap()
        .0
        .unwrap();
    drop(producer);

    let mut reopened = durable_mixed_runtime(&medium);
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            request,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(request, plan)
        .expect("reopen of historical fork with only unnamed entity at C1 must succeed");

    let child_obs = observe_branch(&reopened, &child);
    assert!(
        reopened
            .branch_live_persistent_names(&child)
            .unwrap()
            .is_empty(),
        "child at C1 must contain zero C2 names"
    );
    reopened
        .with_persistent_names_at_observation(request, &child_obs.observation(), |v| {
            assert_eq!(
                v.name_of_record(LineageRecordTarget::Entity(unnamed_entity)),
                PersistentNameOfRecord::Unnamed,
                "unnamed entity created at C1 must remain bound as Unnamed"
            );
            assert_eq!(v.group_of_member(GROUP_SCHEME, &n2[0]), None);
        })
        .unwrap();
}

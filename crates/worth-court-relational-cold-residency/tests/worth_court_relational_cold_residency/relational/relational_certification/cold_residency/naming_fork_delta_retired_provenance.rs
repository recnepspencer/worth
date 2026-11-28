//! Read-free adversarial manifest courts for `retired_forks` graph validation:
//! combined live+retired cycles, unreferenced dangling retired edges,
//! `fork_commit > parent_head`, and zero `fork_commit`.

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::cold_recovery::capture_external_checkpoint;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{create_named_entities, durable_named_runtime, observe_branch};
use super::checkpoint_wire_fixture::{array, field, replace_field, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;
use super::naming_fork_delta_provenance::assert_corrupt_read_free;

#[derive(serde::Serialize)]
pub(super) struct RetiredForkWire<'a> {
    pub(super) branch: &'a BranchId,
    pub(super) parent: &'a BranchId,
    pub(super) fork_commit: u64,
}

#[test]
fn retired_fork_cycle_dangling_and_future_commit_refuse_read_free() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let mut producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let (c_main, _, _) = create_named_entities(request, &producer, &main, &["base-0".into()]);
    let main_head = c_main.commit.commit_id.0;

    let other = BranchId("ret-prov-other".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &other,
    )
    .unwrap();

    let orphan = BranchId("ret-prov-orphan".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &orphan,
    )
    .unwrap();
    let orphan_id = producer.branch_identity(&orphan).unwrap();
    assert!(producer
        .delete_branch(request, &orphan_id)
        .unwrap()
        .deleted()
        .is_some());

    let mid = BranchId("ret-prov-mid".into());
    let leaf = BranchId("ret-prov-leaf".into());
    fork_retained_at(request, &producer, &observe_branch(&producer, &main), &mid).unwrap();
    let _ = create_named_entities(request, &producer, &mid, &["mid-0".into()]);
    fork_retained_at(request, &producer, &observe_branch(&producer, &mid), &leaf).unwrap();
    let (c_leaf, _, _) = create_named_entities(request, &producer, &leaf, &["leaf-0".into()]);
    let leaf_head = c_leaf.commit.commit_id.0;
    assert!(leaf_head > main_head);

    let mid_id = producer.branch_identity(&mid).unwrap();
    assert!(producer
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");

    let rebuild_retired = |edges: &[RetiredForkWire<'_>]| {
        let encoded: Vec<Vec<u8>> = edges
            .iter()
            .map(|e| rmp_serde::to_vec_named(e).unwrap())
            .collect();
        let refs: Vec<&[u8]> = encoded.iter().map(Vec::as_slice).collect();
        let new_naming = replace_field(naming, "retired_forks", &array(&refs));
        let new_manifest = replace_field(manifest, "naming", &new_naming);
        let new_body = replace_field(body, "manifest", &new_manifest);
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat())
    };

    // Case 1: Combined live + retired cycle (`leaf -> mid -> leaf`).
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_retired(&[RetiredForkWire {
            branch: &mid,
            parent: &leaf,
            fork_commit: main_head,
        }]),
        "live+retired fork cycle",
    );

    // Case 2: Unreferenced dangling `retired_forks` edge (`orphan -> main`).
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_retired(&[
            RetiredForkWire {
                branch: &mid,
                parent: &main,
                fork_commit: main_head,
            },
            RetiredForkWire {
                branch: &orphan,
                parent: &main,
                fork_commit: main_head,
            },
        ]),
        "dangling unreferenced retired_forks edge",
    );

    // Case 3: `fork_commit > parent_head` (`other` is at `main_head`, while `leaf_head <= tail`).
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_retired(&[RetiredForkWire {
            branch: &mid,
            parent: &other,
            fork_commit: leaf_head,
        }]),
        "retired_forks edge with fork_commit exceeding parent head",
    );

    // Case 4: `fork_commit == 0` when manifest has commits.
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_retired(&[RetiredForkWire {
            branch: &mid,
            parent: &main,
            fork_commit: 0,
        }]),
        "retired_forks edge with zero fork_commit",
    );
}

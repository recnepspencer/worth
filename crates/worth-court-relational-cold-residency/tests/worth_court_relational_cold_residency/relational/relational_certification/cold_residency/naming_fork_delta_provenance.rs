//! Read-free adversarial manifest courts for fork-delta `CHK24` naming:
//! unrelated `delta_base` provenance, mismatched zero-page alias roots,
//! headerless non-cell delta roots, and conflicting locator descriptors.

use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RecoveryFailureClass, RecoveryVerificationMode, RelationalDurableCheckpointSource,
    RelationalDurableRecoverySource, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::bounded_naming_fixture::{
    create_named_entities, durable_named_runtime, naming_page_reads, observe_branch,
};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_branch_root_and_dissolve::fork_retained_at;

pub(super) fn assert_corrupt_read_free(
    request: &ExecutionRequest<'_, '_>,
    medium: &std::sync::Arc<TestDurableMedium>,
    base_ck: &RelationalNativeCheckpoint,
    tampered: RelationalNativeCheckpoint,
    label: &str,
) {
    let source = immutable_source(base_ck, medium);
    let mut target = durable_named_runtime(medium, u64::MAX / 4);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            base_ck.covered_position().unwrap(),
            base_ck.covered_branch_transitions().unwrap(),
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
            .expect_err(&format!("{label} must refuse")),
    };
    assert!(
        matches!(
            err,
            RelationalRecoveryStop::Durability(ref d)
                if d.class == RecoveryFailureClass::CorruptCheckpoint
        ),
        "{label}: expected CorruptCheckpoint, got {err:?}"
    );
    assert_eq!(
        naming_page_reads(&source),
        0,
        "{label} must fail read-free with 0 naming page reads"
    );
}

#[test]
fn unrelated_delta_base_alias_headerless_and_conflicting_locator_refuse_read_free() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let producer = durable_named_runtime(&medium, u64::MAX / 4);
    let main = producer.main_branch_identity().branch_id().clone();

    let _ = create_named_entities(request, &producer, &main, &["r-0".into(), "r-1".into()]);
    let branch_a = BranchId("prov-a".into());
    let branch_b = BranchId("prov-b".into());
    let grandchild_a = BranchId("prov-grandchild-a".into());
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &branch_a,
    )
    .unwrap();
    let _ = create_named_entities(request, &producer, &branch_a, &["pa-0".into()]);
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &branch_a),
        &grandchild_a,
    )
    .unwrap();
    let _ = create_named_entities(request, &producer, &grandchild_a, &["gca-0".into()]);
    let _ = create_named_entities(request, &producer, &main, &["r-2".into()]);
    fork_retained_at(
        request,
        &producer,
        &observe_branch(&producer, &main),
        &branch_b,
    )
    .unwrap();
    let _ = create_named_entities(request, &producer, &branch_b, &["pb-0".into()]);

    let (ck, _) = capture_external_checkpoint(request, &producer);
    let body = ck.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let branch_rows = rows(field(naming, "branches"));

    let rebuild_ck = |edited_rows: Vec<Vec<u8>>| {
        let refs: Vec<&[u8]> = edited_rows.iter().map(Vec::as_slice).collect();
        let new_naming = replace_field(naming, "branches", &array(&refs));
        let new_manifest = replace_field(manifest, "naming", &new_naming);
        let new_body = replace_field(body, "manifest", &new_manifest);
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &new_body].concat())
    };

    // Case 1: grandchild_a points delta_base at uncle branch_b (no shared parent/ancestor).
    let uncle_tamper: Vec<Vec<u8>> = branch_rows
        .iter()
        .map(|row| {
            let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if name == grandchild_a {
                replace_field(
                    row,
                    "delta_base",
                    &rmp_serde::to_vec(&Some(&branch_b)).unwrap(),
                )
            } else {
                row.to_vec()
            }
        })
        .collect();
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_ck(uncle_tamper),
        "uncle delta_base",
    );

    // Case 2: zero-page alias root pointing at sibling with different commit head.
    let alias_tamper: Vec<Vec<u8>> = branch_rows
        .iter()
        .map(|row| {
            let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if name == branch_a {
                let r = replace_field(row, "pages", &array(&[]));
                replace_field(
                    &r,
                    "delta_base",
                    &rmp_serde::to_vec(&Some(&branch_b)).unwrap(),
                )
            } else {
                row.to_vec()
            }
        })
        .collect();
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_ck(alias_tamper),
        "mismatched-head zero-page alias delta_base",
    );

    // Case 3: headerless non-cell root in manifest.naming.branches carrying delta_base.
    let nil = rmp_serde::to_vec(&Option::<u8>::None).unwrap();
    let phantom = {
        let r = replace_field(
            branch_rows[0],
            "branch",
            &rmp_serde::to_vec(&BranchId("zz-phantom".into())).unwrap(),
        );
        let r = replace_field(&r, "naming", &nil);
        let r = replace_field(&r, "index", &nil);
        let r = replace_field(&r, "groups", &nil);
        let r = replace_field(&r, "pages", &array(&[]));
        replace_field(&r, "delta_base", &rmp_serde::to_vec(&Some(&main)).unwrap())
    };
    let mut with_phantom: Vec<Vec<u8>> = branch_rows.iter().map(|r| r.to_vec()).collect();
    with_phantom.push(phantom);
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_ck(with_phantom),
        "headerless non-cell delta root",
    );

    // Case 4: one locator referenced by two branches with conflicting descriptors.
    let main_first_page = branch_rows
        .iter()
        .find_map(|row| {
            let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            (name == main).then(|| rows(field(row, "pages"))[0].to_vec())
        })
        .expect("main has pages");
    let conflict_page = replace_field(
        &main_first_page,
        "entries",
        &rmp_serde::to_vec(&250u32).unwrap(),
    );
    let conflict_tamper: Vec<Vec<u8>> = branch_rows
        .iter()
        .map(|row| {
            let name: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if name == branch_b {
                replace_field(row, "pages", &array(&[conflict_page.as_slice()]))
            } else {
                row.to_vec()
            }
        })
        .collect();
    assert_corrupt_read_free(
        request,
        &medium,
        &ck,
        rebuild_ck(conflict_tamper),
        "conflicting locator descriptors across branches",
    );
}

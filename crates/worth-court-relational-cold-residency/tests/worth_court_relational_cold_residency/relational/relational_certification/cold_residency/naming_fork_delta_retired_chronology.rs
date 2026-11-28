//! Covered-chronology court proving `manifest.naming.retired_forks` cannot
//! redirect a retired intermediate to an unrelated live branch even when the
//! forged parent has `head >= fork_commit` and matching active naming domains.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::durability::{
    RecoveryVerificationMode, RelationalCheckpointAdmissionArtifact,
    RelationalCompleteSourceArtifact, RelationalDurableCheckpointSource,
    RelationalInitialInputsRefusal,
};
use worth_relational::facade::history::BranchId;
use worth_testkit_relational::initial_input_source;

use super::bounded_naming_fixture::{create_named_entities, named_schema};
use super::checkpoint_wire_fixture::{array, field, replace_field, rows, CHECKPOINT_MAGIC};
use super::naming_fork_delta_retired_provenance::RetiredForkWire;

fn fork_live(
    request: &ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    source: &BranchId,
    target: &BranchId,
) {
    let services = runtime.owner_component_services();
    let port = services.fork_port();
    let (_, admitted_source) = port.observe_fork_source(source).unwrap();
    let reservation = port.reserve_fork_target(target.clone()).unwrap();
    let assoc = worth_relational::facade::mvcc::RelationalCallerAssociation::from_bytes(
        b"retired-chronology-live-fork",
    )
    .unwrap();
    let _ = port
        .fork_reserved_associated_with_basis(request, reservation, admitted_source, assoc)
        .unwrap();
}

fn replace_32(buf: &mut [u8], needle: &[u8; 32], replacement: &[u8; 32]) {
    let pos = buf
        .windows(32)
        .position(|w| w == needle)
        .expect("expected 32-byte digest in admission cut");
    buf[pos..pos + 32].copy_from_slice(replacement);
}

fn rehash_group(
    out: &mut [u8],
    dir: &mut Sha256,
    cut: &[u8],
    offset: &mut usize,
    pos: u64,
    commit: u64,
) {
    let role = out[*offset];
    let len = u64::from_le_bytes(out[*offset + 1..*offset + 9].try_into().unwrap()) as usize;
    let payload_start = *offset + 41;
    let payload = &out[payload_start..payload_start + len];
    let mut h = Sha256::new();
    h.update(b"worth.relational.checkpoint-admission.meaning.v1\0");
    h.update(1u32.to_le_bytes());
    h.update((cut.len() as u64).to_le_bytes());
    h.update(cut);
    h.update(pos.to_le_bytes());
    h.update(commit.to_le_bytes());
    h.update([role]);
    h.update((payload.len() as u64).to_le_bytes());
    h.update(payload);
    let digest: [u8; 32] = h.finalize().into();
    out[*offset + 9..*offset + 41].copy_from_slice(&digest);
    dir.update(&out[*offset..*offset + 41]);
    *offset = payload_start + len;
}

fn reseal_admission_image(
    orig: &[u8],
    old_src: &[u8; 32],
    new_src: &[u8; 32],
    old_cp: &[u8; 32],
    new_cp: &[u8; 32],
    request: &ExecutionRequest<'_, '_>,
) -> RelationalCheckpointAdmissionArtifact {
    let mut out = orig.to_vec();
    let cut_len = u32::from_le_bytes(out[12..16].try_into().unwrap()) as usize;
    let row_count = u64::from_le_bytes(out[16..24].try_into().unwrap()) as usize;
    let cut_end = 64 + cut_len;
    replace_32(&mut out[64..cut_end], old_src, new_src);
    replace_32(&mut out[64..cut_end], old_cp, new_cp);
    let cut = out[64..cut_end].to_vec();

    let mut dir = Sha256::new();
    dir.update(b"worth.relational.checkpoint-admission.directory.v1\0");
    dir.update(&out[..32]);
    dir.update(&cut);

    let mut off = cut_end;
    rehash_group(&mut out, &mut dir, &cut, &mut off, 0, 0);
    for _ in 0..row_count {
        let pos = u64::from_le_bytes(out[off..off + 8].try_into().unwrap());
        let commit = u64::from_le_bytes(out[off + 8..off + 16].try_into().unwrap());
        dir.update(&out[off..off + 16]);
        off += 16;
        for _ in 0..8 {
            rehash_group(&mut out, &mut dir, &cut, &mut off, pos, commit);
        }
    }
    assert_eq!(off, out.len());
    out[32..64].copy_from_slice(&dir.finalize());
    let len = out.len();
    let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(out.into_boxed_slice()));
    RelationalCheckpointAdmissionArtifact::from_untrusted_region(bytes, 0..len, request).unwrap()
}

#[test]
fn retired_fork_redirect_to_unrelated_branch_refuses_under_covered_chronology() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let (mut producer, medium) =
        initial_input_source::initialized_with_schema(named_schema(), request);
    let build_named_home = || {
        worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
            .schema_registry(named_schema())
            .build_durable(medium.attachment())
            .unwrap()
    };

    let main = producer.main_branch_identity().branch_id().clone();
    let (c_main, _, _) = create_named_entities(request, &producer, &main, &["chron-base".into()]);
    let main_head = c_main.commit.commit_id.0;

    // `other` is an unrelated live branch with `head >= fork_commit` and matching active domains.
    let other = BranchId("chron-other".into());
    fork_live(request, &producer, &main, &other);
    let _ = create_named_entities(request, &producer, &other, &["chron-other-0".into()]);

    // `mid` forks from `main`, spawns `leaf`, then retires.
    let mid = BranchId("chron-mid".into());
    let leaf = BranchId("chron-leaf".into());
    fork_live(request, &producer, &main, &mid);
    let _ = create_named_entities(request, &producer, &mid, &["chron-mid-0".into()]);
    fork_live(request, &producer, &mid, &leaf);
    let _ = create_named_entities(request, &producer, &leaf, &["chron-leaf-0".into()]);
    let mid_id = producer.branch_identity(&mid).unwrap();
    assert!(producer
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let complete = initial_input_source::capture_external(&producer, &medium, request);
    medium.file_checkpoint(complete.checkpoint().clone());

    // Authentic covered-chronology recovery succeeds.
    let authentic_input =
        initial_input_source::home_image_input(&medium, &complete, request).unwrap();
    let mut receiver_ok = build_named_home();
    receiver_ok
        .prepare_initial_schema_installation()
        .unwrap()
        .install(initial_input_source::additions())
        .unwrap();
    assert!(receiver_ok
        .durability()
        .recovery_plan_from_source(
            request,
            authentic_input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .is_ok());

    // Forge `retired_forks` to redirect `mid` -> `other` and point `leaf.delta_base = Some(other)`,
    // keeping valid active domains and `other.head >= fork_commit`, and re-seal both wrappers
    // so covered chronology (`chronology::validate`) is the layer rejecting the forged provenance.
    let orig_src = complete.source_artifact().as_bytes();
    let cp_len = u64::from_le_bytes(orig_src[28..36].try_into().unwrap()) as usize;
    let orig_cp = &orig_src[84..84 + cp_len];
    let body = orig_cp.strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let edited_branches: Vec<Vec<u8>> = rows(field(naming, "branches"))
        .into_iter()
        .map(|row| {
            let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if br == leaf {
                replace_field(
                    row,
                    "delta_base",
                    &rmp_serde::to_vec(&Some(&other)).unwrap(),
                )
            } else {
                row.to_vec()
            }
        })
        .collect();
    let b_refs: Vec<&[u8]> = edited_branches.iter().map(Vec::as_slice).collect();
    let forged_edge = rmp_serde::to_vec_named(&RetiredForkWire {
        branch: &mid,
        parent: &other,
        fork_commit: main_head,
    })
    .unwrap();
    let new_naming = replace_field(
        &replace_field(naming, "branches", &array(&b_refs)),
        "retired_forks",
        &array(&[forged_edge.as_slice()]),
    );
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_cp = [
        CHECKPOINT_MAGIC,
        &replace_field(body, "manifest", &new_manifest),
    ]
    .concat();

    let mut new_src = Vec::with_capacity(84 + new_cp.len() + (orig_src.len() - 84 - cp_len));
    new_src.extend_from_slice(&orig_src[..84]);
    new_src[28..36].copy_from_slice(&(new_cp.len() as u64).to_le_bytes());
    new_src.extend_from_slice(&new_cp);
    new_src.extend_from_slice(&orig_src[84 + cp_len..]);
    let mut h = Sha256::new();
    h.update(&new_src[..52]);
    h.update(&new_src[84..]);
    new_src[52..84].copy_from_slice(&h.finalize());

    let orig_image = complete
        .admission_image()
        .seal_home(
            worth_foundational::facade::HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
            [0; 3],
            [2; 32],
            request,
        )
        .unwrap();
    let forged_image = reseal_admission_image(
        orig_image.as_bytes(),
        &Sha256::digest(orig_src).into(),
        &Sha256::digest(&new_src).into(),
        &Sha256::digest(orig_cp).into(),
        &Sha256::digest(&new_cp).into(),
        request,
    );
    let forged_len = new_src.len();
    let forged_artifact = RelationalCompleteSourceArtifact::from_untrusted_region(
        ExecutionImmutableBytes::from_external_bytes(Arc::new(new_src.into_boxed_slice())),
        0..forged_len,
        request,
    )
    .unwrap();
    let forged_cp_source = RelationalDurableCheckpointSource::from_checkpoint_admission_image(
        forged_artifact,
        forged_image,
        request,
    )
    .unwrap();

    let forged_result =
        initial_input_source::home_checkpoint_input(&medium, forged_cp_source, request);
    assert!(
        matches!(forged_result, Err(RelationalInitialInputsRefusal::Malformed)),
        "covered chronology must refuse retired_forks redirecting mid to unrelated live branch other; got {forged_result:?}"
    );
}

#[test]
fn omitted_retired_fork_edge_and_emptied_leaf_refuses_under_covered_chronology() {
    let host =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let (mut producer, medium) =
        initial_input_source::initialized_with_schema(named_schema(), request);

    let main = producer.main_branch_identity().branch_id().clone();
    let _ = create_named_entities(request, &producer, &main, &["omit-base".into()]);

    // Fork `main -> mid -> leaf` with NO writes on either fork, then retire `mid`.
    let mid = BranchId("omit-mid".into());
    let leaf = BranchId("omit-leaf".into());
    fork_live(request, &producer, &main, &mid);
    fork_live(request, &producer, &mid, &leaf);
    let mid_id = producer.branch_identity(&mid).unwrap();
    assert!(producer
        .delete_branch(request, &mid_id)
        .unwrap()
        .deleted()
        .is_some());

    let complete = initial_input_source::capture_external(&producer, &medium, request);
    medium.file_checkpoint(complete.checkpoint().clone());

    // Omit `mid -> main` from `retired_forks` and empty `leaf`'s naming root.
    let orig_src = complete.source_artifact().as_bytes();
    let cp_len = u64::from_le_bytes(orig_src[28..36].try_into().unwrap()) as usize;
    let orig_cp = &orig_src[84..84 + cp_len];
    let body = orig_cp.strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = field(body, "manifest");
    let naming = field(manifest, "naming");
    let empty_none = rmp_serde::to_vec(&Option::<()>::None).unwrap();
    let empty_pages = array(&[]);
    let edited_branches: Vec<Vec<u8>> = rows(field(naming, "branches"))
        .into_iter()
        .map(|row| {
            let br: BranchId = rmp_serde::from_slice(field(row, "branch")).unwrap();
            if br == leaf {
                let r = replace_field(row, "naming", &empty_none);
                let r = replace_field(&r, "index", &empty_none);
                let r = replace_field(&r, "groups", &empty_none);
                let r = replace_field(&r, "pages", &empty_pages);
                replace_field(&r, "delta_base", &empty_none)
            } else {
                row.to_vec()
            }
        })
        .collect();
    let b_refs: Vec<&[u8]> = edited_branches.iter().map(Vec::as_slice).collect();
    let new_naming = replace_field(
        &replace_field(naming, "branches", &array(&b_refs)),
        "retired_forks",
        &array(&[]),
    );
    let new_manifest = replace_field(manifest, "naming", &new_naming);
    let new_cp = [
        CHECKPOINT_MAGIC,
        &replace_field(body, "manifest", &new_manifest),
    ]
    .concat();

    // Read-free validation refuses missing retired ancestor edge.
    let read_free_cp =
        worth_relational::facade::durability::RelationalNativeCheckpoint::from_untrusted_bytes(
            new_cp.clone(),
        );
    super::naming_fork_delta_provenance::assert_corrupt_read_free(
        request,
        &medium,
        complete.checkpoint(),
        read_free_cp,
        "omitted retired_forks edge with emptied leaf",
    );

    // Covered chronology validation ALSO refuses the omitted retired_forks edge.
    let mut new_src = Vec::with_capacity(84 + new_cp.len() + (orig_src.len() - 84 - cp_len));
    new_src.extend_from_slice(&orig_src[..84]);
    new_src[28..36].copy_from_slice(&(new_cp.len() as u64).to_le_bytes());
    new_src.extend_from_slice(&new_cp);
    new_src.extend_from_slice(&orig_src[84 + cp_len..]);
    let mut h = Sha256::new();
    h.update(&new_src[..52]);
    h.update(&new_src[84..]);
    new_src[52..84].copy_from_slice(&h.finalize());

    let orig_image = complete
        .admission_image()
        .seal_home(
            worth_foundational::facade::HomeIdentity::from_observed_bytes([3; 16]).unwrap(),
            [0; 3],
            [4; 32],
            request,
        )
        .unwrap();
    let forged_image = reseal_admission_image(
        orig_image.as_bytes(),
        &Sha256::digest(orig_src).into(),
        &Sha256::digest(&new_src).into(),
        &Sha256::digest(orig_cp).into(),
        &Sha256::digest(&new_cp).into(),
        request,
    );
    let forged_len = new_src.len();
    let forged_artifact = RelationalCompleteSourceArtifact::from_untrusted_region(
        ExecutionImmutableBytes::from_external_bytes(Arc::new(new_src.into_boxed_slice())),
        0..forged_len,
        request,
    )
    .unwrap();
    let forged_cp_source = RelationalDurableCheckpointSource::from_checkpoint_admission_image(
        forged_artifact,
        forged_image,
        request,
    )
    .unwrap();
    let forged_result =
        initial_input_source::home_checkpoint_input(&medium, forged_cp_source, request);
    assert!(
        matches!(
            forged_result,
            Err(RelationalInitialInputsRefusal::Malformed)
                | Err(RelationalInitialInputsRefusal::Native(_))
        ),
        "home recovery must refuse omitted retired_forks edge; got {forged_result:?}"
    );
}

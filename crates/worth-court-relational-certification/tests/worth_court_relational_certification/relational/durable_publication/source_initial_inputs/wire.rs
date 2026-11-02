//! Independent resealing changes roles while preserving the original CHK18 index.
use super::*;
#[test]
fn normalized_policy_rewrite_refuses_with_unchanged_canonical_index_and_resealed_wrapper() {
    require_rewrite_refusal(|normalized, _| {
        let policy = &mut normalized["canonical"][0]["mutation"]["cascade_delete_policy"];
        *policy = match policy.as_str().unwrap() {
            "RetainDanglingForAudit" => "CascadeDeleteRelations".into(),
            "CascadeDeleteRelations" => "RetainDanglingForAudit".into(),
            other => panic!("unexpected producer policy {other}"),
        };
    });
}
#[test]
fn normalized_schema_role_rewrite_refuses_with_unchanged_canonical_index_and_resealed_wrapper() {
    require_rewrite_refusal(|normalized, original| {
        let birth = &original["entries"][0]["Birth"]["schema_carrier"];
        assert_ne!(&normalized["canonical"][0]["prestate_schema"], birth);
        normalized["canonical"][0]["prestate_schema"] = birth.clone();
    });
}
fn require_rewrite_refusal(rewrite: impl FnOnce(&mut serde_json::Value, &serde_json::Value)) {
    let policy=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let (runtime, medium) = initialized(&request);
    let commit = create_entity_outcome(&request, &runtime, "original");
    release_test_commit_snapshot(&runtime, &commit);
    let complete = capture(&runtime, &request);
    medium.file_checkpoint(complete.checkpoint().clone());
    let raw = complete.source_artifact().as_bytes();
    let len = u64::from_le_bytes(raw[44..52].try_into().unwrap()) as usize;
    let prefix = raw.len() - len;
    let mut normalized = crate::source_wire_value::read(&raw[prefix..]);
    rewrite(&mut normalized, &initial(complete.source_artifact()));
    let bytes = rmp_serde::to_vec_named(&normalized).unwrap();
    let mut changed = raw[..prefix].to_vec();
    changed.extend_from_slice(&bytes);
    changed[44..52].copy_from_slice(&(bytes.len() as u64).to_le_bytes());
    use sha2::{Digest, Sha256};
    let mut digest = Sha256::new();
    digest.update(&changed[..52]);
    digest.update(&changed[84..]);
    changed[52..84].copy_from_slice(&digest.finalize());
    assert_eq!(&changed[84..prefix], &raw[84..prefix]);
    let length = changed.len();
    let changed =
        ExecutionImmutableBytes::from_external_bytes(Arc::new(changed.into_boxed_slice()));
    let artifact =
        RelationalCompleteSourceArtifact::from_untrusted_region(changed, 0..length, &request)
            .unwrap();
    assert!(matches!(
        home_input(&medium, artifact, &request),
        Err(RelationalInitialInputsRefusal::Malformed)
    ));
}

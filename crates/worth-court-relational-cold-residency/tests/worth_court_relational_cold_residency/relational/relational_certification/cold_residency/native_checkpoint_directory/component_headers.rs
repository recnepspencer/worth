//! The unchanged checkpoint reads, then every isolated component header mutation refuses.
use super::super::checkpoint_wire_fixture::omit_field;
use super::*;
use worth_relational::facade::branch::RelationalBranchBasisDescriptor;

#[test]
fn recorded_component_checkpoint_header_tamper_refuses_without_defaults_or_foreign_binding() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    create_entity(execution, &runtime, "actual-header-source");
    let native = capture(execution, &runtime);
    restore(execution, &native);
    let payload = native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
    let manifest = manifest(&native);
    let original = rows(field(manifest, "roots"))[0];
    for change in 0..6 {
        let mut roots = rows(field(manifest, "roots"));
        let mut components = rows(field(original, "recorded_components"));
        let leaf = match change {
            0 => {
                let amended = omit_field(manifest, "retained_empty_components");
                assert_header_refused(execution, payload, amended, change);
                continue;
            }
            1 => omit_field(original, "recorded_components"),
            2 => {
                components.push(components[0]);
                replace_field(original, "recorded_components", &array(&components))
            }
            3..=5 => {
                let mut descriptor = serde_json::to_value(
                    rmp_serde::from_slice::<RelationalBranchBasisDescriptor>(components[0])
                        .unwrap(),
                )
                .unwrap();
                match change {
                    3 => descriptor["runtime_instance_id"] = serde_json::json!(42),
                    4 | 5 => {
                        let field = if change == 4 {
                            &mut descriptor["reference"]["target"]["Basis"]["roots"]["truth_root"]
                        } else {
                            &mut descriptor["schema_commitment"]
                        };
                        let digest: worth_foundational::CanonicalDigestId =
                            serde_json::from_value(field.clone()).unwrap();
                        let mut bytes = *digest.bytes();
                        bytes[0] ^= 1;
                        *field =
                            serde_json::to_value(worth_foundational::CanonicalDigestId::new(bytes))
                                .unwrap();
                    }
                    _ => unreachable!(),
                }
                let typed: RelationalBranchBasisDescriptor =
                    serde_json::from_value(descriptor).unwrap();
                let changed = rmp_serde::to_vec_named(&typed).unwrap();
                components[0] = &changed;
                replace_field(original, "recorded_components", &array(&components))
            }
            _ => unreachable!(),
        };
        roots[0] = &leaf;
        let amended = replace_field(manifest, "roots", &array(&roots));
        assert_header_refused(execution, payload, amended, change);
    }
}

fn assert_header_refused(
    execution: &ExecutionRequest<'_, '_>,
    payload: &[u8],
    manifest: Vec<u8>,
    change: usize,
) {
    let payload = replace_field(payload, "manifest", &manifest);
    let changed =
        RelationalNativeCheckpoint::from_untrusted_bytes([CHECKPOINT_MAGIC, &payload].concat());
    let stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &changed,
            worth_testkit_relational::cold_recovery::members_source(&changed),
        )
        .unwrap_err();
    let error = expect_durability_failure(stop);
    assert_eq!(
        error.class,
        RecoveryFailureClass::CorruptCheckpoint,
        "header mutation {change}: {error:?}"
    );
    assert!(
        error.detail.contains("component") || error.detail.contains("recorded_components"),
        "header mutation {change}: {error:?}"
    );
}

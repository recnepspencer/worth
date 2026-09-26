use std::path::Path;

const FORMER_SEMANTIC_IDENTITY_OWNERS: &[(&str, &str)] = &[
    (
        "declaration erased definition",
        include_str!("../../../../worth-query-declaration/src/application_query/erased_definition.rs"),
    ),
    (
        "declaration field selector",
        include_str!("../../../../worth-query-declaration/src/application_query/result_field_selector.rs"),
    ),
    (
        "declaration relation selector",
        include_str!("../../../../worth-query-declaration/src/application_query/result_relation_selector.rs"),
    ),
    ("installed contract", include_str!("../installed_contract.rs")),
    ("planning contract", include_str!("../planning_contract.rs")),
    ("read-family binding", include_str!("../read_family_binding.rs")),
    (
        "continuation contract",
        include_str!("../continuation_contract.rs"),
    ),
    (
        "admission requirements",
        include_str!("../../../../worth-query-admission/src/application_query/requirements.rs"),
    ),
    (
        "execution projected tree",
        include_str!(
            "../../../../worth-query-execution/src/domain_computation/primary_graph/application_query/projection/projected_tree.rs"
        ),
    ),
];

#[test]
fn portable_query_meaning_has_no_parallel_hash_or_warm_reconstruction_grammar() {
    for (owner, source) in FORMER_SEMANTIC_IDENTITY_OWNERS {
        for forbidden in [
            "Sha256",
            "hash_text_field",
            "derive_canonical_read_graph_planning_identity",
            "format!(\"{:?}\"",
        ] {
            assert!(
                !source.contains(forbidden),
                "{owner} reintroduced forbidden semantic identity residue: {forbidden}"
            );
        }
    }
}

#[test]
fn query_authority_uses_the_central_keyed_seal_family() {
    let query_seal = include_str!("../authority_seal.rs");
    let cryptography = include_str!("../../authority_cryptography.rs");

    assert!(query_seal.contains("AuthorityTranscript"));
    assert!(query_seal.contains("InstalledApplicationQuery"));
    assert!(!query_seal.contains("Sha256"));
    assert!(!query_seal.contains("nonce"));
    assert!(cryptography.contains("HmacSha256"));
    assert!(cryptography.contains("verify_slice"));
    assert!(cryptography.contains("getrandom::fill"));
}

#[test]
fn phase_six_semantic_families_share_the_foundational_digest_slot() {
    let declared_query = include_str!(
        "../../../../worth-query-declaration/src/application_query/canonical_basis/mod.rs"
    );
    let installed_query = include_str!("../canonical_basis/mod.rs");
    let declared_schema = include_str!(
        "../../../../worth-query-declaration/src/application_schema/canonical_identity.rs"
    );
    let installed_schema = include_str!("../../application_schema/canonical_identity.rs");
    let installed_authorization_policy =
        include_str!("../../application_operation/authorization_requirement.rs");
    let bridge_authorization = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/worth-runtime-bridge/src/authorization/runtime.rs"
    ));
    let live_open = include_str!(
        "../../../../worth-query-execution/src/domain_computation/primary_graph/application_query/live/lease/open.rs"
    );
    let query_definition = include_str!(
        "../../../../worth-query-declaration/src/application_query/canonical_basis/definition.rs"
    );
    let package = include_str!("../../package/identity.rs");
    let admission = include_str!("../../admission/identity.rs");
    let installed_index = include_str!("../../installed_index/index_identity.rs");
    let installation_digest = include_str!("../../canonical_digest_derivation.rs");
    let parameters = include_str!(
        "../../../../worth-query-admission/src/application_query/parameter_canonical_basis.rs"
    );
    let preconditions = include_str!(
        "../../../../worth-query-execution/src/domain_computation/primary_graph/application_attempt/precondition_binding/canonical_identity.rs"
    );
    let evidence_slot =
        include_str!("../../../../worth-query/src/evidence_identity/foundational.rs");
    let capability = concat!(
        include_str!(
            "../../../../worth-query/src/domain_capabilities/payloads/invariant_capability/mod.rs"
        ),
        include_str!(
            "../../../../worth-query/src/domain_capabilities/payloads/invariant_capability/graph_semantics.rs"
        ),
        include_str!(
            "../../../../worth-query/src/domain_capabilities/payloads/invariant_capability/payload.rs"
        ),
        include_str!(
            "../../../../worth-query/src/domain_capabilities/payloads/invariant_capability/posture.rs"
        ),
        include_str!(
            "../../../../worth-query/src/domain_capabilities/payloads/invariant_capability/registration_semantics.rs"
        ),
    );
    let aftermath =
        include_str!("../../../../worth-query/src/domain_capabilities/payloads/aftermath.rs");

    assert!(declared_query.contains("prepare_canonical_basis_sequence"));
    assert!(!declared_query.contains("CanonicalDigestAlgorithmId::sha256()"));
    assert!(!declared_query.contains("canonical_basis_sequence_material"));
    assert!(!declared_query.contains("use sha2"));
    assert!(!declared_query.contains("hash_parts"));
    assert!(declared_schema.contains("prepare_canonical_basis_sequence"));
    assert!(!declared_schema.contains("CanonicalDigestAlgorithmId::sha256()"));

    for (family, source) in [
        ("installed query", installed_query),
        ("installed schema", installed_schema),
        ("installed index", installed_index),
        ("installation digest seam", installation_digest),
        ("parameters", parameters),
        ("preconditions", preconditions),
    ] {
        assert!(
            source.contains("CanonicalDigestAlgorithmId::sha256()"),
            "{family} bypassed the admitted Foundational digest slot"
        );
        assert!(
            !source.contains("use sha2"),
            "{family} reintroduced a private SHA implementation"
        );
        assert!(
            !source.contains("hash_parts"),
            "{family} reintroduced an opaque string-part digest grammar"
        );
    }
    for (family, source) in [("package", package), ("admission", admission)] {
        assert!(
            source.contains("InstallationCanonicalIdentityBasis"),
            "{family} bypassed the bounded installation digest seam"
        );
        assert!(!source.contains("use sha2"));
        assert!(!source.contains("hash_parts"));
    }
    assert!(installed_authorization_policy.contains("InstallationCanonicalIdentityBasis"));
    assert!(!installed_authorization_policy.contains("use sha2"));
    assert!(bridge_authorization.contains("request.correspondence"));
    assert!(!bridge_authorization.contains("Sha256"));
    assert!(!live_open.contains("render_support_hex"));
    assert!(!live_open.contains("render_hex"));

    assert!(query_definition.contains("controls.disclosure-posture"));
    assert!(query_definition.contains("controls.disclosure-classification"));
    assert!(evidence_slot.contains("CanonicalDigestFrontDoor"));
    assert!(evidence_slot.contains("derive_canonical_digest"));
    for (family, source) in [("capability", capability), ("aftermath", aftermath)] {
        assert!(source.contains("domain_capability_scope_encoder"));
        assert!(!source.contains("use sha2"));
        assert!(!source.contains("hash_parts"));
        assert!(
            !source.contains("format!(\"{:?}\""),
            "{family} reintroduced debug-string identity"
        );
    }
}

#[test]
fn phase_six_warm_consumers_cannot_hide_hashing_behind_a_helper() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "../worth-query-execution/src/domain_computation/primary_graph/application_query",
        "../worth-query-execution/src/domain_computation/primary_graph/application_contribution/producer/demand/progression/admission/restoration.rs",
        "../worth-query-execution/src/domain_computation/authorization",
        "../worth-query-execution/src/domain_computation/primary_graph/application_attempt/idempotency_resolution.rs",
        "../worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_recomparison.rs",
        "../worth-query-execution/src/domain_computation/primary_graph/application_attempt/provider_execution",
        "../worth-query-execution/src/domain_computation/operation_binding/authority/application_binding.rs",
        "../worth-query-execution/src/domain_computation/provider_session/execution_attempt_identity.rs",
        "../worth-query-execution/src/domain_computation/provider_session/session_identity.rs",
        "../worth-query-execution/src/domain_computation/provider_session/graph_obligation",
        "../worth-query-execution/src/domain_computation/provider_session/attempt_evidence.rs",
        "../worth-query-admission/src/graph_obligation",
        "../worth-query-execution/src/domain_computation/managed_run/run_affinity",
        "../worth-query-execution/src/domain_computation/provider_session/protocol/plan_contract.rs",
        "../worth-query-execution/src/domain_computation/provider_session/protocol/provider_port.rs",
        "../worth-query-execution/src/domain_computation/provider_session/protocol/session_binding.rs",
        "../worth-query-execution/src/domain_computation/provider_session/decision_read_set/capture.rs",
        "../worth-query-execution/src/domain_computation/provider_session/decision_read_set/fact.rs",
        "../worth-query-execution/src/domain_computation/provider_session/provisional_attempt",
        "../../../../crates/worth-runtime-bridge/src/authorization",
        "../../../worth-query-bank-world/crates/bank-server/src/application_query",
        "../../../worth-query-bank-world/crates/bank-server/src/estate_capability_admission",
        "../../../worth-query-bank-world/crates/bank-server/src/ordinary/mutation",
    ] {
        assert_warm_path_has_no_hashing(&manifest.join(relative));
    }
}

fn assert_warm_path_has_no_hashing(path: &Path) {
    let is_source_meaning_owner =
        path.ends_with("application_query/observed_source/source_identity.rs");
    if path.is_dir() {
        for entry in std::fs::read_dir(path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let child = entry
                .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()))
                .path();
            if child
                .file_name()
                .is_some_and(|name| name == "tests" || name == "tests.rs")
            {
                continue;
            }
            assert_warm_path_has_no_hashing(&child);
        }
        return;
    }
    if path.extension().is_none_or(|extension| extension != "rs") {
        return;
    }
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    // Dynamic row and result-set witnesses require runtime evidence digests;
    // the named source-identity owner and its two commitment files must not
    // become a semantic contract hash seam. Each keeps its exact domain.
    for (owner, domain) in [
        (
            "application_query/observed_source/source_identity.rs",
            "worth-query:observed-result-set:v1",
        ),
        (
            "application_query/observed_source/source_identity/checkpoint_commitment.rs",
            "worth-query:checkpoint-observed-source:v2",
        ),
        (
            "application_query/observed_source/source_identity/runtime_commitment.rs",
            "worth-query:runtime-source-commitment:v1",
        ),
    ] {
        if path.ends_with(Path::new(owner)) {
            assert!(
                source.contains(domain),
                "missing runtime evidence domain {domain}"
            );
            assert!(!source.contains("prepare_canonical_basis_sequence"));
            return;
        }
    }
    if path.ends_with(Path::new(
        "application_query/observed_source/root_selection.rs",
    )) {
        // Native selection facts are observation-time evidence, not installed
        // semantic identities; keep this exception tied to its exact domain.
        assert!(source.contains("worth-query:root-selection-source:v2"));
        assert!(!source.contains("prepare_canonical_basis_sequence"));
        return;
    }
    for forbidden in [
        "hash_parts",
        "digest_hash_parts",
        "execution_digest",
        "admission_digest",
        "publication_digest",
        "checkpoint_source_identity",
        "Sha256",
        "sha2::",
        "prepare_canonical_basis_sequence",
        "for_sequence_with_budget",
        "canonicalization().digest().derive",
        ".render_support_hex()",
        ".render_hex()",
    ] {
        if is_source_meaning_owner && forbidden == "checkpoint_source_identity" {
            // The intern miss path is the named cold derivation boundary. The
            // rest of this file remains subject to every inline hashing rule.
            continue;
        }
        assert!(
            !source.contains(forbidden),
            "{} reintroduced warm-path canonical or digest work through `{forbidden}`",
            path.display()
        );
    }
}

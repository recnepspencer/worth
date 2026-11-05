use worth_testkit_compiler::{
    ProductionDiagnosticExpectation as Diagnostic, ProductionSemanticDenialContract as Contract,
};
mod contracts;

#[test]
fn bridge_phase_boundaries_are_compile_time_private() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/causal_envelope/*.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::production_denials(
        concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/writeback_contract/*.rs"
    ),
        &[Contract {
            fixture: "raw_external_authority_evidence_cannot_construct_writeback_identity.rs",
            errors: &[
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "WritebackEffectIdentityTag",
                    ],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "BridgeNamingAttachmentIdentity",
                    ],
                },
                Diagnostic::Code {
                    code: "E0599",
                    symbols: &[
                        "from_external_authority_evidence",
                        "BridgeExistingTruthBindingAuthoritativeIdentity",
                    ],
                },
            ],
        }],
    )
    .expect("production compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/relational_identity/*.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/crate_private_constructors/*.rs")).expect("prebuilt compiler denial");
}

#[test]
fn bridge_public_contracts_a_to_f_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[a-f]*.rs"
        ),
        &contracts::between(b'a', b'f'),
    )
    .expect("production compiler denial");
}

#[test]
fn bridge_public_contracts_g_to_r_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[g-r]*.rs"
        ),
        &contracts::between(b'g', b'r'),
    )
    .expect("production compiler denial");
}

#[test]
fn bridge_public_contracts_s_to_z_are_sealed() {
    worth_testkit_compiler::production_denials(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/[s-z]*.rs"
        ),
        &contracts::between(b's', b'z'),
    )
    .expect("production compiler denial");
}

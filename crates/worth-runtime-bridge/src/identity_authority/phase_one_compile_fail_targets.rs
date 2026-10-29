pub struct BridgeTruthIdentityPhaseOneCompileFailTarget {
    path: &'static str,
    forbidden_substitution: &'static str,
}

impl BridgeTruthIdentityPhaseOneCompileFailTarget {
    pub const fn new(path: &'static str, forbidden_substitution: &'static str) -> Self {
        Self {
            path,
            forbidden_substitution,
        }
    }

    pub const fn path(&self) -> &'static str {
        self.path
    }

    pub const fn forbidden_substitution(&self) -> &'static str {
        self.forbidden_substitution
    }
}

const BRIDGE_TRUTH_IDENTITY_PHASE_ONE_COMPILE_FAIL_TARGETS:
    &[BridgeTruthIdentityPhaseOneCompileFailTarget] = &[
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/projection_cannot_satisfy_truth_authority.rs"),
        "projection identity cannot satisfy bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/digest_cannot_satisfy_truth_authority.rs"),
        "digest evidence cannot satisfy bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/external_token_cannot_satisfy_truth_authority.rs"),
        "external token cannot satisfy bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/bridged_cannot_satisfy_current_truth_authority.rs"),
        "boundary-bridged identity requires owner readmission",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/wrong_kind_cannot_satisfy_truth_family.rs"),
        "wrong identity kind cannot satisfy another bridge truth family",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/raw_text_cannot_satisfy_truth_authority.rs"),
        "raw text cannot satisfy bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneCompileFailTarget::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-court-runtime-bridge/tests/worth_court_runtime_bridge/sources/worth_runtime_bridge/ui/bridge_truth_identity/retained_evidence_cannot_rebuild_from_text.rs"),
        "retained evidence cannot rebuild authority from text",
    ),
];

pub const fn bridge_truth_identity_phase_one_compile_fail_targets(
) -> &'static [BridgeTruthIdentityPhaseOneCompileFailTarget] {
    BRIDGE_TRUTH_IDENTITY_PHASE_ONE_COMPILE_FAIL_TARGETS
}

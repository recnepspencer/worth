pub struct BridgeTruthIdentityPhaseOneRootBreakTarget {
    api: &'static str,
    required_restriction: &'static str,
}

impl BridgeTruthIdentityPhaseOneRootBreakTarget {
    pub const fn new(api: &'static str, required_restriction: &'static str) -> Self {
        Self {
            api,
            required_restriction,
        }
    }

    pub const fn api(&self) -> &'static str {
        self.api
    }

    pub const fn required_restriction(&self) -> &'static str {
        self.required_restriction
    }
}

const BRIDGE_TRUTH_IDENTITY_PHASE_ONE_ROOT_BREAK_TARGETS:
    &[BridgeTruthIdentityPhaseOneRootBreakTarget] = &[
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::from_external_authority",
        "accept only BridgeTruthExternalIdentityToken; raw/projection/digest text cannot enter",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::from_query_evidence_identity",
        "treat Query evidence as evidence, not bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::as_str",
        "rename or quarantine as terminal reporting projection",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::is_empty",
        "keep evidence predicates crate-private so public bridge evidence is not string-like",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::Display",
        "prevent formatting from reconstructing bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentityEvidence::AsRef<str>",
        "prevent generic string consumers from receiving bridge truth authority",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentity::<Tag>::new",
        "witness-gate or authority-category gate bridge truth construction",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentity::<Tag>::as_str",
        "rename or quarantine as terminal reporting projection",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentity::<Tag>::from_reference_evidence",
        "rebuild only from typed retained evidence, never text",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "BridgeIdentity::<Tag>::bridge_trust_boundary",
        "allowed boundary export for current bridge truth only when the source tag implements a tag-specific bridge identity family kind",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "relational_source::bridge_snapshot_identity_for_commit",
        "mint bridge snapshot identity from relational source-truth authority only",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "relational_source::bridge_snapshot_identity_for_handle",
        "mint bridge snapshot identity from relational source-truth authority only",
    ),
    BridgeTruthIdentityPhaseOneRootBreakTarget::new(
        "relational_source::RuntimeBridgeRelationalSource",
        "carry relational source-truth authority only from a change receipt the live Relational runtime minted",
    ),
];

pub const fn bridge_truth_identity_phase_one_root_break_targets(
) -> &'static [BridgeTruthIdentityPhaseOneRootBreakTarget] {
    BRIDGE_TRUTH_IDENTITY_PHASE_ONE_ROOT_BREAK_TARGETS
}

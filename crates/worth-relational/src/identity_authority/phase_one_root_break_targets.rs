pub struct RelationalSourceTruthIdentityPhaseOneRootBreakTarget {
    api: &'static str,
    required_restriction: &'static str,
}

impl RelationalSourceTruthIdentityPhaseOneRootBreakTarget {
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

const RELATIONAL_SOURCE_TRUTH_IDENTITY_PHASE_ONE_ROOT_BREAK_TARGETS:
    &[RelationalSourceTruthIdentityPhaseOneRootBreakTarget] =
    &[RelationalSourceTruthIdentityPhaseOneRootBreakTarget::new(
        "runtime::RelationalRuntime::mint_change_receipt",
        "mint a change receipt only from a commit the live Relational runtime selected",
    )];

pub const fn relational_source_truth_identity_phase_one_root_break_targets(
) -> &'static [RelationalSourceTruthIdentityPhaseOneRootBreakTarget] {
    RELATIONAL_SOURCE_TRUTH_IDENTITY_PHASE_ONE_ROOT_BREAK_TARGETS
}

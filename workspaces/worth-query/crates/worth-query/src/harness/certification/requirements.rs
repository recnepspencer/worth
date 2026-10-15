#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RequiredAssertionClass {
    Equality,
    Inequality,
    TypedFailure,
    ZeroResidue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuiteRequirements {
    pub suite_name: &'static str,
    pub required_canonical_rows: &'static [&'static str],
    pub required_rejection_rows: &'static [&'static str],
    pub required_assertion_classes: &'static [RequiredAssertionClass],
    pub missing_rows_block_full_spec: bool,
    pub missing_rows_block_offline_ready: bool,
}

pub fn milestone_three_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Planner / Executor / Binding Parity Test",
        required_canonical_rows: &[
            "direct-runtime-plan-parity",
            "replanned-runtime-parity",
            "type-bound-runtime-parity",
            "runtime-basis-repeatability",
            "identity-bearing-binding-difference",
            "basis-difference",
            "route-semantic-difference",
        ],
        required_rejection_rows: &[
            "unsupported-backend-route",
            "unsupported-fallback-shape",
            "binding-fulfillment-conflict",
            "snapshot-basis-resolution-failure",
        ],
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_five_point_two_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Preview Session Basis And Promotion Parity Test",
        required_canonical_rows:
            crate::harness::preview_certification::PREVIEW_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows:
            crate::harness::preview_certification::PREVIEW_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_five_point_four_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Structural Correspondence And Historical Materialization Path Test",
        required_canonical_rows: crate::harness::correspondence_history_certification::
            CORRESPONDENCE_HISTORY_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows: crate::harness::correspondence_history_certification::
            CORRESPONDENCE_HISTORY_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_nine_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Policy And Tenant Context Admission Test",
        required_canonical_rows:
            crate::harness::milestone_nine_certification::MILESTONE_NINE_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows:
            crate::harness::milestone_nine_certification::MILESTONE_NINE_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_nine_one_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Query Subscription Declaration And Lowering Parity Test",
        required_canonical_rows: crate::harness::milestone_nine_one_certification::
            MILESTONE_NINE_ONE_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows: crate::harness::milestone_nine_one_certification::
            MILESTONE_NINE_ONE_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_nine_five_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Milestone 9.5 Debt-Close Hostile Certification Matrix",
        required_canonical_rows: crate::harness::milestone_nine_five_hostile_matrix::
            MILESTONE_NINE_FIVE_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows: crate::harness::milestone_nine_five_hostile_matrix::
            MILESTONE_NINE_FIVE_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_nine_two_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Subscription Lifecycle Sharing And Preview Parity Test",
        required_canonical_rows: crate::harness::milestone_nine_two_certification::
            MILESTONE_NINE_TWO_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows: crate::harness::milestone_nine_two_certification::
            MILESTONE_NINE_TWO_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub fn milestone_nine_three_requirements() -> SuiteRequirements {
    SuiteRequirements {
        suite_name: "Query Subscription Bridge Parity And Diagnostic Sufficiency Test",
        required_canonical_rows: crate::harness::milestone_nine_three_certification::
            MILESTONE_NINE_THREE_REQUIRED_CANONICAL_ROW_NAMES,
        required_rejection_rows: crate::harness::milestone_nine_three_certification::
            MILESTONE_NINE_THREE_REQUIRED_REJECTION_ROW_NAMES,
        required_assertion_classes: &[
            RequiredAssertionClass::Equality,
            RequiredAssertionClass::Inequality,
            RequiredAssertionClass::TypedFailure,
            RequiredAssertionClass::ZeroResidue,
        ],
        missing_rows_block_full_spec: true,
        missing_rows_block_offline_ready: true,
    }
}

pub mod adapters;
pub mod aspect_contracts;
pub mod bridge;
pub mod builder_bootstrap;
pub mod common_bootstrap;
pub mod existing_truth_adapter;
pub mod external_row;
pub mod profiles;
pub mod relational_merge;
pub mod state;

pub use std::cell::RefCell;
pub use std::rc::Rc;

pub use worth_foundational::facade::AspectValue;
pub use worth_query::facade::runtime::RuntimeWorldBranchBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldCustodyBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldHistoryBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldObservationBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldPublicationBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldRecoveryBudgetInstallation;
pub use worth_query::facade::runtime::RuntimeWorldRetentionBudgetInstallation;
pub use worth_query::facade::runtime::WorthQueryAspectTouch;
pub use worth_query::facade::runtime::WorthQueryConditionalEvaluationCacheBudget;
pub use worth_query::facade::runtime::WorthQueryConditionalExecutionResources;
pub use worth_query::facade::runtime::WorthQueryExistingTruthTargetBinding;
pub use worth_query::facade::runtime::WorthQueryProductWorldClock;
pub use worth_query::facade::runtime::WorthQueryProductWorldResources;
pub use worth_query::facade::runtime::WorthQueryRuntime;
pub use worth_query::facade::runtime::WorthQueryRuntimeBuilder;
pub use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;

pub use self::adapters::PublicInspectorEvidenceAdapter;
pub use self::adapters::PublicPreviewBasisAdapter;
pub use self::adapters::PublicSchemaAdapter;
pub use self::adapters::PublicSignalSinkAdapter;
pub use self::adapters::PublicSnapshotIdentityAdapter;
pub use self::adapters::PublicSourceAdapter;
pub use self::adapters::PublicSubscriptionActivationAdapter;
pub use self::adapters::PublicWriteAuthorityAdapter;
pub use self::aspect_contracts::public_bridge_aspect_contracts;
pub use self::existing_truth_adapter::PublicExistingTruthVerificationAdapter;
pub use self::state::PublicBridgeRuntimeState;
pub use self::state::PublicExistingTruthKey;

pub type SharedRuntimeState = Rc<RefCell<PublicBridgeRuntimeState>>;

pub use self::profiles::public_graph_support_profile;
pub use self::relational_merge::public_relational_merge_runtime;
pub use common_bootstrap::public_bridge_runtime_bootstrap_invocation_count;
pub use common_bootstrap::reset_public_bridge_runtime_bootstrap_invocations;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicBridgeRuntimeBootstrapPath {
    Common,
    Builder,
}

pub struct PublicBridgeRuntimeHarness {
    pub state: SharedRuntimeState,
}
pub struct PublicBridgeRuntimeBootstrapBuilder {
    pub state: SharedRuntimeState,
}
pub struct PublicBridgeRuntimeBootstrapWithSupportProfile {
    pub state: SharedRuntimeState,
    pub support_profile: WorthQueryRuntimeSupportProfile,
}

thread_local! {
    static BOOTSTRAP_INVOCATIONS: RefCell<[usize; 2]> = const { RefCell::new([0; 2]) };
}

pub fn record_public_bridge_runtime_bootstrap_invocation(path: PublicBridgeRuntimeBootstrapPath) {
    BOOTSTRAP_INVOCATIONS.with(|counts| {
        counts.borrow_mut()[bootstrap_index(path)] += 1;
    });
}

pub fn public_product_resources() -> WorthQueryConditionalExecutionResources {
    WorthQueryConditionalExecutionResources::new(
        WorthQueryConditionalEvaluationCacheBudget::bounded(128, 8 * 1024 * 1024)
            .expect("public bridge tests require a nonempty Query conditional budget"),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
            maximum_retained_slots: Some(128),
            maximum_retained_bytes: Some(512 * 1024 * 1024),
            maximum_attempt_visits: None,
        },
    )
}

pub fn public_product_world_resources() -> WorthQueryProductWorldResources {
    public_product_world_resources_with_branch_limit(128)
}

pub fn public_product_world_resources_with_branch_limit(
    live_product_branches: u64,
) -> WorthQueryProductWorldResources {
    // Serial requests retain accounting without a fixture-imposed byte or work ceiling.
    let execution_policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
    );
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        WorthQueryProductWorldClock::start(),
        worth_query_execution::facade::runtime::WorthQueryInvalidationResources::new(),
        execution_policy,
    )
    .expect("the public test Product World resources are valid")
}

pub fn bootstrap_index(path: PublicBridgeRuntimeBootstrapPath) -> usize {
    match path {
        PublicBridgeRuntimeBootstrapPath::Common => 0,
        PublicBridgeRuntimeBootstrapPath::Builder => 1,
    }
}

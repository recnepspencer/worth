mod adapters;
mod aspect_contracts;
mod bridge;
mod builder_bootstrap;
mod common_bootstrap;
mod existing_truth_adapter;
mod external_row;
mod profiles;
mod relational_merge;
mod state;

use std::cell::RefCell;
use std::rc::Rc;

use worth_foundational::facade::AspectValue;
use worth_query::facade::runtime::{
    RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
    RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
    RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
    RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
    WorthQueryAspectTouch, WorthQueryConditionalEvaluationCacheBudget,
    WorthQueryConditionalExecutionResources, WorthQueryExistingTruthTargetBinding,
    WorthQueryProductWorldClock, WorthQueryProductWorldResources, WorthQueryRuntime,
    WorthQueryRuntimeBuilder, WorthQueryRuntimeSupportProfile,
};

use self::adapters::{
    PublicInspectorEvidenceAdapter, PublicPreviewBasisAdapter, PublicSchemaAdapter,
    PublicSignalSinkAdapter, PublicSnapshotIdentityAdapter, PublicSourceAdapter,
    PublicSubscriptionActivationAdapter, PublicWriteAuthorityAdapter,
};
use self::aspect_contracts::public_bridge_aspect_contracts;
use self::existing_truth_adapter::PublicExistingTruthVerificationAdapter;
use self::state::{PublicBridgeRuntimeState, PublicExistingTruthKey};

type SharedRuntimeState = Rc<RefCell<PublicBridgeRuntimeState>>;

pub use self::profiles::public_graph_support_profile;
pub use self::relational_merge::public_relational_merge_runtime;
pub use common_bootstrap::{
    public_bridge_runtime_bootstrap_invocation_count,
    reset_public_bridge_runtime_bootstrap_invocations,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicBridgeRuntimeBootstrapPath {
    Common,
    Builder,
}

pub struct PublicBridgeRuntimeHarness {
    state: SharedRuntimeState,
}
pub struct PublicBridgeRuntimeBootstrapBuilder {
    state: SharedRuntimeState,
}
pub struct PublicBridgeRuntimeBootstrapWithSupportProfile {
    state: SharedRuntimeState,
    support_profile: WorthQueryRuntimeSupportProfile,
}

thread_local! {
    static BOOTSTRAP_INVOCATIONS: RefCell<[usize; 2]> = const { RefCell::new([0; 2]) };
}

fn record_public_bridge_runtime_bootstrap_invocation(path: PublicBridgeRuntimeBootstrapPath) {
    BOOTSTRAP_INVOCATIONS.with(|counts| {
        counts.borrow_mut()[bootstrap_index(path)] += 1;
    });
}

fn public_product_resources() -> WorthQueryConditionalExecutionResources {
    WorthQueryConditionalExecutionResources::new(
        WorthQueryConditionalEvaluationCacheBudget::bounded(128, 8 * 1024 * 1024)
            .expect("public bridge tests require a nonempty Query conditional budget"),
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget {
            maximum_retained_slots: 128,
            maximum_retained_bytes: 512 * 1024 * 1024,
            maximum_attempt_visits: 8_000_000,
        },
    )
}

pub fn public_product_world_resources() -> WorthQueryProductWorldResources {
    public_product_world_resources_with_branch_limit(128)
}

pub fn public_product_world_resources_with_branch_limit(
    live_product_branches: u64,
) -> WorthQueryProductWorldResources {
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
    )
    .expect("the public test Product World resources are valid")
}

fn bootstrap_index(path: PublicBridgeRuntimeBootstrapPath) -> usize {
    match path {
        PublicBridgeRuntimeBootstrapPath::Common => 0,
        PublicBridgeRuntimeBootstrapPath::Builder => 1,
    }
}

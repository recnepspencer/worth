//! Every branch and mutation enters the public serving Home runtime.
use super::opening::Home;
use std::sync::Arc;
use worth_query_host::facade::{primary_graph, product, runtime};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::{
    contribution_installation::*, world::program_amendment::commit_definition_on_program_lane,
};

pub(super) fn branches(home: &Home) -> Vec<product::WorthQueryProductBranch> {
    let app = home.runtime();
    let main = app.current_world();
    let mut result = vec![main];
    for _ in 0..2 {
        let answer = app
            .with_host_advancement(|phase| {
                app.branches()
                    .fork(main)
                    .components(|components| components.fork_relational().fork_signal())
                    .create(&phase)
            })
            .unwrap();
        let branch = match answer {
            Ok(branch) => branch,
            Err(product::WorthQueryProductBranchCreateError::ProductUnpublished(recovery)) => {
                finish_fork(home, recovery)
            }
            Err(cause) => panic!("public Home fork refused: {cause:?}"),
        };
        result.push(branch);
    }
    result
}
/// Retry only the retained original creation, never the owner effects.
fn finish_fork(
    home: &Home,
    mut original: product::WorthQueryProductBranchCreationRecovery,
) -> product::WorthQueryProductBranch {
    let app = home.runtime();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "original Home fork did not settle"
        );
        let answer = app
            .with_host_advancement(|phase| original.continue_in_advancement(&phase))
            .unwrap();
        match answer {
            Ok(product::WorthQueryProductBranchCreationContinuation::Performed(branch)) => {
                return branch
            }
            Ok(product::WorthQueryProductBranchCreationContinuation::NoEffect(cause)) => {
                panic!("original fork refused: {cause:?}")
            }
            Ok(product::WorthQueryProductBranchCreationContinuation::ReleasedOriginal(cause)) => {
                panic!("original fork released: {cause:?}")
            }
            Err(failure) => {
                assert!(
                    matches!(
                        failure.denial(),
                        product::WorthQueryProductBranchRecoveryDenial::Busy
                    ),
                    "original fork continuation: {failure:?}"
                );
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}
pub(super) fn mixed(home: &Home, branch: product::WorthQueryProductBranch, ordinal: u8) {
    let receipt = commit_mixed(home, branch, ordinal)
        .require_committed()
        .unwrap_or_else(|outcome| panic!("the mixed definition commit must land: {outcome:?}"));
    assert_eq!(receipt.product_branch(), branch);
    drop(receipt);
}
/// One mixed conditional-definition commit under its own idempotency key,
/// answered exactly as Query answers that key.
pub(super) fn commit_mixed(
    home: &Home,
    branch: product::WorthQueryProductBranch,
    ordinal: u8,
) -> primary_graph::WorthQueryApplicationCommitOutcome {
    let app = home.runtime();
    let installed = app.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    commit_definition_on_program_lane(
        app,
        &installed.clock,
        &installed.invariant,
        selected,
        &format!("authored-{ordinal}"),
        Arc::new(provider),
        ordinal,
    )
}
pub(super) fn definition(home: &Home, branch: product::WorthQueryProductBranch) {
    let app = home.runtime();
    let installed = app.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    let outcome = selected
        .publish_conditional_definition(
            &installed.clock,
            Arc::new(provider),
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    assert!(matches!(
        outcome,
        primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
    ));
}
#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct State {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub lifecycle: String,
    pub input: String,
    pub signal_generation: u64,
}
pub(super) fn observe(home: &Home, branches: &[product::WorthQueryProductBranch]) -> Vec<State> {
    let app = home.runtime();
    branches
        .iter()
        .map(|branch| {
            let row = publication_limit::read_intent_on_branch(app, *branch);
            let selected = app
                .with_host_advancement(|phase| app.on_branch(*branch).select(&phase))
                .unwrap()
                .unwrap();
            let signal_generation = selected.product().signal_generation_for_test();
            State {
                identity: row.identity,
                revision: row.revision,
                due: row.due,
                lifecycle: row.lifecycle,
                input: row.input,
                signal_generation,
            }
        })
        .collect()
}

/// All selectors are freshly issued by the Home-admitted World inventory.
pub(super) fn installed(home: &Home) -> Vec<product::WorthQueryProductBranch> {
    let app = home.runtime();
    let inventory = app
        .with_host_advancement(|phase| app.branches().inventory(&phase))
        .unwrap()
        .unwrap();
    let mut branches = inventory.iter().collect::<Vec<_>>();
    branches.sort_by_key(|branch| branch.occurrence_ordinal());
    branches
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in super::super::super) struct SignalOracleDetails {
    pub signal_generation: u64,
    pub world_reference_generation: u64,
    pub installed_conditional_definition_count: usize,
    pub installed_conditional_target_ref_count: usize,
    pub owned_signal_active_node_count: usize,
    pub head_attempt_ordinal: Option<u64>,
    pub head_native_commit_id: Option<u64>,
    pub head_native_version_id: Option<u64>,
    pub head_relational_changed: bool,
    pub head_signal_changed: bool,
    pub head_signal_movement: Option<String>,
}

/// Durable Signal/Bridge generations, installed conditional counts, and head
/// World occurrence coordinates observed via the authorized Signal testkit boundary.
pub(in super::super::super) fn signal_oracle_details(
    home: &Home,
    branch: product::WorthQueryProductBranch,
) -> SignalOracleDetails {
    use worth_runtime_world::facade::{
        CompositeComponentChangePosture as Posture, CompositeCommitProvenance as Prov,
    };
    let app = home.runtime();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let prod = selected.product();
    let commit_ord = prod.selected_commit().ordinal();
    let signal_generation = prod.signal_generation_for_test();
    let world_reference_generation = prod.world_reference_generation_for_test();
    drop(selected);
    let rt = app.runtime();
    let ancestry = rt.application_oracle_ancestry_for_test();
    let occ = ancestry
        .occurrences
        .iter()
        .find(|e| e.identity.ordinal() == commit_ord)
        .expect("selected commit must exist in durable World ancestry");
    let head_attempt_ordinal = match &occ.provenance {
        Prov::Publication(id) => Some(id.ordinal()),
        Prov::Bootstrap(_) => None,
    };
    let (head_native_commit_id, head_native_version_id) = match &occ.relational_publication {
        Some(p) => (Some(p.commit_id().0), Some(p.version_id().as_u64())),
        None => (None, None),
    };
    SignalOracleDetails {
        signal_generation,
        world_reference_generation,
        installed_conditional_definition_count: rt.installed_conditional_definition_count_for_test(),
        installed_conditional_target_ref_count: rt
            .installed_conditional_target_reference_count_for_test(),
        owned_signal_active_node_count: rt.owned_signal_active_node_count_for_test(),
        head_attempt_ordinal,
        head_native_commit_id,
        head_native_version_id,
        head_relational_changed: occ.relational_change == Posture::Published,
        head_signal_changed: occ.signal_change == Posture::Published,
        head_signal_movement: occ.signal_movement.map(|m| format!("{m:?}")),
    }
}

pub(in super::super::super) fn product_generations(
    home: &Home,
    branch: product::WorthQueryProductBranch,
) -> (u64, u64) {
    let d = signal_oracle_details(home, branch);
    (d.signal_generation, d.world_reference_generation)
}
/// These are observations for comparison, never reopen identities or authority.
pub(super) fn frontiers(
    home: &Home,
    branches: &[product::WorthQueryProductBranch],
) -> Vec<(u64, u64, u64)> {
    let app = home.runtime();
    branches
        .iter()
        .map(|branch| {
            let selected = app
                .with_host_advancement(|phase| app.on_branch(*branch).select(&phase))
                .unwrap()
                .unwrap();
            let product = selected.product();
            (
                product
                    .relational_basis_descriptor()
                    .truth_version()
                    .as_u64(),
                product.signal_generation_for_test(),
                product.world_reference_generation_for_test(),
            )
        })
        .collect()
}

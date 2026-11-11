//! Every branch and mutation enters the public serving Home runtime.
use super::opening::Home;
use std::sync::Arc;
use worth_query_host::facade::{primary_graph, product, runtime};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::{
    contribution_installation::*, world::program_amendment::publish_definition_on_program_lane,
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
    let app = home.runtime();
    let installed = app.conditional::<TemporalConditional>().unwrap();
    let installed = installed.as_ref().as_ref().unwrap();
    let selected = app
        .with_host_advancement(|phase| app.on_branch(branch).select(&phase))
        .unwrap()
        .unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    let receipt = publish_definition_on_program_lane(
        app,
        &installed.clock,
        &installed.invariant,
        selected,
        &format!("authored-{ordinal}"),
        Arc::new(provider),
        ordinal,
    );
    assert_eq!(receipt.product_branch(), branch);
    drop(receipt);
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

//! Real edits, at every budget, compared to equally sized debt-free twins.
use super::super::super::{BoundOutputSource, PreparedOutputRootKind, SourceCustody};
use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support;
use worth_runtime_world::facade::CompositeCommitIdentity;

struct Context {
    completion: super::super::super::ReadyCompletion,
    commit: CompositeCommitIdentity,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    owner: SourceInvalidationOwner,
}

impl Context {
    fn new() -> Self {
        let receipt = recoverable_commit_support::committed_recoverable_application();
        let DemandState::Output(output) = closed_retirement::ready() else {
            panic!("Ready fixture")
        };
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else {
            panic!("Ready fixture")
        };
        Self {
            completion,
            commit: receipt
                .committed_product_publication()
                .composite_commit()
                .clone(),
            scope: source_custody::real_scope_and_occurrence().0,
            owner: invalidation_owner(),
        }
    }

    fn make(&self, entry: u8, seed: u64, debt: bool) -> Twin {
        // Both sides have the same map cardinality and prerequisite edges.
        // The foreign pending source in the clean twin has no cleanup debt.
        let (registry, interest) = fixture(seed * 2, &self.completion, entry == 0 || entry == 4);
        let foreign = key("foreign-completed", seed * 2 + 1, 200);
        {
            let mut state = registry.state.lock().unwrap();
            let phase = if debt {
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "completed foreign work",
                ))
            } else {
                DemandState::Admitted
            };
            let mut row = record(occurrence(), phase, 0);
            row.source_commits.push(self.commit.clone());
            state.records.insert(foreign.clone(), row);
            state.source_custody.insert(
                self.commit.clone(),
                SourceCustody {
                    occurrence: occurrence(),
                    root_kind: PreparedOutputRootKind::Discovered(
                        crate::equality_type_id::EqualityTypeId::of::<()>(),
                    ),
                    source: None,
                    discovery: None,
                    bound_sources: Some(vec![BoundOutputSource {
                        scope: self.scope,
                        identity: foreign.source.clone(),
                    }]),
                    consumed_sources: vec![foreign.source.clone()],
                    retired_sources: Vec::new(),
                    retired: None,
                    token_count: 0,
                    cleanup: Default::default(),
                    completed: debt,
                },
            );
            if debt {
                state.note_custody(&self.commit);
                // A retained terminal note after its last claim left: the
                // claimant pass, member release and custody prune remain owed.
                state.note_closed_row(&foreign);
            }
            if entry == 3 {
                state.records.get_mut(&interest.key).unwrap().state =
                    DemandState::Failed(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::NoEffect,
                        "terminal own work",
                    ));
                assert!(state.defer_terminal_cleanup(&Arc::new(interest.key.clone()), 0));
            }
        }
        Twin {
            registry,
            interest,
            foreign,
        }
    }

    fn own_value(&self, twin: &Twin) -> RegistryValue {
        let mut result = value(&twin.registry.state.lock().unwrap());
        result.rows.retain(|row| row.key != twin.foreign);
        result
            .custodians
            .retain(|entry| entry.commit != self.commit);
        result
    }
}

struct Twin {
    registry: WorthQueryOutputDemandRegistry,
    interest: WorthQueryOutputDemandInterest,
    foreign: WorthQueryOutputDemandKey,
}

#[derive(Debug, PartialEq, Eq)]
enum Answer {
    Released(bool),
    Reclaimed,
    Drained,
    Joined(WorthQueryOutputDemandKey),
}

fn run(
    entry: u8,
    twin: &Twin,
    context: &Context,
    paid: &mut InvalidationEditAdmission,
) -> Result<(Answer, Option<WorthQueryOutputDemandInterest>), WorthQueryOutputDemandDenialKind> {
    let registry = &twin.registry;
    let result = match entry {
        0 => registry
            .release_joined_ready(&twin.interest, paid)
            .map(|released| (Answer::Released(released), None)),
        1 => registry
            .reclaim_cached_rows(1, paid)
            .map(|()| (Answer::Reclaimed, None)),
        2 => registry
            .reclaim_unheld_observation(paid)
            .map(|released| (Answer::Released(released), None)),
        3 => registry
            .drain_terminal_cleanup_admitted(&context.owner, paid)
            .map(|()| (Answer::Drained, None)),
        4 => registry
            .admit(
                twin.interest.key.clone(),
                None,
                context.scope,
                occurrence(),
                super::super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                paid,
            )
            .map(|owner| (Answer::Joined(owner.key.clone()), Some(owner))),
        _ => panic!("unknown entry"),
    };
    result.map_err(|denial| denial.kind())
}

fn invalidation_owner() -> SourceInvalidationOwner {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    SourceInvalidationOwner::new(
        WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
        )
        .unwrap(),
        1,
    )
}

fn check(entry: u8) {
    let context = Context::new();
    // One real edit per door, with every work ceiling. Shifting generations
    // alone would repeat this identical three-row shape and its prices.
    let seed = 0;
    let complete = context.make(entry, seed, false);
    let mut full = admission(1_000_000);
    let (expected_answer, _owner) = run(entry, &complete, &context, &mut full).unwrap();
    let expected = context.own_value(&complete);
    assert_ne!(
        expected,
        context.own_value(&context.make(entry, seed, false))
    );
    // Find the debt-free entry's own ceiling; optional cleanup after the
    // edit may spend slack, so ample charged work is only the upper bound.
    let mut own_price = None;
    for k in 0..=full.charged_work() {
        let clean = context.make(entry, seed, false);
        let noted = context.make(entry, seed, true);
        let clean_before = value(&clean.registry.state.lock().unwrap());
        let noted_before = value(&noted.registry.state.lock().unwrap());
        let mut clean_paid = admission(k);
        let mut noted_paid = admission(k);
        match run(entry, &clean, &context, &mut clean_paid) {
            Err(stop) => {
                assert_eq!(stop, WorthQueryOutputDemandDenialKind::WorkBudgetExceeded);
                assert_eq!(
                    value(&clean.registry.state.lock().unwrap()),
                    clean_before,
                    "clean stop entry {entry}, seed {seed}, k {k}"
                );
                let stopped = run(entry, &noted, &context, &mut noted_paid);
                assert!(matches!(
                    stopped,
                    Err(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded)
                ));
                assert_eq!(noted_paid.charged_work(), clean_paid.charged_work());
                assert_eq!(value(&noted.registry.state.lock().unwrap()), noted_before);
                let (retried, _owner) =
                    run(entry, &noted, &context, &mut admission(1_000_000)).unwrap();
                assert_eq!(retried, expected_answer);
                assert_eq!(
                    context.own_value(&noted),
                    expected,
                    "retry entry {entry}, seed {seed}, k {k}"
                );
            }
            Ok((answer, _clean_owner)) => {
                assert_eq!(answer, expected_answer);
                assert_eq!(context.own_value(&clean), expected);
                let (answer, _noted_owner) = run(entry, &noted, &context, &mut noted_paid).unwrap();
                assert_eq!(answer, expected_answer);
                assert_eq!(context.own_value(&noted), expected);
                assert_eq!(clean_paid.charged_work(), k);
                assert_eq!(noted_paid.charged_work(), k);
                let after = value(&noted.registry.state.lock().unwrap());
                assert_eq!(
                    after.rows.iter().find(|row| row.key == noted.foreign),
                    noted_before
                        .rows
                        .iter()
                        .find(|row| row.key == noted.foreign)
                );
                assert_eq!(after.custodians, noted_before.custodians);
                {
                    let state = noted.registry.state.lock().unwrap();
                    assert!(state.records[&noted.foreign].closed_cleanup.queued());
                    assert!(state.source_custody[&context.commit].cleanup.queued);
                }
                // The own reclamation ceiling is deliberately zero in the
                // fixture; the next request has retention as well as work.
                noted.registry.state.lock().unwrap().required_budget_bytes = 8 * 1024 * 1024;
                let _reader = noted
                    .registry
                    .admit(
                        key("foreign-reader", seed * 2 + 2, 200),
                        None,
                        context.scope,
                        occurrence(),
                        super::super::super::DemandAdmissionKind::Ordinary,
                        None,
                        None,
                        &mut admission(1_000_000),
                    )
                    .unwrap();
                let state = noted.registry.state.lock().unwrap();
                assert!(!state.records.contains_key(&noted.foreign));
                assert!(!state.source_custody.contains_key(&context.commit));
                own_price = Some(k);
                break;
            }
        }
    }
    assert!(own_price.is_some(), "entry {entry}, seed {seed}");
}

#[test]
fn registry_eager_foreign_debt_real_joined_edit_matches_its_twin_at_every_budget() {
    check(0);
}
#[test]
fn registry_eager_foreign_debt_real_cache_edit_matches_its_twin_at_every_budget() {
    check(1);
}
#[test]
fn registry_eager_foreign_debt_real_unheld_edit_matches_its_twin_at_every_budget() {
    check(2);
}
#[test]
fn registry_eager_foreign_debt_real_terminal_edit_matches_its_twin_at_every_budget() {
    check(3);
}
#[test]
fn registry_eager_foreign_debt_real_admit_edit_matches_its_twin_at_every_budget() {
    check(4);
}

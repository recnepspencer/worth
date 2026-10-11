//! Every pre-effect work refusal can be retried to the same registry value.
use super::super::WorthQueryOutputCheckpoint;
use super::*;
mod completed_custody;

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 8 * 1024 * 1024,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct RowValue {
    key: WorthQueryOutputDemandKey,
    state: u8,
    interests: usize,
    required: usize,
    framework: usize,
    prerequisites: Vec<WorthQueryOutputDemandKey>,
    prerequisite_slots: usize,
    settlement_slots: usize,
    queued: bool,
    source_commits: Vec<worth_runtime_world::facade::CompositeCommitIdentity>,
}

#[derive(Debug, PartialEq, Eq)]
struct RegistryValue {
    rows: Vec<RowValue>,
    members: Vec<WorthQueryOutputDemandKey>,
    required: usize,
    obligations: usize,
    custody: usize,
    custodians: Vec<CustodyValue>,
}

#[derive(Debug, PartialEq, Eq)]
struct CustodyValue {
    commit: worth_runtime_world::facade::CompositeCommitIdentity,
    bound: Option<Vec<super::super::BoundOutputSource>>,
    consumed: Vec<super::super::SourceEpoch>,
    completed: bool,
    tokens: usize,
}

fn value(state: &DemandRegistryState) -> RegistryValue {
    RegistryValue {
        rows: state
            .records
            .iter()
            .map(|(key, row)| {
                assert!(row.settlements.is_empty());
                assert!(row.performed_obligations.is_empty());
                assert!(row.checkpoint_prerequisites.is_none());
                assert!(row.performed_source.is_none());
                assert!(row.held_successor.is_none());
                let state = match &row.state {
                    DemandState::Admitted => 0,
                    DemandState::Failed(denial) => match denial.kind() {
                        WorthQueryOutputDemandDenialKind::Superseded => 1,
                        WorthQueryOutputDemandDenialKind::NoEffect => 4,
                        _ => panic!("unexpected failure in the mutation oracle"),
                    },
                    DemandState::Output(output) => {
                        assert!(matches!(
                            output.checkpoint,
                            Some(WorthQueryOutputCheckpoint::Ready(_))
                        ));
                        match &output.advancement {
                            super::super::WorthQueryOutputAdvancement::Idle => 2,
                            super::super::WorthQueryOutputAdvancement::Stopped {
                                denial, ..
                            } => {
                                assert_eq!(
                                    denial.kind(),
                                    WorthQueryOutputDemandDenialKind::Superseded
                                );
                                3
                            }
                            _ => panic!("unexpected output phase in the oracle"),
                        }
                    }
                    _ => panic!("unexpected state in the mutation oracle"),
                };
                RowValue {
                    key: key.clone(),
                    state,
                    interests: row.interests,
                    required: row.required_interests,
                    framework: row.framework_required_count,
                    prerequisites: row
                        .prerequisites
                        .iter()
                        .map(|key| key.as_ref().clone())
                        .collect(),
                    prerequisite_slots: row.prerequisites.capacity(),
                    settlement_slots: row.settlements.capacity(),
                    queued: row.pending_cleanup_queued,
                    source_commits: row.source_commits.clone(),
                }
            })
            .collect(),
        members: state
            .required_keys
            .iter()
            .map(|key| key.as_ref().clone())
            .collect(),
        required: state.required_reserved_bytes,
        obligations: state.obligation_reserved_bytes,
        custody: state
            .required_custody_retained_bytes
            .load(std::sync::atomic::Ordering::Acquire),
        custodians: state
            .source_custody
            .iter()
            .map(|(commit, custody)| {
                assert!(custody.source.is_none());
                assert!(custody.discovery.is_none());
                assert!(custody.retired.is_none());
                assert!(custody.retired_sources.is_empty());
                CustodyValue {
                    commit: commit.clone(),
                    bound: custody.bound_sources.clone(),
                    consumed: custody.consumed_sources.clone(),
                    completed: custody.completed,
                    tokens: custody.token_count,
                }
            })
            .collect(),
    }
}

fn ready(completion: &super::super::ReadyCompletion) -> DemandState {
    DemandState::Output(super::super::WorthQueryOutputProgress::new(
        WorthQueryOutputCheckpoint::Ready(completion.clone()),
    ))
}

fn fixture(
    seed: u64,
    completion: &super::super::ReadyCompletion,
    joined: bool,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let upstream = key("upstream", seed + 1, 1);
    let dependent = key("dependent", seed + 1, 2);
    let superseded_chain = seed & 1 != 0;
    let mut row = record(
        occurrence(),
        ready(completion),
        usize::from(joined || superseded_chain),
    );
    row.prerequisites = vec![Arc::new(upstream.clone())];
    let wake = Arc::clone(&row.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.required_reserved_bytes =
            row.prerequisites.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
        state.records.insert(dependent.clone(), row);
        let mut upstream_row = record(occurrence(), DemandState::Admitted, 0);
        upstream_row.framework_required_count = 1;
        if superseded_chain {
            upstream_row.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::Superseded,
                "",
            ));
        }
        state.records.insert(upstream.clone(), upstream_row);
        let member = state.prepare_required_member(&upstream).unwrap();
        state.install_required_member(member);
        if joined || superseded_chain {
            let member = state.prepare_required_member(&dependent).unwrap();
            state.install_required_member(member);
        }
        if superseded_chain {
            let newest = key("upstream", seed + 2, 1);
            state
                .records
                .insert(newest.clone(), record(occurrence(), ready(completion), 1));
            let member = state.prepare_required_member(&newest).unwrap();
            state.install_required_member(member);
        }
        if !joined {
            state.required_budget_bytes = 0;
        }
    }
    let interest = interest(&registry, dependent, wake);
    (registry, interest)
}

#[test]
fn selected_mutation_entries_refuse_every_budget_and_retry_by_value() {
    let DemandState::Output(output) = closed_retirement::ready() else {
        panic!("Ready fixture")
    };
    let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else {
        panic!("Ready fixture")
    };
    for seed in 0..16 {
        for entry in 0..3 {
            let (completed, interest) = fixture(seed, &completion, entry == 0);
            let mut measured = admission(1_000_000);
            run(entry, &completed, &interest, &mut measured).unwrap();
            let expected = value(&completed.state.lock().unwrap());
            assert!(measured.charged_work() > 0);
            for k in 0..measured.charged_work() {
                let (retry, interest) = fixture(seed, &completion, entry == 0);
                let before = value(&retry.state.lock().unwrap());
                let denial = run(entry, &retry, &interest, &mut admission(k)).unwrap_err();
                assert_eq!(denial, WorthQueryOutputDemandDenialKind::WorkBudgetExceeded);
                if entry != 1 {
                    assert_eq!(value(&retry.state.lock().unwrap()), before);
                }
                run(entry, &retry, &interest, &mut admission(1_000_000)).unwrap();
                assert_eq!(
                    value(&retry.state.lock().unwrap()),
                    expected,
                    "entry {entry}, seed {seed}, k {k}"
                );
            }
        }
    }
}

fn run(
    entry: u8,
    registry: &WorthQueryOutputDemandRegistry,
    interest: &WorthQueryOutputDemandInterest,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenialKind> {
    match entry {
        0 => registry
            .release_joined_ready(interest, admission)
            .map(|released| assert!(released))
            .map_err(|stop| {
                assert!(matches!(
                    stop,
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted { .. }
                ));
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            }),
        1 => registry
            .reclaim_cached_rows(1, admission)
            .map_err(|denial| denial.kind()),
        2 => registry
            .reclaim_unheld_observation(admission)
            .map(|released| assert!(released))
            .map_err(|denial| denial.kind()),
        _ => panic!("unknown entry"),
    }
}

#[test]
fn supersession_prices_all_release_walks_before_stopping_any_row() {
    for seed in 0..16 {
        let successor = key("dependent", seed * 3 + 3, 2);
        let DemandState::Output(output) = closed_retirement::ready() else {
            panic!("Ready fixture")
        };
        let Some(WorthQueryOutputCheckpoint::Ready(completion)) = output.checkpoint else {
            panic!("Ready fixture")
        };
        let (completed, interest) = two_predecessors(seed, &completion);
        let mut measured = admission(1_000_000);
        supersede_predecessors(
            &mut completed.state.lock().unwrap(),
            &successor,
            &mut measured,
        )
        .unwrap();
        let expected = value(&completed.state.lock().unwrap());
        for k in 0..measured.charged_work() {
            let (retry, _interest) = two_predecessors(seed, &completion);
            let before = value(&retry.state.lock().unwrap());
            let denial = supersede_predecessors(
                &mut retry.state.lock().unwrap(),
                &successor,
                &mut admission(k),
            )
            .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            );
            assert_eq!(value(&retry.state.lock().unwrap()), before);
            supersede_predecessors(
                &mut retry.state.lock().unwrap(),
                &successor,
                &mut admission(1_000_000),
            )
            .unwrap();
            assert_eq!(value(&retry.state.lock().unwrap()), expected);
        }
        drop(interest);
    }
}

fn two_predecessors(
    seed: u64,
    completion: &super::super::ReadyCompletion,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
) {
    let (registry, interest) = fixture(seed * 3, completion, true);
    let mut state = registry.state.lock().unwrap();
    let upstream = key("upstream", seed * 3 + 1, 1);
    let second = key("dependent", seed * 3 + 2, 2);
    let mut row = record(occurrence(), ready(completion), 1);
    row.prerequisites = vec![Arc::new(upstream.clone())];
    state.required_reserved_bytes +=
        row.prerequisites.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
    state
        .records
        .get_mut(&upstream)
        .unwrap()
        .framework_required_count += 1;
    state.records.insert(second.clone(), row);
    let member = state.prepare_required_member(&second).unwrap();
    state.install_required_member(member);
    assert_eq!(
        super::super::refreshed_rejoin::infallible(
            super::super::refreshed_rejoin::occurrence_rows(
                &state.records,
                &second,
                &mut super::OracleWalk::after_effect()
            )
        )
        .count(),
        2
    );
    drop(state);
    (registry, interest)
}

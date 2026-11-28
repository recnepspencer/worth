//! Courts covering installed serial World policy ceilings (including
//! single-metered occupied-key probes), watch refresh and delivery handoff under cancellation.
use std::num::NonZeroUsize;

use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource,
    },
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
        WorthQueryManagedComputationResourceDenial,
    },
    application_entry::{
        WorthQueryApplicationRequestMutationDenial, WorthQueryApplicationRequestQueryDenial,
    },
    application_installation::ApplicationBuildIdentity,
    WorthQueryApplicationHome, WorthQueryHomeConfig,
    WorthQuerySessionRunDenial, WorthQueryTypedManagedIndexRefreshDenial,
};

use super::actions::{
    ProbeOccupiedTaskState, SequencedAutoKey, SetTaskState,
};
use super::program::{owner, seed, DaymarkHome, DaymarkRuntime};
use super::queries::{FindTasksByTitle, ReadyTasks};

pub(super) fn open_memory_home_with_policy(
    clock: &WorthQueryAdmissionClock,
    policy: Option<ExecutionRequestPolicy>,
) -> DaymarkHome {
    let build = ApplicationBuildIdentity::parse("5a5a5a5a-5a5a-5a5a-5a5a-5a5a5a5a5a5a").unwrap();
    let mut config = WorthQueryHomeConfig::new(build, (DaymarkRuntime::new(),))
        .with_clock(clock.clone())
        .with_initial_state(|_, graph, installed| seed(graph, installed));
    if let Some(policy) = policy {
        config = config.with_execution_policy(policy);
    }
    WorthQueryApplicationHome::open_memory(config).expect("open_memory succeeds")
}

#[test]
fn installed_serial_ceiling_and_single_metered_probe_budget() {
    let clock = WorthQueryAdmissionClock::system();
    let installed_ceiling = 2_000_000_u64;
    let installed_policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, None, Some(installed_ceiling)),
    );
    let home = open_memory_home_with_policy(&clock, Some(installed_policy));
    let me = owner(&home, &clock);

    // 1. Default session inherits the installed serial ceiling and succeeds.
    let default_session = home.session(&me);
    let rows = default_session
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect("default session stays within installed serial World ceiling");
    assert_eq!(rows.len(), 2);

    // 2. Requesting a budget exceeding the installed serial World ceiling MUST
    //    fail closed with WorkLimit on both query and run!
    let wider = default_session.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(installed_ceiling + 1),
    ));
    let query_err = wider
        .query(ReadyTasks {
            project: 1,
            priority: None,
            due_before: None,
        })
        .expect_err("session exceeding installed serial ceiling must refuse query");
    assert!(
        matches!(
            query_err,
            WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkLimit
                )
            )
        ),
        "expected WorkLimit refusal on wider serial policy, got {query_err:?}"
    );
    let run_err = wider
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .expect_err("session exceeding installed serial ceiling must refuse run");
    assert!(
        matches!(
            run_err,
            WorthQuerySessionRunDenial::Request(
                WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::WorkLimit
                    )
                )
            )
        ),
        "expected WorkLimit refusal on wider serial run, got {run_err:?}"
    );

    // 3. Single metered request across seed read + occupied-key probes:
    //    Occupy 48 consecutive keys so `run` must charge seed read + 48 probes
    //    cumulatively against ONE request budget.
    let funded = home.session(&me);
    for slot in 1..=48_u64 {
        let _ = funded
            .run_with_key(
                ProbeOccupiedTaskState {
                    task: 1,
                    state: format!("SLOT-{slot}"),
                },
                &SequencedAutoKey(slot),
            )
            .expect("occupying probe slot succeeds");
    }

    let mut found = None;
    let mut candidate = 1_000_u64;
    let mut explicit_key = 10_000_u64;
    while candidate <= installed_ceiling {
        explicit_key += 1;
        let tight = funded.clone().with_budget(ExecutionBudget::new(
            NonZeroUsize::MIN,
            None,
            Some(candidate),
        ));
        if tight
            .run_with_key(
                ProbeOccupiedTaskState {
                    task: 1,
                    state: "SINGLE-COMMIT".to_owned(),
                },
                &SequencedAutoKey(explicit_key),
            )
            .is_ok()
        {
            found = Some(candidate);
            break;
        }
        candidate = candidate.saturating_mul(2);
    }
    let single_commit_budget = found
        .expect("found a finite budget where a single direct commit succeeds")
        .saturating_add(found.unwrap() / 2)
        .min(installed_ceiling);
    // Confirm a single direct commit succeeds at `single_commit_budget`.
    explicit_key += 1;
    let tight_direct = funded.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(single_commit_budget),
    ));
    let _ = tight_direct
        .run_with_key(
            ProbeOccupiedTaskState {
                task: 1,
                state: "SINGLE-COMMIT-OK".to_owned(),
            },
            &SequencedAutoKey(explicit_key),
        )
        .expect("single direct commit fits within calibrated single_commit_budget");

    let tight_auto = funded.clone().with_budget(ExecutionBudget::new(
        NonZeroUsize::MIN,
        None,
        Some(single_commit_budget),
    ));
    let cumulative_err = tight_auto
        .run(ProbeOccupiedTaskState {
            task: 1,
            state: "CUMULATIVE-OVERFLOW".to_owned(),
        })
        .expect_err("cumulative work across 48 occupied probes must exhaust single-commit budget");
    let rendered = format!("{cumulative_err:?}");
    assert!(
        rendered.contains("WorkExhausted"),
        "expected cumulative WorkExhausted refusal, got {rendered}"
    );
}

#[test]
fn watch_refresh_preserves_delivery_and_handles_cancellation() {
    let clock = WorthQueryAdmissionClock::system();
    let home = open_memory_home_with_policy(&clock, None);
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let _idx = session
        .ensure_index_ready(FindTasksByTitle {
            project: 1,
            prefix: String::new(),
        })
        .unwrap();

    let mut watch = session
        .watch(FindTasksByTitle {
            project: 1,
            prefix: "Draft".to_owned(),
        })
        .expect("watch opens");
    assert_eq!(watch.current().unwrap().len(), 2);

    let _ = session
        .run(SetTaskState {
            task: 1,
            state: "REVIEW".to_owned(),
        })
        .unwrap();

    // Pre-cancelled refresh refuses before acknowledging the notification and
    // clears stale rows; subsequent healthy refresh delivers via `finish_next`.
    let cancel = WorthQueryCancellationSource::new();
    cancel.cancel();
    let cancelled_session = session.clone().with_cancellation(cancel.token());
    let cancel_err = watch
        .refresh(&cancelled_session)
        .expect_err("cancelled session must refuse watch refresh");
    assert!(
        matches!(
            cancel_err,
            WorthQueryTypedManagedIndexRefreshDenial::Query(
                WorthQueryApplicationRequestQueryDenial::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(
                        WorthQueryManagedComputationInterruption::Cancelled
                    )
                )
            )
        ),
        "expected Cancelled refresh denial, got {cancel_err:?}"
    );
    assert!(!watch.is_current());
    assert!(watch.current().is_none());

    let delivered = watch
        .refresh(&session)
        .expect("healthy refresh after pre-entry cancellation succeeds")
        .expect("unconsumed notification delivers updated rows");
    assert_eq!(delivered.len(), 2);
    assert_eq!(delivered[0].state, "REVIEW");
    assert!(watch.is_current());
    watch.close().unwrap();
}

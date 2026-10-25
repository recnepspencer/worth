//! Resolution and continuation evidence for a transition interrupted in Store.

use worth_execution::ExecutionRequest;
use worth_relational::facade::branch::RelationalBranchTransitionSequence;
use worth_relational::facade::durability::{
    RelationalBranchTransitionArtifact, RelationalDurabilityUnresolvedSubject,
};
use worth_relational::facade::history::BranchId;
use worth_store_relational::{
    RelationalStoreHome, RelationalStoreTransitionFate, RelationalStoreTransitionJournalRefusal,
};

use super::branch_lifecycle::{self, LifecycleStop};
use super::crash_oracle::InFlight;
use super::media_case::Ended;
use super::workload::{self, Step};

pub(crate) fn outcome(
    home: &RelationalStoreHome,
    before: &[Vec<u8>],
    result: Result<(), LifecycleStop>,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> (Ended, Vec<Vec<u8>>, InFlight) {
    let now = branch_lifecycle::append_live(home.runtime(), before);
    match result {
        Ok(()) => {
            assert_eq!(
                now.len(),
                before.len() + 1,
                "{case}: acknowledged transition not live"
            );
            (Ended::Acknowledged, now, InFlight::Nothing)
        }
        Err(LifecycleStop::Denied(denial)) => {
            assert!(denial.publication_refused(), "{case}: {denial:?}");
            assert_eq!(now, before, "{case}: refused transition was live");
            (Ended::Refused, before.to_vec(), InFlight::Refused)
        }
        Err(LifecycleStop::Unresolved(fate)) => {
            assert_eq!(now, before, "{case}: unresolved transition was live");
            let RelationalDurabilityUnresolvedSubject::BranchTransition {
                sequence, digest, ..
            } = fate.subject()
            else {
                panic!("expected transition fate")
            };
            let resolution = home.resolve_branch_transition(sequence, digest);
            assert!(matches!(resolution, Ok(RelationalStoreTransitionFate::Unresolved) | Err(RelationalStoreTransitionJournalRefusal::Unavailable(_))), "{case}: unresolved transition resolved in its original incarnation: {resolution:?}");
            let later = workload::commit(
                home.runtime(),
                execution,
                &BranchId("main".into()),
                super::batches::create(2_000_000),
            )
            .unwrap_err();
            assert_eq!(later.durability_unresolved_fate(), Some(&fate), "{case}");
            let later = Step::Fork {
                target: BranchId("later".into()),
            }
            .lifecycle(home.runtime(), execution);
            assert!(
                matches!(later, Err(LifecycleStop::Unresolved(ref found)) if found == &fate),
                "{case}: later transition did not retain fate"
            );
            (
                Ended::Unresolved { recovered: false },
                before.to_vec(),
                InFlight::Unresolved(fate),
            )
        }
        Err(LifecycleStop::RequestStopped(reason)) => {
            panic!("{case}: unexpected request stop: {reason:?}")
        }
    }
}

pub(crate) fn assert_recovered(
    home: &RelationalStoreHome,
    acknowledged: &[Vec<u8>],
    flight: &InFlight,
    case: &str,
) -> bool {
    let recovered = branch_lifecycle::artifacts_of_home(&home);
    assert!(
        recovered.starts_with(acknowledged),
        "{case}: lost acknowledged transitions"
    );
    let extra = recovered.len() - acknowledged.len();
    assert!(extra <= 1, "{case}: overtook an in-flight transition");
    let next = RelationalBranchTransitionSequence(acknowledged.len() as u64 + 1);
    let digest = recovered.get(acknowledged.len()).map(|bytes| {
        RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
            .unwrap()
            .payload_digest()
    });
    let probe = match flight {
        InFlight::Unresolved(fate) => {
            let RelationalDurabilityUnresolvedSubject::BranchTransition {
                sequence,
                digest: expected,
                ..
            } = fate.subject()
            else {
                panic!("expected transition fate")
            };
            assert_eq!(sequence, next, "{case}");
            if let Some(digest) = digest {
                assert_eq!(digest, expected, "{case}: recovered a different transition");
            }
            Some(expected)
        }
        _ => digest.or_else(|| {
            acknowledged.last().map(|bytes| {
                RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
                    .unwrap()
                    .payload_digest()
            })
        }),
    };
    if let Some(digest) = probe {
        let resolution = home.resolve_branch_transition(next, digest).unwrap();
        let expected = if extra == 1 {
            RelationalStoreTransitionFate::Committed
        } else {
            RelationalStoreTransitionFate::NotCommitted {
                head: acknowledged.len() as u64,
            }
        };
        assert_eq!(resolution, expected, "{case}: resolve transition");
    }
    match flight {
        InFlight::Nothing | InFlight::Refused | InFlight::NotDurable => {
            assert_eq!(extra, 0, "{case}")
        }
        InFlight::Durable => assert_eq!(extra, 1, "{case}"),
        InFlight::Unresolved(_) => {}
    }
    extra == 1
}

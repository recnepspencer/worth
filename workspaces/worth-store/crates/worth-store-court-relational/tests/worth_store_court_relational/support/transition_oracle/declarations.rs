//! Resolution and continuation evidence for a transition interrupted in Store.

pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_relational::facade::branch::RelationalBranchTransitionSequence;
pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use {
    worth_relational::facade::durability::RelationalBranchTransitionArtifact,
    worth_relational::facade::durability::RelationalDurabilityUnresolvedSubject,
};
pub(crate) use {
    worth_store_relational::RelationalStoreHome,
    worth_store_relational::RelationalStoreTransitionFate,
    worth_store_relational::RelationalStoreTransitionJournalRefusal,
};

pub(crate) use crate::crash_oracle::InFlight;
pub(crate) use crate::media_case::Ended;
pub(crate) use {crate::branch_lifecycle, crate::branch_lifecycle::LifecycleStop};
pub(crate) use {crate::workload, crate::workload::Step};

pub(crate) fn outcome(
    home: &RelationalStoreHome,
    before: &[Vec<u8>],
    result: Result<(), LifecycleStop>,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> (Ended, Vec<Vec<u8>>, InFlight) {
    let now = crate::branch_lifecycle::append_live(home.runtime(), before);
    match result {
        Ok(()) => {
            assert_eq!(
                now.len(),
                before.len() + 1,
                "{case}: acknowledged transition not live"
            );
            (
                crate::media_case::Ended::Acknowledged,
                now,
                crate::crash_oracle::InFlight::Nothing,
            )
        }
        Err(crate::branch_lifecycle::LifecycleStop::Denied(denial)) => {
            assert!(denial.publication_refused(), "{case}: {denial:?}");
            assert_eq!(now, before, "{case}: refused transition was live");
            (
                crate::media_case::Ended::Refused,
                before.to_vec(),
                crate::crash_oracle::InFlight::Refused,
            )
        }
        Err(crate::branch_lifecycle::LifecycleStop::Unresolved(fate)) => {
            assert_eq!(now, before, "{case}: unresolved transition was live");
            let worth_relational::facade::durability::RelationalDurabilityUnresolvedSubject::BranchTransition {
                sequence, digest, ..
            } = fate.subject()
            else {
                panic!("expected transition fate")
            };
            let resolution = home.resolve_branch_transition(sequence, digest);
            assert!(matches!(resolution, Ok(worth_store_relational::RelationalStoreTransitionFate::Unresolved) | Err(worth_store_relational::RelationalStoreTransitionJournalRefusal::Unavailable(_))), "{case}: unresolved transition resolved in its original incarnation: {resolution:?}");
            let later = crate::workload::commit(
                home.runtime(),
                execution,
                &BranchId("main".into()),
                crate::batches::create(2_000_000),
            )
            .unwrap_err();
            assert_eq!(later.durability_unresolved_fate(), Some(&fate), "{case}");
            let later = crate::workload::Step::Fork {
                target: BranchId("later".into()),
            }
            .lifecycle(home.runtime(), execution);
            assert!(
                matches!(later, Err(crate::branch_lifecycle::LifecycleStop::Unresolved(ref found)) if found == &fate),
                "{case}: later transition did not retain fate"
            );
            (
                crate::media_case::Ended::Unresolved { recovered: false },
                before.to_vec(),
                crate::crash_oracle::InFlight::Unresolved(fate),
            )
        }
        Err(crate::branch_lifecycle::LifecycleStop::RequestStopped(reason)) => {
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
    let recovered = crate::branch_lifecycle::artifacts_of_home(home);
    assert!(
        recovered.starts_with(acknowledged),
        "{case}: lost acknowledged transitions"
    );
    let extra = recovered.len() - acknowledged.len();
    assert!(extra <= 1, "{case}: overtook an in-flight transition");
    let next = RelationalBranchTransitionSequence(acknowledged.len() as u64 + 1);
    let digest = recovered.get(acknowledged.len()).map(|bytes| {
        worth_relational::facade::durability::RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
            .unwrap()
            .payload_digest()
    });
    let probe = match flight {
        crate::crash_oracle::InFlight::Unresolved(fate) => {
            let worth_relational::facade::durability::RelationalDurabilityUnresolvedSubject::BranchTransition {
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
                worth_relational::facade::durability::RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
                    .unwrap()
                    .payload_digest()
            })
        }),
    };
    if let Some(digest) = probe {
        let resolution = home.resolve_branch_transition(next, digest).unwrap();
        let expected = if extra == 1 {
            worth_store_relational::RelationalStoreTransitionFate::Committed
        } else {
            worth_store_relational::RelationalStoreTransitionFate::NotCommitted {
                head: acknowledged.len() as u64,
            }
        };
        assert_eq!(resolution, expected, "{case}: resolve transition");
    }
    match flight {
        crate::crash_oracle::InFlight::Nothing
        | crate::crash_oracle::InFlight::Refused
        | crate::crash_oracle::InFlight::NotDurable => {
            assert_eq!(extra, 0, "{case}")
        }
        crate::crash_oracle::InFlight::Durable => assert_eq!(extra, 1, "{case}"),
        crate::crash_oracle::InFlight::Unresolved(_) => {}
    }
    extra == 1
}

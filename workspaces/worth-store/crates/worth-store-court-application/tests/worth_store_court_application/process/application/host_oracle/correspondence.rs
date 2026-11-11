use super::*;
use worth_relational::facade::{history::CommitId, identity::VersionId};

#[test]
fn identity_correspondence_keeps_both_directions_and_refusal_is_atomic() {
    use ObservedDurableIdentity::{NativeCommit as C, NativeVersion as V};
    let mut map = DurableIdentityCorrespondence::default();
    map.admit(C(CommitId(1)), C(CommitId(9))).unwrap();
    map.admit(C(CommitId(1)), C(CommitId(9))).unwrap();
    assert_eq!(map.len(), 1);
    assert!(matches!(
        map.admit(C(CommitId(1)), C(CommitId(10))),
        Err(IdentityDivergence::HomeRemapped { .. })
    ));
    assert!(matches!(
        map.admit(C(CommitId(2)), C(CommitId(9))),
        Err(IdentityDivergence::ReferenceCollision { .. })
    ));
    assert!(matches!(
        map.admit(C(CommitId(2)), V(VersionId::new(2))),
        Err(IdentityDivergence::Family { .. })
    ));
    assert_eq!(map.len(), 1, "all refusals preserve both maps");
    map.admit(C(CommitId(2)), C(CommitId(10))).unwrap();
    map.admit(C(CommitId(1)), C(CommitId(9))).unwrap();
    assert_eq!(
        map.len(),
        2,
        "later observations preserve the original correspondence"
    );
}

#[test]
fn record_identity_admission_preserves_partition_correspondence_atomically() {
    use worth_relational::facade::identity::{EntityId, PartitionId};
    use ObservedDurableIdentity::{Entity as E, NativePartition as P};
    let mut map = DurableIdentityCorrespondence::default();
    map.admit(
        E(EntityId::new(PartitionId(1), 0, 0)),
        E(EntityId::new(PartitionId(9), 0, 0)),
    )
    .unwrap();
    assert_eq!(map.len(), 2);
    assert!(matches!(
        map.admit(
            E(EntityId::new(PartitionId(1), 1, 0)),
            E(EntityId::new(PartitionId(10), 0, 0))
        ),
        Err(IdentityDivergence::HomeRemapped { .. })
    ));
    assert_eq!(
        map.len(),
        2,
        "refused record cannot install either identity"
    );
    assert!(matches!(
        map.admit(P(PartitionId(2)), P(PartitionId(9))),
        Err(IdentityDivergence::ReferenceCollision { .. })
    ));
    map.admit(
        E(EntityId::new(PartitionId(1), 1, 0)),
        E(EntityId::new(PartitionId(9), 1, 0)),
    )
    .unwrap();
    assert_eq!(map.len(), 3);
}

#[test]
fn every_seed_generates_the_complete_host_vocabulary_and_explicit_owner_gates() {
    for seed in [0, 1, u64::MAX] {
        let script = HostScript::generate(seed);
        assert_eq!(script, HostScript::generate(seed));
        assert_eq!(script.seed(), seed);
        let steps = script.commands();
        for predicate in [
            |s: &HostCommand| matches!(s, HostCommand::Open),
            |s: &HostCommand| matches!(s, HostCommand::Initialize),
            |s: &HostCommand| matches!(s, HostCommand::Mutate { .. }),
            |s: &HostCommand| matches!(s, HostCommand::ConcurrentSameKey { .. }),
            |s: &HostCommand| matches!(s, HostCommand::LoseResponse { .. }),
            |s: &HostCommand| matches!(s, HostCommand::CreateBranch(_)),
            |s: &HostCommand| {
                matches!(
                    s,
                    HostCommand::MoveProduct {
                        branch: BranchLabel::Left,
                        ..
                    }
                )
            },
            |s: &HostCommand| {
                matches!(
                    s,
                    HostCommand::MoveProduct {
                        branch: BranchLabel::Right,
                        ..
                    }
                )
            },
            |s: &HostCommand| matches!(s, HostCommand::SettlementFailure { .. }),
            |s: &HostCommand| matches!(s, HostCommand::ContinueCustody { .. }),
            |s: &HostCommand| matches!(s, HostCommand::ReleaseCustody),
            |s: &HostCommand| matches!(s, HostCommand::AbandonCaller { .. }),
            |s: &HostCommand| matches!(s, HostCommand::Checkpoint(CheckpointKind::Explicit)),
            |s: &HostCommand| matches!(s, HostCommand::Checkpoint(CheckpointKind::Automatic)),
            |s: &HostCommand| matches!(s, HostCommand::Close),
            |s: &HostCommand| matches!(s, HostCommand::Reopen),
            |s: &HostCommand| matches!(s, HostCommand::ProgramTransition { .. }),
        ] {
            assert!(
                steps.iter().any(predicate),
                "seed {seed} omits a required command family"
            );
        }
        for command in steps {
            assert_eq!(
                command.activation() == OperationActivation::Current,
                command.activation().owner_merge().is_none()
            );
        }
        assert!(steps
            .iter()
            .any(|s| s.activation() == OperationActivation::ReopenCheckpointImage));
        assert!(steps
            .iter()
            .any(|s| s.activation() == OperationActivation::LifecycleProgramTransition));
        assert!(steps
            .iter()
            .any(|s| s.activation() == OperationActivation::LifecycleDurableSibling));
        assert!(steps
            .iter()
            .any(|s| s.activation() == OperationActivation::HomeMaintenanceFunding));
        let mut fresh = std::collections::HashSet::new();
        for command in steps {
            let issued = match command {
                HostCommand::Mutate { key, .. }
                | HostCommand::MoveProduct { key, .. }
                | HostCommand::LoseResponse { key, .. }
                | HostCommand::ConcurrentSameKey { key, .. }
                | HostCommand::SettlementFailure { key, .. }
                | HostCommand::AbandonCaller { key, .. } => Some(*key),
                HostCommand::Retry { .. }
                | HostCommand::Open
                | HostCommand::Initialize
                | HostCommand::CreateBranch(_)
                | HostCommand::ContinueCustody { .. }
                | HostCommand::ReleaseCustody
                | HostCommand::Checkpoint(_)
                | HostCommand::Close
                | HostCommand::Reopen
                | HostCommand::ProgramTransition { .. }
                | HostCommand::ResumeTransition => None,
            };
            if let Some(key) = issued {
                assert!(
                    fresh.insert(key),
                    "seed {seed} aliases distinct business operations"
                );
            }
        }
        assert!(fresh.len() > 1);
        let (key, value) = steps
            .iter()
            .find_map(|command| {
                if let HostCommand::Mutate { key, value, .. } = command {
                    Some((*key, *value))
                } else {
                    None
                }
            })
            .unwrap();
        assert!(steps.iter().any(|s| matches!(s, HostCommand::Retry { key: retry, value: input } if *retry == key && *input == value)));
        assert!(steps.iter().any(|s| matches!(s, HostCommand::Retry { key: retry, value: input } if *retry == key && *input != value)));
    }
}

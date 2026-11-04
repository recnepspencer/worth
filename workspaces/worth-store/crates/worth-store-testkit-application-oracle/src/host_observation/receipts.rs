//! Compare fresh and readmitted original meaning without equating their authority.
use super::{
    comparison::{compare_head, HostDivergence, ProjectionPart},
    projection::{owner, HeadProjection},
    requests::MutationOutcome,
};
use crate::facade::{DurableIdentityCorrespondence, ObservedDurableIdentity as Id};
use worth_foundational::facade::{CanonicalDigestId, WorldOwnerCoordinate};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationMutationOutcome as Outcome, primary_graph::*,
};
use worth_relational::facade::history::RelationalCommitReceipt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MutationFate {
    Committed,
    IntentDrift,
    DomainDenied,
    Cancelled,
    TimedOut,
    CleanupRequired,
    ProductStale,
    ProductUnpublished,
    ProductPreparationPending,
    ProductPreparationRefused,
    NoEffect,
    Stale,
    Denied,
    Aborted,
    Deferred,
    Unresolved,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptProjection {
    pub native: RelationalCommitReceipt,
    pub outcome: Option<u64>,
    pub key: Option<CanonicalDigestId>,
    pub intent: Option<CanonicalDigestId>,
    pub changed: usize,
    pub emitted: usize,
    pub before: HeadProjection,
    pub after: HeadProjection,
    pub attempt_owner: WorldOwnerCoordinate,
    pub attempt: u64,
}
#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutationProjection {
    pub fate: MutationFate,
    pub receipt: Option<ReceiptProjection>,
}
pub fn observe(outcome: &MutationOutcome, original_before: &HeadProjection) -> MutationProjection {
    let (fate, receipt) = match outcome {
        Outcome::Committed { receipt, .. } | Outcome::AlreadyCommitted(receipt) => {
            let publication = receipt.committed_product_publication();
            let after = HeadProjection {
                owner: owner(publication.product_branch().owner_identity()),
                branch: publication.product_branch().name().as_str().into(),
                incarnation: publication.product_incarnation().ordinal(),
                generation: publication.product_generation().get(),
                commit: publication.composite_commit().ordinal(),
            };
            let binding = receipt.idempotency_binding();
            (
                MutationFate::Committed,
                Some(ReceiptProjection {
                    native: receipt.commit_reference().clone(),
                    outcome: receipt.outcome_identity().map(|id| id.get()),
                    key: Some(CanonicalDigestId::new(*binding.key_identity())),
                    intent: Some(CanonicalDigestId::new(*binding.intent_identity())),
                    changed: receipt.changed_record_count(),
                    emitted: receipt.emitted_effect_count(),
                    before: original_before.clone(),
                    after,
                    attempt_owner: owner(publication.publication_attempt().owner_identity()),
                    attempt: publication.publication_attempt().ordinal(),
                }),
            )
        }
        Outcome::RecoveredCommitted(receipt) => return observe_original(receipt),
        Outcome::IdempotencyIntentDrift => (MutationFate::IntentDrift, None),
        Outcome::DomainDenied(()) => (MutationFate::DomainDenied, None),
        Outcome::Cancelled => (MutationFate::Cancelled, None),
        Outcome::DeadlineExceeded => (MutationFate::TimedOut, None),
        Outcome::Commit(uncommitted) => (
            match uncommitted {
                WorthQueryApplicationUncommitted::CleanupRequired(_) => {
                    MutationFate::CleanupRequired
                }
                WorthQueryApplicationUncommitted::ProductStale(_) => MutationFate::ProductStale,
                WorthQueryApplicationUncommitted::ProductUnpublished(_) => {
                    MutationFate::ProductUnpublished
                }
                WorthQueryApplicationUncommitted::ProductPreparationPending(_) => {
                    MutationFate::ProductPreparationPending
                }
                WorthQueryApplicationUncommitted::ProductPreparationRefused(_) => {
                    MutationFate::ProductPreparationRefused
                }
                WorthQueryApplicationUncommitted::NoEffect(_) => MutationFate::NoEffect,
                WorthQueryApplicationUncommitted::Stale(_) => MutationFate::Stale,
                WorthQueryApplicationUncommitted::Cancelled => MutationFate::Cancelled,
                WorthQueryApplicationUncommitted::TimedOut => MutationFate::TimedOut,
                WorthQueryApplicationUncommitted::Denied(_) => MutationFate::Denied,
                WorthQueryApplicationUncommitted::Aborted => MutationFate::Aborted,
                WorthQueryApplicationUncommitted::Deferred(_) => MutationFate::Deferred,
                WorthQueryApplicationUncommitted::Indeterminate(_) => MutationFate::Unresolved,
            },
            None,
        ),
    };
    MutationProjection { fate, receipt }
}
pub fn observe_original(
    receipt: &WorthQueryRecoveredApplicationCommitReceipt,
) -> MutationProjection {
    let (fate, receipt) = (
        MutationFate::Committed,
        Some(ReceiptProjection {
            native: receipt.commit_reference().clone(),
            outcome: Some(receipt.outcome_identity().get()),
            key: Some(CanonicalDigestId::new(*receipt.idempotency_key_identity())),
            intent: Some(recorded_intent_root(receipt.recorded_intent())),
            changed: receipt.changed_record_count(),
            emitted: receipt.emitted_effect_count(),
            before: HeadProjection::observe(receipt.old_product_head()),
            after: HeadProjection::observe(receipt.new_product_head()),
            attempt_owner: receipt.attempt_identity().owner(),
            attempt: receipt.attempt_identity().ordinal(),
        }),
    );
    MutationProjection { fate, receipt }
}
pub fn compare(
    map: &mut DurableIdentityCorrespondence,
    h: &MutationProjection,
    r: &MutationProjection,
) -> Result<(), HostDivergence> {
    let bad = || HostDivergence::Meaning {
        part: ProjectionPart::Receipt,
        index: 0,
    };
    if h.fate != r.fate {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::Outcome,
            index: 0,
        });
    }
    match (&h.receipt, &r.receipt) {
        (None, None) => Ok(()),
        (None, Some(_)) | (Some(_), None) => Err(bad()),
        (Some(h), Some(r)) => {
            if h.changed != r.changed
                || h.emitted != r.emitted
                || h.native.parents.len() != r.native.parents.len()
            {
                return Err(bad());
            }
            map.admit(
                Id::NativeCommit(h.native.commit_id),
                Id::NativeCommit(r.native.commit_id),
            )?;
            map.admit(
                Id::NativeVersion(h.native.version_id),
                Id::NativeVersion(r.native.version_id),
            )?;
            map.admit(
                Id::NativeBranch(h.native.branch_id.clone()),
                Id::NativeBranch(r.native.branch_id.clone()),
            )?;
            for (hp, rp) in h.native.parents.iter().zip(&r.native.parents) {
                map.admit(Id::NativeCommit(*hp), Id::NativeCommit(*rp))?;
            }
            match (h.outcome, r.outcome) {
                (None, None) => (),
                (Some(h), Some(r)) => map.admit(Id::QueryOutcome(h), Id::QueryOutcome(r))?,
                (None, Some(_)) | (Some(_), None) => return Err(bad()),
            }
            if h.key != r.key || h.intent != r.intent {
                return Err(bad());
            }
            compare_head(map, &h.before, &r.before)?;
            compare_head(map, &h.after, &r.after)?;
            map.admit(
                Id::WorldAttempt {
                    owner: h.attempt_owner,
                    ordinal: h.attempt,
                },
                Id::WorldAttempt {
                    owner: r.attempt_owner,
                    ordinal: r.attempt,
                },
            )?;
            Ok(())
        }
    }
}

fn recorded_intent_root(intent: &str) -> CanonicalDigestId {
    let root = intent
        .strip_prefix("v2:")
        .expect("actual admitted intent uses v2")
        .split(':')
        .next()
        .unwrap();
    assert_eq!(root.len(), 64, "owner-admitted intent root");
    let mut bytes = [0; 32];
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&root[2 * i..2 * i + 2], 16).expect("owner-admitted canonical root");
    }
    CanonicalDigestId::new(bytes)
}

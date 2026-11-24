//! Adversarial reopen verdict against a MemoryOnly reference host.
//!
//! Decides strictly from durable reopened state whether an interrupted
//! in-flight operation landed before its durable effect (`PreOperation`),
//! left recovered unpublished custody (`UnpublishedCustody`), or committed
//! durably (`CommittedInFlight`), then validates full owner-state parity and
//! identity non-reissue under a single run-wide `DurableIdentityCorrespondence`.
use super::{
    comparison::{self, HostDivergence, HostOracleComparison, ProjectionPart},
    custody::{CustodyAction, CustodyCause},
    projection::{HeadProjection, OccurrenceProvenance, StateProjection},
    receipts::ReceiptProjection,
};
use crate::facade::ObservedDurableIdentity as Id;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::history::RelationalCommitReceipt;

/// Exact durable decision of one in-flight operation observed on reopen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveredPrefix {
    /// No durable effect of the in-flight operation survived; state equals `before_ref`.
    PreOperation,
    /// Native effect survived without World product movement; custody is recoverable.
    UnpublishedCustody {
        owner: WorldOwnerCoordinate,
        identity: u64,
        attempt_owner: WorldOwnerCoordinate,
        attempt: u64,
        original_native: RelationalCommitReceipt,
    },
    /// The in-flight operation reached durable World publication; state equals `after_ref`.
    CommittedInFlight,
}

/// Inputs for verifying a continued unpublished custody publication.
pub struct ContinuedParityCheck<'a> {
    pub continued: &'a StateProjection,
    pub after_ref: &'a StateProjection,
    pub before_head: &'a HeadProjection,
    pub attempt_owner: WorldOwnerCoordinate,
    pub attempt: u64,
    pub original_native: &'a RelationalCommitReceipt,
    pub receipt: &'a ReceiptProjection,
    pub reference_receipt: &'a ReceiptProjection,
}

#[derive(Debug)]
pub enum AdversarialDivergence {
    IncompleteHistory,
    UnexpectedCustodyCount {
        found: usize,
        generation: u64,
    },
    UnexpectedGeneration {
        found: u64,
        before: u64,
        after: u64,
    },
    UnexpectedCustodyPosture(String),
    StateParity(HostDivergence),
    ContinuedParity(String),
}

impl From<HostDivergence> for AdversarialDivergence {
    fn from(value: HostDivergence) -> Self {
        Self::StateParity(value)
    }
}

/// Classify a fresh-process reopen after an in-flight interruption and assert
/// full parity with the MemoryOnly reference under `correspondence`.
pub fn classify_and_verify_reopen(
    correspondence: &mut HostOracleComparison,
    reopened: &StateProjection,
    before_ref: &StateProjection,
    after_ref: &StateProjection,
) -> Result<RecoveredPrefix, AdversarialDivergence> {
    if !reopened.history_complete || reopened.history_crossed_retired {
        return Err(AdversarialDivergence::IncompleteHistory);
    }
    // Seed MemoryOnly's predecessor occurrence chain so bounded cache retirement
    // in `after_ref` never hides bootstrap ancestry during `CommittedInFlight`.
    correspondence.observe_reference(before_ref)?;
    let generation = reopened.head.generation;
    let before_gen = before_ref.head.generation;
    let after_gen = after_ref.head.generation;

    if reopened.custody.is_empty() {
        if generation == before_gen {
            comparison::compare(correspondence, reopened, before_ref)?;
            return Ok(RecoveredPrefix::PreOperation);
        }
        if generation == after_gen {
            comparison::compare(correspondence, reopened, after_ref)?;
            return Ok(RecoveredPrefix::CommittedInFlight);
        }
        return Err(AdversarialDivergence::UnexpectedGeneration {
            found: generation,
            before: before_gen,
            after: after_gen,
        });
    }

    verify_unpublished_custody_state(correspondence, reopened, before_ref, after_ref)
}

fn verify_unpublished_custody_state(
    correspondence: &mut HostOracleComparison,
    reopened: &StateProjection,
    before_ref: &StateProjection,
    after_ref: &StateProjection,
) -> Result<RecoveredPrefix, AdversarialDivergence> {
    let generation = reopened.head.generation;
    if reopened.custody.len() != 1 {
        return Err(AdversarialDivergence::UnexpectedCustodyCount {
            found: reopened.custody.len(),
            generation,
        });
    }
    if generation != before_ref.head.generation {
        return Err(AdversarialDivergence::UnexpectedGeneration {
            found: generation,
            before: before_ref.head.generation,
            after: after_ref.head.generation,
        });
    }

    let row = &reopened.custody[0];
    verify_custody_posture(row, &reopened.head)?;
    let orig = row.original_effect.as_ref().ok_or_else(|| {
        AdversarialDivergence::UnexpectedCustodyPosture(
            "recovered unpublished custody lost its original Native effect".into(),
        )
    })?;

    // World product head and occurrence ancestry stayed at `before_ref`.
    comparison::compare_head(
        correspondence.identities_mut(),
        &reopened.head,
        &before_ref.head,
    )?;
    let expected_before = correspondence.reference_occurrences(&before_ref.head)?;
    super::occurrence::compare(
        correspondence.identities_mut(),
        &reopened.occurrences,
        &expected_before,
    )?;

    // Native live branch already committed the in-flight effect (`after_ref`).
    admit_native_graph_parity(correspondence, reopened, after_ref)?;
    super::idempotency::compare(correspondence.identities_mut(), reopened, after_ref)?;

    Ok(RecoveredPrefix::UnpublishedCustody {
        owner: row.owner,
        identity: row.identity,
        attempt_owner: orig.attempt_owner,
        attempt: orig.attempt,
        original_native: orig.native.receipt().clone(),
    })
}

fn verify_custody_posture(
    row: &super::projection::CustodyProjection,
    reopened_head: &HeadProjection,
) -> Result<(), AdversarialDivergence> {
    let bad = AdversarialDivergence::UnexpectedCustodyPosture;
    if row.cause != Some(CustodyCause::RecoveredWithoutProductMovement) {
        return Err(bad(format!(
            "expected RecoveredWithoutProductMovement, got {:?}",
            row.cause
        )));
    }
    if row.settlement != Some(false) || row.effects != Some(1) {
        return Err(bad(format!(
            "expected 1 unsettled owner effect, got effects={:?} settlement={:?}",
            row.effects, row.settlement
        )));
    }
    if row.expected.as_ref() != Some(reopened_head) {
        return Err(bad(format!(
            "custody expected head {:?} != reopened head {:?}",
            row.expected, reopened_head
        )));
    }
    let fresh_actions = [
        CustodyAction::Inspect,
        CustodyAction::ContinueOriginalPublication,
        CustodyAction::ReleaseObligations,
        CustodyAction::CloseOwner,
    ];
    let continuing_actions = [
        CustodyAction::Inspect,
        CustodyAction::ContinueOriginalPublication,
    ];
    if row.actions.as_deref() != Some(&fresh_actions)
        && row.actions.as_deref() != Some(&continuing_actions)
    {
        return Err(bad(format!(
            "unexpected recovered custody actions {:?}",
            row.actions
        )));
    }
    Ok(())
}

fn admit_native_graph_parity(
    correspondence: &mut HostOracleComparison,
    reopened: &StateProjection,
    after_ref: &StateProjection,
) -> Result<(), AdversarialDivergence> {
    let map = correspondence.identities_mut();
    if reopened.entities.len() != after_ref.entities.len()
        || reopened.relations.len() != after_ref.relations.len()
    {
        return Err(HostDivergence::Meaning {
            part: ProjectionPart::GraphCardinality,
            index: 0,
        }
        .into());
    }
    for (index, (h, r)) in reopened.entities.iter().zip(&after_ref.entities).enumerate() {
        if (h.kind_name.as_str(), h.lifecycle, &h.authoritative_aspect_state)
            != (r.kind_name.as_str(), r.lifecycle, &r.authoritative_aspect_state)
        {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::EntityMeaning,
                index,
            }
            .into());
        }
        map.admit(Id::Entity(h.entity_id), Id::Entity(r.entity_id))
            .map_err(HostDivergence::from)?;
        map.admit(
            Id::NativeVersion(h.created_at_version),
            Id::NativeVersion(r.created_at_version),
        )
        .map_err(HostDivergence::from)?;
    }
    for (index, (h, r)) in reopened.relations.iter().zip(&after_ref.relations).enumerate() {
        if (h.kind_name.as_str(), h.lifecycle, &h.authoritative_aspect_state)
            != (r.kind_name.as_str(), r.lifecycle, &r.authoritative_aspect_state)
        {
            return Err(HostDivergence::Meaning {
                part: ProjectionPart::RelationMeaning,
                index,
            }
            .into());
        }
        map.admit(Id::Relation(h.relation_id), Id::Relation(r.relation_id))
            .map_err(HostDivergence::from)?;
    }
    Ok(())
}

/// Verify that continuing recovered unpublished custody produced exact semantic
/// and original-attempt parity with `after_ref` without repeating owner work.
pub fn verify_continued_parity(
    check: &ContinuedParityCheck<'_>,
) -> Result<(), AdversarialDivergence> {
    let err = AdversarialDivergence::ContinuedParity;
    let continued = check.continued;
    let after_ref = check.after_ref;
    if !continued.history_complete || continued.history_crossed_retired {
        return Err(AdversarialDivergence::IncompleteHistory);
    }
    if !continued.custody.is_empty() {
        return Err(err(format!(
            "continued state still retains {} custody rows",
            continued.custody.len()
        )));
    }
    if continued.head.branch != after_ref.head.branch
        || continued.head.generation != check.before_head.generation + 1
        || continued.head.generation != after_ref.head.generation
    {
        return Err(err(format!(
            "continued head {:?} != expected generation {} / ref {:?}",
            continued.head,
            check.before_head.generation + 1,
            after_ref.head
        )));
    }
    if check.receipt.attempt_owner != check.attempt_owner || check.receipt.attempt != check.attempt {
        return Err(err(format!(
            "continued receipt attempt ({:?}, {}) != original custody attempt ({:?}, {})",
            check.receipt.attempt_owner, check.receipt.attempt, check.attempt_owner, check.attempt
        )));
    }
    if check.receipt.before != *check.before_head || check.receipt.after != continued.head {
        return Err(err(format!(
            "continued receipt heads ({:?} -> {:?}) != ({:?} -> {:?})",
            check.receipt.before, check.receipt.after, check.before_head, continued.head
        )));
    }
    if check.receipt.native != *check.original_native {
        return Err(err(format!(
            "continued receipt native {:?} != original custody native {:?}",
            check.receipt.native, check.original_native
        )));
    }
    if (
        check.receipt.changed,
        check.receipt.emitted,
        check.receipt.key,
        &check.receipt.intent,
    ) != (
        check.reference_receipt.changed,
        check.reference_receipt.emitted,
        check.reference_receipt.key,
        &check.reference_receipt.intent,
    ) {
        return Err(err(format!(
            "continued receipt semantic fields {:?} != reference {:?}",
            check.receipt, check.reference_receipt
        )));
    }
    let Some(binding) = continued
        .idempotency
        .iter()
        .find(|row| Some(row.key) == check.receipt.key)
    else {
        return Err(err("continued key missing durable idempotency binding".into()));
    };
    if (check.receipt.outcome, binding.emitted as usize, binding.created)
        != (
            Some(binding.outcome),
            check.receipt.emitted,
            check.original_native.version_id,
        )
    {
        return Err(err(format!(
            "continued idempotency binding {binding:?} != receipt/native"
        )));
    }
    let Some(new_occ) = continued
        .occurrences
        .iter()
        .find(|occ| occ.commit == continued.head.commit)
    else {
        return Err(err(
            "continued head missing from World occurrence ancestry".into(),
        ));
    };
    if new_occ.owner != check.attempt_owner
        || new_occ.parent != Some(check.before_head.commit)
        || new_occ.provenance != OccurrenceProvenance::Publication(check.attempt)
    {
        return Err(err(format!(
            "continued occurrence {:?} does not match original attempt ({:?}, {}) from parent {}",
            new_occ, check.attempt_owner, check.attempt, check.before_head.commit
        )));
    }
    let Some(rel_pub) = new_occ.relational_publication.as_ref() else {
        return Err(err("continued occurrence missing Native publication".into()));
    };
    if (rel_pub.commit_id(), rel_pub.version_id(), rel_pub.authoring_branch())
        != (
            check.original_native.commit_id,
            check.original_native.version_id,
            &check.original_native.branch_id,
        )
    {
        return Err(err(format!(
            "continued occurrence Native publication {:?} != original {:?}",
            rel_pub, check.original_native
        )));
    }
    Ok(())
}

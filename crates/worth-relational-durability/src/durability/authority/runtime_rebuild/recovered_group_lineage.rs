//! Readmission of durable group lineage from canonical commit envelopes.
//!
//! Every recovered frame is decoded strictly, and every group identity it
//! carries is re-minted from its recorded scheme, anchor, continuity rule and
//! sources before any branch state sees it: a frame whose identity bytes, or
//! the parts they were minted from, were altered refuses typed. The lineage
//! subsystem then applies the verified transitions along each branch's fork
//! ancestry, refusing any that its recorded posture contradicts.

use worth_foundational::facade::AggregateIdentity;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_lineage::internal_port::durability::naming::decode_group_lineage_frame;
use worth_relational_lineage::internal_port::lineage::groups::{
    CommittedGroupTransition, GroupStateDenial,
};
use worth_relational_mutation::internal_port::aggregate_identity::AggregateGroupNaming as _;

use contracts::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use contracts::history::data::coordinates::{BranchId, CommitId};

/// One commit's verified group transitions.
pub type RecoveredGroupLineage = (BranchId, CommitId, Vec<CommittedGroupTransition>);

/// Decodes and verifies every envelope's group lineage frame.
///
/// Complexity: O(total frame bytes + groups * sources) hashing.
pub fn recovered_group_lineage<'envelope>(
    envelopes: impl IntoIterator<Item = &'envelope CanonicalCommitEnvelope>,
    failure_class: RecoveryFailureClass,
) -> Result<Vec<RecoveredGroupLineage>, DurabilityError> {
    let mut recovered = Vec::new();
    for envelope in envelopes {
        let frame = envelope.published_lineage().group_lineage();
        if frame.is_empty() {
            continue;
        }
        let commit = envelope.commit.commit_id;
        let refuse = |detail: String| {
            DurabilityError::new(
                failure_class.clone(),
                format!("commit {} group lineage: {detail}", commit.0),
            )
        };
        let transitions =
            decode_group_lineage_frame(frame).map_err(|refusal| refuse(format!("{refusal:?}")))?;
        for committed in &transitions {
            for assignment in &committed.transition.assignments {
                let (scheme, rule) = (committed.scheme.id(), committed.scheme.rule());
                // A source-free group carries its pure `(scheme, anchor, rule)`
                // name; a split or merge successor also hashes its sources.
                let minted = if assignment.sources.is_empty() {
                    AggregateIdentity::from_group_name(scheme, assignment.anchor.clone(), rule)
                } else {
                    AggregateIdentity::from_group_transition(
                        scheme,
                        assignment.anchor.clone(),
                        rule,
                        &assignment.sources,
                    )
                };
                if minted != assignment.identity {
                    return Err(refuse(format!(
                        "group {:?} is not the name its recorded anchor and sources mint",
                        assignment.identity
                    )));
                }
            }
        }
        recovered.push((envelope.branch_context.clone(), commit, transitions));
    }
    Ok(recovered)
}

/// Verifies a checkpoint's envelopes' group lineage and installs it through
/// `install`, which rebuilds every branch's group state along the recorded
/// fork ancestry. A frame, or a branch whose records contradict its group
/// posture, refuses `CorruptCheckpoint`: recovery never installs a branch
/// whose group state is unavailable.
///
/// Complexity: as [`recovered_group_lineage`] plus the rebuild.
pub fn install_recovered_group_lineage<'envelope>(
    envelopes: impl IntoIterator<Item = &'envelope CanonicalCommitEnvelope>,
    install: impl FnOnce(Vec<RecoveredGroupLineage>) -> Result<(), (BranchId, GroupStateDenial)>,
) -> Result<(), DurabilityError> {
    let class = RecoveryFailureClass::CorruptCheckpoint;
    install(recovered_group_lineage(envelopes, class.clone())?).map_err(|(branch, denial)| {
        let detail = format!("branch {} group lineage cannot apply: {denial:?}", branch.0);
        DurabilityError::new(class, detail)
    })
}

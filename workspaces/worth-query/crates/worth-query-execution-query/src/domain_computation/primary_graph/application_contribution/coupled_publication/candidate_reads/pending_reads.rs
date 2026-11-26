//! Pending name reads (#426): a decision of this assembly read the name the
//! envelope gives a record an earlier member created, or a birth an earlier
//! member mints, from the envelope's own naming authority. A later member's
//! append can falsify that answer without touching any fact the decision
//! recorded: a successor round (#119) re-creates the same record identity
//! under another name, or drops the birth. Each append therefore re-verifies
//! every retained read against the envelope's names at the new pin; one it
//! falsifies, or names that can no longer be read, refuse the envelope.
//!
//! A read is retained only while the decision that made it is live. Each
//! decision's reads are staged in its own frame (frames nest with nested
//! preparation) and adopted under the member that appended it: an authored
//! stage, or a chain's slot, whose successor round replaces its reads. A
//! decision that refuses, fails or retries never appends, so its frame is
//! dropped, and an `assemble` retry clears every read. A read made outside
//! any frame is kept unowned and always checked.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Attempt, WorthQueryApplicationAttemptDenialKind as Kind,
};
use std::collections::{BTreeMap, BTreeSet};
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::PendingNameRead;
use worth_relational::facade::runtime::{CustomInvariantNamePosture, CustomInvariantNameSource};

/// The member whose decision made a set of pending reads.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in super::super) enum ReadOwner {
    /// The authored stage applied at this position.
    Stage(usize),
    /// The live round of the chain in this writer slot.
    Part(usize),
}

/// One decision's pending reads.
pub(in super::super) type DecisionReads = BTreeSet<PendingNameRead>;

/// The pending names and births live decisions of this assembly read.
#[derive(Default)]
pub(in super::super) struct PendingReads {
    deciding: Vec<DecisionReads>,
    unowned: DecisionReads,
    owned: BTreeMap<ReadOwner, DecisionReads>,
}

impl PendingReads {
    /// Retains `read` in the deciding frame, or unowned outside any.
    pub(in super::super) fn retain(&mut self, read: PendingNameRead) {
        match self.deciding.last_mut() {
            Some(frame) => frame.insert(read),
            None => self.unowned.insert(read),
        };
    }

    /// Opens the frame of one decision being prepared.
    fn begin_decision(&mut self) {
        self.deciding.push(DecisionReads::new());
    }

    /// Closes the innermost decision's frame and returns its reads.
    fn end_decision(&mut self) -> DecisionReads {
        self.deciding.pop().unwrap_or_default()
    }

    /// Retains an appended decision's reads under `owner`, replacing what
    /// that owner's earlier (superseded) decision read.
    fn adopt(&mut self, owner: ReadOwner, reads: DecisionReads) {
        if reads.is_empty() {
            self.owned.remove(&owner);
        } else {
            self.owned.insert(owner, reads);
        }
    }

    /// Every read a live member's decision rests on.
    fn all(&self) -> impl Iterator<Item = &PendingNameRead> {
        self.unowned.iter().chain(self.owned.values().flatten())
    }
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Runs one decision's preparation in its own read frame; returns its
    /// outcome and the pending reads it made, to adopt once it appends.
    pub(in super::super) fn deciding<T>(&self, decide: impl FnOnce() -> T) -> (T, DecisionReads) {
        self.candidate_reads.borrow_mut().pending.begin_decision();
        let outcome = decide();
        let reads = self.candidate_reads.borrow_mut().pending.end_decision();
        (outcome, reads)
    }

    /// Adopts an appended decision's reads under `owner`, replacing what a
    /// superseded round of the same chain read.
    pub(in super::super) fn adopt_pending_reads(&self, owner: ReadOwner, reads: DecisionReads) {
        self.candidate_reads
            .borrow_mut()
            .pending
            .adopt(owner, reads);
    }

    /// Refuse once an append changes a pending name or birth a decision read.
    ///
    /// Complexity: O(p log r) per append for p retained reads over the
    /// envelope's r named records; the pin's names preview once per pin.
    pub(super) fn refuse_changed_pending_reads(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        let reads = self.candidate_reads.borrow();
        let reads = &reads.pending;
        if reads.all().next().is_none() {
            return Ok(());
        }
        let refuse = |detail: String| {
            CoupledSourceReadStop::capture(Attempt::new(Kind::DecisionDependencyMismatch, detail))
        };
        let pin = candidate.pin_source(execution)?;
        let mut falsified = None;
        let verify = |names: &dyn CustomInvariantNameSource| {
            names.read(CustomInvariantNamePosture::Proposed, &mut |view| {
                falsified = reads.all().find(|read| !read.holds(view)).copied();
            })
        };
        match candidate.with_candidate_names(execution, pin.candidate_identity(), verify)? {
            Some(Ok(())) => match falsified {
                None => Ok(()),
                Some(read) => Err(refuse(format!(
                    "a coupled member changed the pending name {read:?} a candidate decision read"
                ))),
            },
            Some(Err(denial)) => Err(refuse(format!(
                "the pending names a candidate decision read cannot be re-read: {denial:?}"
            ))),
            None => Err(refuse(
                "the pending names a candidate decision read are not the current pin's".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
#[path = "pending_reads_tests.rs"]
mod tests;

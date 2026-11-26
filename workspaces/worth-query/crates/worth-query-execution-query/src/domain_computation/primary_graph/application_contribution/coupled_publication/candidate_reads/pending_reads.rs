//! Pending name reads (#426): a decision of this assembly read the name the
//! envelope gives a record an earlier member created, or a birth an earlier
//! member mints, from the envelope's own naming authority. A later member's
//! append can falsify that answer without touching any fact the decision
//! recorded: a successor round (#119) re-creates the same record identity
//! under another name, or drops the birth. Each append therefore re-verifies
//! every retained read against the envelope's names at the new pin; one it
//! falsifies, or names that can no longer be read, refuse the envelope.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Attempt, WorthQueryApplicationAttemptDenialKind as Kind,
};
use std::collections::BTreeSet;
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::PendingNameRead;
use worth_relational::facade::runtime::{CustomInvariantNamePosture, CustomInvariantNameSource};

/// The pending names and births decisions of this assembly read.
#[derive(Default)]
pub(in super::super) struct PendingReads {
    reads: BTreeSet<PendingNameRead>,
}

impl PendingReads {
    pub(in super::super) fn retain(&mut self, read: PendingNameRead) {
        self.reads.insert(read);
    }
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
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
        let reads = &reads.pending.reads;
        if reads.is_empty() {
            return Ok(());
        }
        let refuse = |detail: String| {
            CoupledSourceReadStop::capture(Attempt::new(Kind::DecisionDependencyMismatch, detail))
        };
        let pin = candidate.pin_source(execution)?;
        let mut falsified = None;
        let verify = |names: &dyn CustomInvariantNameSource| {
            names.read(CustomInvariantNamePosture::Proposed, &mut |view| {
                falsified = reads.iter().find(|read| !read.holds(view)).copied();
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

//! Candidate-basis group reads (#361): a decision of this assembly read a
//! group scheme's state from the committed basis because the unpublished
//! candidate claimed no partition of that scheme. Commit finalization names
//! every claim against the basis groups, so once any member claims the
//! scheme, no group answer read on the candidate is the envelope's answer.
//! Each append is therefore judged against every retained read scheme; a
//! claim, including the reader's own, refuses the envelope with no effect.
//! Member records the read rested on are ordinary decision support: final
//! support compares their liveness between images.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Attempt, WorthQueryApplicationAttemptDenialKind as Kind,
};
use std::collections::BTreeSet;

/// The group schemes decisions of this assembly read on its candidate.
#[derive(Default)]
pub(super) struct GroupReads {
    schemes: BTreeSet<String>,
}

impl GroupReads {
    pub(super) fn retain(&mut self, scheme: &str) {
        if !self.schemes.contains(scheme) {
            self.schemes.insert(scheme.to_owned());
        }
    }
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// Refuse once the candidate claims a scheme a decision read on it.
    pub(super) fn refuse_claimed_group_reads(
        &self,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        let reads = self.group_reads.borrow();
        match reads
            .schemes
            .iter()
            .find(|scheme| candidate.claims_group_scheme(scheme))
        {
            Some(scheme) => Err(CoupledSourceReadStop::capture(Attempt::new(
                Kind::DecisionDependencyMismatch,
                format!("a coupled member claimed group scheme {scheme} a candidate decision read"),
            ))),
            None => Ok(()),
        }
    }
}

//! Actual protected bytes enter the same ordinary recovery admission consumer.
use super::super::{
    RelationalDurableColdSource, RelationalDurableOwnedReadRefusal as Refusal,
    RelationalDurableReadProtection,
};
use super::*;
impl RelationalDurableRecoverySource {
    /// Keep `protection` alive through source admission. Transport/plan collection
    /// heaps retain their ordinary contract; each raw body owns its genuine grant.
    pub fn from_protected_source(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: Option<RelationalDurableCheckpointSource>,
        source: std::sync::Arc<dyn RelationalDurableColdSource>,
        protection: &RelationalDurableReadProtection,
    ) -> Result<Self, Refusal> {
        protection.validate(&source)?;
        let bounds = protection.bounds();
        let covered = checkpoint.as_ref().map_or(0, |cp| cp.covered().0);
        let transitions = checkpoint
            .as_ref()
            .map_or(0, |cp| cp.covered_transitions().0);
        if covered > bounds.canonical.0 || transitions > bounds.transitions.0 {
            return Err(Refusal::Bounds);
        }
        let mut tail = Vec::new();
        let mut branch = Vec::new();
        for position in covered..bounds.canonical.0 {
            let bytes = protection.read_commit_owned_at(
                &source,
                execution,
                PatchStreamPosition(position + 1),
                256 << 20,
            )?;
            tail.push(
                RelationalCanonicalCommitArtifact::from_owned_bytes(bytes)
                    .map_err(Refusal::CanonicalFrame)?,
            );
        }
        for sequence in transitions..bounds.transitions.0 {
            let bytes = protection.read_transition_owned_at(
                &source,
                execution,
                RelationalBranchTransitionSequence(sequence + 1),
                1 << 20,
            )?;
            branch.push(
                RelationalBranchTransitionArtifact::from_owned_bytes(bytes)
                    .map_err(Refusal::TransitionFrame)?,
            );
        }
        Ok(Self::new(checkpoint, tail, branch, source))
    }
}

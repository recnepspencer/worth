//! Closed Home transport is preflighted before descriptor/incarnation publication.
use super::*;
use crate::durability::source_initial_inputs::{
    source_admission, RelationalInitialInputsRefusal as Refusal,
};
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionImmutableBytes, ExecutionRequest};

impl RelationalDurableRecoverySource {
    /// Reattempt against the same immutable bytes/issuer after a denied request.
    /// Only the two concrete transport Vec backings are copied; bytes and
    /// validated original DATA retain their SAME intrinsic owners.
    pub fn copy_home_input_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        if self.readmitted_originals.is_none() {
            return Err(Refusal::Missing);
        }
        self.copy_original_transport_for_request(request)
    }
    /// Copy descriptive transport backing under its invoking request. This
    /// preserves the weaker ordinary source contract and creates no admission.
    pub(crate) fn copy_original_transport_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let canonical_bytes =
            std::alloc::Layout::array::<RelationalCanonicalCommitArtifact>(self.tail.len())
                .map_err(|_| Refusal::Malformed)?
                .size() as u64;
        let transition_bytes =
            std::alloc::Layout::array::<RelationalBranchTransitionArtifact>(self.transitions.len())
                .map_err(|_| Refusal::Malformed)?
                .size() as u64;
        let memory = request
            .reserve_retained_memory(
                canonical_bytes
                    .checked_add(transition_bytes)
                    .ok_or(Refusal::Malformed)?,
            )
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let mut input = Self::new(
            self.checkpoint.clone(),
            self.tail.clone(),
            self.transitions.clone(),
            self.cold_source.clone(),
        );
        input.initial_inputs = self.initial_inputs.clone();
        input.readmitted_originals = self.readmitted_originals.clone();
        input.source_memory = memory;
        Ok(input)
    }
    /// Closed Store source preparation transports bytes from its SAME retained
    /// Home read. This creates no runtime, schema authority or serving permission.
    pub fn from_home_original_bytes(
        checkpoint: Option<RelationalDurableCheckpointSource>,
        initial: Option<ExecutionImmutableBytes>,
        canonical: ExecutionArray<ExecutionImmutableBytes>,
        transitions: ExecutionArray<ExecutionImmutableBytes>,
        bounds: super::super::RelationalDurableReadBounds,
        source: Arc<dyn super::super::RelationalDurableColdSource>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Refusal> {
        let covered = checkpoint.as_ref().map_or(0, |source| source.covered().0);
        let through = checkpoint
            .as_ref()
            .map_or(0, |source| source.covered_transitions().0);
        if checkpoint.is_some() != bounds.selected_checkpoint
            || covered.checked_add(canonical.len() as u64) != Some(bounds.canonical.0)
            || through.checked_add(transitions.len() as u64) != Some(bounds.transitions.0)
        {
            return Err(Refusal::Malformed);
        }
        let memory = request
            .reserve_retained_memory(
                std::alloc::Layout::array::<RelationalCanonicalCommitArtifact>(canonical.len())
                    .map_err(|_| Refusal::Malformed)?
                    .size() as u64
                    + std::alloc::Layout::array::<RelationalBranchTransitionArtifact>(
                        transitions.len(),
                    )
                    .map_err(|_| Refusal::Malformed)?
                    .size() as u64,
            )
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let mut tail = Vec::with_capacity(canonical.len());
        let mut branch = Vec::with_capacity(transitions.len());
        for bytes in canonical.iter() {
            request
                .consult()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            tail.push(
                RelationalCanonicalCommitArtifact::from_owned_bytes(bytes.clone()).map_err(
                    |cause| {
                        Refusal::Source(
                            super::super::RelationalDurableOwnedReadRefusal::CanonicalFrame(cause),
                        )
                    },
                )?,
            );
        }
        for bytes in transitions.iter() {
            request
                .consult()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            branch.push(
                RelationalBranchTransitionArtifact::from_owned_bytes(bytes.clone()).map_err(
                    |cause| {
                        Refusal::Source(
                            super::super::RelationalDurableOwnedReadRefusal::TransitionFrame(cause),
                        )
                    },
                )?,
            );
        }
        let mut input = Self::new(checkpoint, tail, branch, source);
        input.initial_inputs = initial;
        input.source_memory = memory;
        input.preflight_home(request)?;
        if input.readmitted_originals.as_ref().is_some_and(|original| original.current.entries.iter().any(|entry| match entry {
            crate::durability::source_initial_inputs::image::InitialInputEntry::Birth(_) => false,
            crate::durability::source_initial_inputs::image::InitialInputEntry::Installation { follows_transition, .. } => *follows_transition > bounds.transitions,
        })) { return Err(Refusal::Malformed); }
        Ok(input)
    }

    fn preflight_home(&mut self, request: &ExecutionRequest<'_, '_>) -> Result<(), Refusal> {
        let complete = self
            .checkpoint
            .as_ref()
            .map(|source| source.complete_artifact().ok_or(Refusal::Missing))
            .transpose()?;
        let checkpoint = self
            .checkpoint
            .as_ref()
            .map(|source| {
                preflight_covered_versions(
                    source.checkpoint().bytes(),
                    &self.cold_source,
                    request,
                )?;
                let checkpoint =
                    crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
                        request,
                        source.checkpoint().bytes(),
                        Some(self.cold_source.clone()),
                    )
                    .map_err(Refusal::Native)?
                    .checkpoint;
                if crate::durability::access::checkpoint_coverage(&checkpoint) != source.covered()
                    || checkpoint.coverage.up_to_branch_transition != source.covered_transitions()
                {
                    return Err(Refusal::Malformed);
                }
                Ok(checkpoint)
            })
            .transpose()?;
        let initial = self
            .initial_inputs
            .as_ref()
            .ok_or(Refusal::Missing)?
            .clone();
        self.readmitted_originals = Some(Arc::new(source_admission::readmit(
            initial,
            complete,
            checkpoint.as_ref(),
            request,
        )?));
        Ok(())
    }
}

fn preflight_covered_versions(
    bytes: &[u8],
    source: &Arc<dyn super::super::RelationalDurableColdSource>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    use crate::durability::canonical_commit_artifact::borrowed_payload::BorrowedPayload;
    let payload = bytes
        .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
        .ok_or(Refusal::Malformed)?;
    let roster = BorrowedPayload(payload)
        .field("manifest", request)
        .map_err(Refusal::Payload)?
        .field("canonical_members", request)
        .map_err(Refusal::Payload)?;
    let original = source
        .checkpoint_selected_read_protection(request)
        .map_err(Refusal::Source)?;
    let mut stopped = None;
    let mut position = 0u64;
    let result = roster.elements(request, |row| {
        let entry: crate::durability::checkpoint_history::inventory::CanonicalCheckpointMember =
            rmp_serde::from_slice(row.0).map_err(|_| malformed())?;
        position = position.checked_add(1).ok_or_else(malformed)?;
        if entry.position.0 != position {
            return Err(malformed());
        }
        let prefix =
            match source.read_member_prefix_owned(request, original.as_ref(), entry.member, 10) {
                Ok(prefix) => prefix,
                Err(cause) => {
                    stopped = Some(Refusal::Source(cause));
                    return Err(malformed());
                }
            };
        if let Err(cause) =
            crate::durability::canonical_commit_artifact::validate_frame_version(prefix.bytes())
        {
            stopped = Some(Refusal::Source(
                super::super::RelationalDurableOwnedReadRefusal::CanonicalFrame(cause),
            ));
            return Err(malformed());
        }
        Ok(())
    });
    if let Some(cause) = stopped {
        return Err(cause);
    }
    result.map_err(Refusal::Payload)?;
    Ok(())
}

fn malformed(
) -> crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal {
    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal::PayloadDecode {
        detail: "Complete source canonical roster is malformed".into(),
    }
}

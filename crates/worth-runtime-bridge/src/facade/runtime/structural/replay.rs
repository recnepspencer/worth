use super::*;

impl RuntimeBridge {
    /// Replays and verifies a canonical structural remap record.
    pub fn replay_canonical_structural_remap_record(
        &self,
        record: &BridgeCanonicalStructuralRemapRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeStructuralRemapReplaySummary, BridgeReplayError> {
        let record = record.decode()?;
        let reconstructed_contract = self
            .admit_structural_comparison(
                record
                    .contract()
                    .validated_declaration()
                    .declaration()
                    .clone(),
            )
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural remap replay could not reconstruct the admitted structural contract: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        validate_structural_replay_contract(record.contract(), &reconstructed_contract)?;

        let target_fingerprint = record
            .planned_packet_set()
            .target_fingerprint()
            .ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::StructuralReplayBasisTruncated,
                    "Bridge structural remap replay requires a retained target fingerprint basis.",
                )
                .with_context(BridgeErrorContext::default())
            })?;
        let replayed_target = self
            .materialize_structural_fingerprint(
                &reconstructed_contract,
                target_fingerprint.read_packet().clone(),
             execution)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural remap replay could not reconstruct the target structural fingerprint: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        if replayed_target.digest() != target_fingerprint.digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural remap replay reconstructed target fingerprint `{}` but original fingerprint was `{}`.",
                    replayed_target.digest(),
                    target_fingerprint.digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }
        let mut replayed_candidates =
            Vec::with_capacity(record.planned_packet_set().candidates().len());
        for candidate in record.planned_packet_set().candidates() {
            let fingerprint = candidate.fingerprint().ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::StructuralReplayBasisTruncated,
                    format!(
                        "Bridge structural remap replay requires retained fingerprint basis for candidate `{}`.",
                        candidate.candidate_identity().as_str()
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
            let replayed = self
                .materialize_structural_fingerprint(
                    &reconstructed_contract,
                    fingerprint.read_packet().clone(),
                 execution)
                .map_err(|error| {
                    BridgeReplayError::new(
                        BridgeReplayErrorKind::PlanningContractMismatch,
                        format!(
                            "Bridge structural remap replay could not reconstruct candidate fingerprint `{}`: {error}",
                            candidate.candidate_identity().as_str()
                        ),
                    )
                    .with_context(BridgeErrorContext::default())
                })?;
            if replayed.digest() != fingerprint.digest() {
                return Err(BridgeReplayError::new(
                    BridgeReplayErrorKind::DigestMismatch,
                    format!(
                        "Bridge structural remap replay reconstructed candidate fingerprint `{}` but original fingerprint was `{}`.",
                        replayed.digest(),
                        fingerprint.digest()
                    ),
                )
                .with_context(BridgeErrorContext::default()));
            }
            replayed_candidates.push(replayed);
        }
        let planned = self
            .plan_structural_match_packet_set_from_read_packets(
                &reconstructed_contract,
                target_fingerprint.read_packet().clone(),
                replayed_candidates
                    .iter()
                    .map(|fingerprint| fingerprint.read_packet().clone())
                    .collect(),
             execution)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural remap replay could not reconstruct the planned structural packet set: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        if planned.digest() != record.planned_packet_set().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural remap replay reconstructed planned packet set `{}` but original packet set was `{}`.",
                    planned.digest(),
                    record.planned_packet_set().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        let reduced = self
            .reduce_structural_match_set(&planned)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural remap replay could not reduce the planned structural packet set: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        validate_structural_replay_outcome(
            &planned,
            &reduced,
            StructuralComparisonMode::AdvisoryRemap,
        )?;
        if reduced.digest() != record.reduced_match_set().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural remap replay reconstructed reduced match set `{}` but original reduced match set was `{}`.",
                    reduced.digest(),
                    record.reduced_match_set().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        let artifact = self.publish_structural_remap_artifact(&reduced).map_err(|error| {
            BridgeReplayError::new(
                BridgeReplayErrorKind::LoweringContractMismatch,
                format!(
                    "Bridge structural remap replay could not republish the structural remap artifact: {error}"
                ),
            )
            .with_context(BridgeErrorContext::default())
        })?;
        if artifact.digest() != record.artifact().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::LoweringContractMismatch,
                format!(
                    "Bridge structural remap replay reconstructed artifact `{}` but original artifact was `{}`.",
                    artifact.digest(),
                    record.artifact().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        Ok(artifact)
    }

    /// Replays and verifies a canonical structural branch comparison record.
    pub fn replay_canonical_structural_branch_comparison_record(
        &self,
        record: &BridgeCanonicalStructuralBranchComparisonRecord,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeStructuralBranchComparisonReplaySummary, BridgeReplayError> {
        let record = record.decode()?;
        let reconstructed_contract = self
            .admit_structural_comparison(
                record
                    .contract()
                    .validated_declaration()
                    .declaration()
                    .clone(),
            )
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural branch comparison replay could not reconstruct the admitted structural contract: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        validate_structural_replay_contract(record.contract(), &reconstructed_contract)?;

        let left_fingerprint = record
            .planned_packet_set()
            .target_fingerprint()
            .ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::StructuralReplayBasisTruncated,
                    "Bridge structural branch comparison replay requires a retained left-side fingerprint basis.",
                )
                .with_context(BridgeErrorContext::default())
            })?;
        let right_fingerprint = record
            .planned_packet_set()
            .comparison_fingerprint()
            .ok_or_else(|| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::StructuralReplayBasisTruncated,
                    "Bridge structural branch comparison replay requires a retained right-side fingerprint basis.",
                )
                .with_context(BridgeErrorContext::default())
            })?;
        let (left_replayed, right_replayed) = self
            .materialize_structural_branch_fingerprints(
                &reconstructed_contract,
                left_fingerprint.read_packet().clone(),
             execution)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural branch comparison replay could not reconstruct the branch-pair fingerprint basis: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        if left_replayed.digest() != left_fingerprint.digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural branch comparison replay reconstructed left fingerprint `{}` but original fingerprint was `{}`.",
                    left_replayed.digest(),
                    left_fingerprint.digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }
        if right_replayed.digest() != right_fingerprint.digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural branch comparison replay reconstructed right fingerprint `{}` but original fingerprint was `{}`.",
                    right_replayed.digest(),
                    right_fingerprint.digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }
        let planned = self
            .plan_structural_branch_comparison_from_read_packet(
                &reconstructed_contract,
                left_fingerprint.read_packet().clone(),
             execution)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural branch comparison replay could not reconstruct the planned structural packet set: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        if planned.digest() != record.planned_packet_set().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural branch comparison replay reconstructed planned packet set `{}` but original packet set was `{}`.",
                    planned.digest(),
                    record.planned_packet_set().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        let reduced = self
            .reduce_structural_match_set(&planned)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::PlanningContractMismatch,
                    format!(
                        "Bridge structural branch comparison replay could not reduce the planned structural packet set: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        validate_structural_replay_outcome(
            &planned,
            &reduced,
            StructuralComparisonMode::BranchComparison,
        )?;
        if reduced.digest() != record.reduced_match_set().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::DigestMismatch,
                format!(
                    "Bridge structural branch comparison replay reconstructed reduced match set `{}` but original reduced match set was `{}`.",
                    reduced.digest(),
                    record.reduced_match_set().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        let artifact = self
            .publish_branch_comparison_artifact(&reduced)
            .map_err(|error| {
                BridgeReplayError::new(
                    BridgeReplayErrorKind::LoweringContractMismatch,
                    format!(
                        "Bridge structural branch comparison replay could not republish the branch comparison artifact: {error}"
                    ),
                )
                .with_context(BridgeErrorContext::default())
            })?;
        if artifact.digest() != record.artifact().digest() {
            return Err(BridgeReplayError::new(
                BridgeReplayErrorKind::LoweringContractMismatch,
                format!(
                    "Bridge structural branch comparison replay reconstructed artifact `{}` but original artifact was `{}`.",
                    artifact.digest(),
                    record.artifact().digest()
                ),
            )
            .with_context(BridgeErrorContext::default()));
        }

        Ok(artifact)
    }
}

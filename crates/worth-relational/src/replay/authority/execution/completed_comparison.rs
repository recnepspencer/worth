//! Compare observed surfaces against the exact reconstructed replay target.
use super::*;

impl ReplayAuthority<'_> {
    pub(super) fn compare_replay_reconstruction(
        &self,
        request: RelationalReplayRequest,
        admission: ReplayAdmission,
        reconstruction: ReplayReconstruction,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalReplayOutcome, RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let _reconstructed_envelopes = demand_replay_envelopes(
            &reconstruction.runtime.history,
            execution,
            reconstruction
                .runtime
                .history()
                .inspect_commit_ancestry(request.commit_id)
                .commit_ids(),
        )?;
        let compared_surfaces = promised_replay_surfaces(
            snapshot_surface_is_authoritative(
                self.runtime,
                &admission.envelope,
                &admission.commit_closure,
            ),
            &admission.envelope,
            Some(&reconstruction.envelope),
        );
        let mut mismatches = Vec::new();
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let selected_lineage_authority =
            if compared_surfaces.contains(&ReplayObservableSurface::Lineage) {
                match select_replay_lineage_authority(
                    self.runtime,
                    &admission.envelope,
                    request.verification_mode,
                ) {
                    Ok(selected) => Some(selected),
                    Err(()) => {
                        return Ok(self.fail_and_record(
                            request,
                            Some(&admission.envelope),
                            Some(&admission.commit_closure),
                            ReplayFailureClass::AuthoritativeBasisUnavailable,
                            None,
                        ));
                    }
                }
            } else {
                None
            };
        audit_retained_envelope_authority(
            self.runtime,
            &mut mismatches,
            &admission.envelope,
            selected_lineage_authority.as_ref(),
        );
        let validated_envelope = match validated_replay_continuity_envelope(
            self.runtime,
            &admission.envelope,
            &admission.verification_plan,
        ) {
            Ok(validated) => validated,
            Err(mismatch) => {
                return Ok(self.record_continuity_rejection(
                    request,
                    &admission.envelope,
                    &admission.commit_closure,
                    mismatch,
                ))
            }
        };
        let validated_replayed_envelope = match validated_replay_continuity_envelope(
            self.runtime,
            &reconstruction.envelope,
            &admission.verification_plan,
        ) {
            Ok(validated) => validated,
            Err(mismatch) => {
                return Ok(self.record_continuity_rejection(
                    request,
                    &admission.envelope,
                    &admission.commit_closure,
                    mismatch,
                ))
            }
        };
        compare_replay_surfaces(
            execution,
            self.runtime,
            &admission.verification_plan,
            &mut mismatches,
            &compared_surfaces,
            &admission.envelope,
            &reconstruction.envelope,
            &validated_envelope,
            &validated_replayed_envelope,
            &reconstruction.runtime,
            &request,
            selected_lineage_authority.as_ref(),
        )?;
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let outcome = replay_comparison_outcome(
            request,
            &admission,
            &compared_surfaces,
            mismatches,
            selected_lineage_authority.as_ref(),
        );
        Ok(outcome)
    }
}

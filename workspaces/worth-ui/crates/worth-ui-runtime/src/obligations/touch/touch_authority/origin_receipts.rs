use super::UiGraphTouchAuthority;
use crate::declaration::UiDeclarationArtifact;
use crate::graph::UiGraphWorldProfile;
use crate::obligations::touch::{
    inspection_authored_provenance_digests, require_host_observation_alignment,
    require_runtime_diagnostic_alignment, require_service_event_alignment, UiGraphTouchDenial,
    UiGraphTouchOriginClass, UiGraphTouchOriginReceipt, UiGraphTouchOriginWitness,
};
use crate::runtime::{
    WorthUiActiveRuntimeObservation, WorthUiExecutionPlanInspection,
    WorthUiOrdinaryLaneFrameReceipt, WorthUiReplacementCandidate, WorthUiRuntimeDiagnosticReport,
};

impl UiGraphTouchAuthority<'_> {
    pub fn declaration_change_receipt(
        self,
        artifact: &UiDeclarationArtifact,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        if self
            .snapshot
            .lookup()
            .declaration_instances(artifact.identity())
            .value()
            .is_empty()
        {
            return Err(UiGraphTouchDenial::DeclarationChangeOutsideGraphAuthority {
                declaration_identity: artifact.identity().clone(),
            });
        }

        Ok(UiGraphTouchOriginWitness::declaration_instances(
            UiGraphTouchOriginReceipt::declaration_change(artifact),
            artifact.identity().clone(),
        ))
    }

    pub fn query_binding_change_receipt(
        self,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        match self.snapshot.world_profile() {
            UiGraphWorldProfile::SettledQueryBinding {
                view_binding_id,
                binding_reference,
            } => Ok(UiGraphTouchOriginWitness::settled_query_binding(
                UiGraphTouchOriginReceipt::settled_query_binding_change(
                    view_binding_id,
                    binding_reference,
                ),
                view_binding_id.clone(),
                binding_reference.clone(),
            )),
            UiGraphWorldProfile::Authoritative
            | UiGraphWorldProfile::PreviewSessionLabel { .. }
            | UiGraphWorldProfile::PreviewSessionIdentity { .. }
            | UiGraphWorldProfile::BranchSessionLabel { .. }
            | UiGraphWorldProfile::HotReloadCandidate { .. }
            | UiGraphWorldProfile::Diagnostic { .. }
            | UiGraphWorldProfile::HostObservation { .. }
            | UiGraphWorldProfile::TestCertification { .. } => {
                Err(UiGraphTouchDenial::QueryBindingChangeUnavailableInCurrentWorld)
            }
        }
    }

    pub fn host_observation_receipt(
        self,
        observation: WorthUiActiveRuntimeObservation,
        inspection: &WorthUiExecutionPlanInspection,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        require_host_observation_alignment(&observation, inspection)?;
        let digests = inspection_authored_provenance_digests(inspection.provenance().iter());
        if digests.is_empty() {
            return Err(UiGraphTouchDenial::OriginAuthorityUnavailable {
                origin_class: UiGraphTouchOriginClass::HostObservation,
            });
        }

        Ok(UiGraphTouchOriginWitness::authored_provenance_digests(
            UiGraphTouchOriginReceipt::host_observation(
                observation.artifact_digest().rotate_left(7) ^ observation.active_plan_digest(),
            ),
            digests,
        ))
    }

    pub fn service_event_receipt(
        self,
        frame_receipt: &WorthUiOrdinaryLaneFrameReceipt,
        inspection: &WorthUiExecutionPlanInspection,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        require_service_event_alignment(frame_receipt, inspection)?;
        let touch = frame_receipt.touch();
        let digests = inspection_authored_provenance_digests(
            inspection
                .provenance()
                .iter()
                .filter(|row| touch.names_plan_index(row.plan_index())),
        );
        if digests.is_empty() {
            return Err(UiGraphTouchDenial::OriginAuthorityUnavailable {
                origin_class: UiGraphTouchOriginClass::ServiceEvent,
            });
        }

        Ok(UiGraphTouchOriginWitness::authored_provenance_digests(
            UiGraphTouchOriginReceipt::service_event(touch.touch_digest()),
            digests,
        ))
    }

    pub fn intent_submission_receipt(
        self,
        candidate: &WorthUiReplacementCandidate,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        let digests = candidate
            .artifact_bundle()
            .artifact()
            .authored_provenance_digests();
        if digests.is_empty() {
            return Err(UiGraphTouchDenial::OriginAuthorityUnavailable {
                origin_class: UiGraphTouchOriginClass::IntentSubmission,
            });
        }

        Ok(UiGraphTouchOriginWitness::authored_provenance_digests(
            UiGraphTouchOriginReceipt::intent_submission(
                candidate.provenance_handle().raw() ^ candidate.basis().lowering_basis_digest(),
            ),
            digests,
        ))
    }

    pub fn diagnostic_only_report_receipt(
        self,
        report: &WorthUiRuntimeDiagnosticReport,
        inspection: &WorthUiExecutionPlanInspection,
    ) -> Result<UiGraphTouchOriginWitness, UiGraphTouchDenial> {
        require_runtime_diagnostic_alignment(report, inspection)?;
        let digests = inspection_authored_provenance_digests(inspection.provenance().iter());
        if digests.is_empty() {
            return Err(UiGraphTouchDenial::OriginAuthorityUnavailable {
                origin_class: UiGraphTouchOriginClass::DiagnosticOnly,
            });
        }

        Ok(UiGraphTouchOriginWitness::authored_provenance_digests(
            UiGraphTouchOriginReceipt::diagnostic_only(
                report.active_artifact_digest() ^ report.active_plan_digest().rotate_left(11),
            ),
            digests,
        ))
    }
}

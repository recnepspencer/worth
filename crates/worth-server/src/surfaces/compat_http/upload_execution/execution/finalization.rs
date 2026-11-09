use worth_proof::TransitionOutcome;

use crate::{
    WorthServerCompatibilityFacade, WorthServerCompatibilityPreparedRequest,
    WorthServerMultipartUpload, WorthServerOperationFamily, WorthServerOperationInputEnvelope,
    WorthServerOperationRequestFacade, WorthServerQueryHandoffDenial,
};

use super::super::super::{build_upload_certification_bundle, project_upload_envelope};
use super::super::{
    integrity::WorthServerIngressIntegrityDigest,
    lifecycle::{remove_active_session, require_active_session},
    performance::WorthServerIngressPerformanceReceipt,
    response::WorthServerCompatibilityUpload,
    session::{WorthServerBinaryIngressSession, WorthServerVerifiedBinaryIngress},
};
use super::contract::{WorthServerCompatibilityUploadOutcome, WorthServerPreparedMultipartUpload};

impl WorthServerCompatibilityFacade {
    pub fn finalize_upload(
        &self,
        verified: WorthServerVerifiedBinaryIngress,
    ) -> WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload> {
        let (session, integrity_digest) = verified.into_parts();
        if let Err(denial) = require_active_session(self, &session) {
            return TransitionOutcome::Denied(denial);
        }
        let (
            prepared_request,
            operation_name,
            upload,
            mutation_request,
            _binary_operation_admission,
            _tenant_id,
            _workspace_digest,
            _branch_digest,
            _session_digest,
            performance_receipt,
        ) = session.clone().into_parts();
        let operation_request =
            match self.build_query_operation_request(&prepared_request, &operation_name, &upload) {
                Ok(value) => value,
                Err(denial) => return TransitionOutcome::Denied(denial),
            };
        let outcome = self.execute_and_certify_upload(
            &session,
            prepared_request,
            operation_request,
            mutation_request,
            upload,
            integrity_digest,
            performance_receipt,
        );
        if matches!(outcome, TransitionOutcome::Success(_)) {
            remove_active_session(self, &session);
        }
        outcome
    }

    fn build_query_operation_request(
        &self,
        prepared_request: &WorthServerCompatibilityPreparedRequest,
        operation_name: &str,
        upload: &WorthServerMultipartUpload,
    ) -> Result<crate::WorthServerOperationRequest, WorthServerQueryHandoffDenial> {
        match WorthServerOperationRequestFacade::new(self.operation_registry.clone())
            .admit_from_compat_http(
                prepared_request,
                WorthServerOperationFamily::QueryDirectSubmission,
                operation_name,
                Some(WorthServerOperationInputEnvelope::json(
                    "compat-http.query-mutation.v1",
                    upload.manifest().metadata_body(),
                )),
            ) {
            Ok(value) => Ok(value),
            Err(denial) => Err(WorthServerQueryHandoffDenial::new(
                crate::WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid,
                denial.diagnostics_profile(),
                denial.detail(),
            )),
        }
    }

    fn execute_and_certify_upload(
        &self,
        session: &WorthServerBinaryIngressSession,
        prepared_request: WorthServerCompatibilityPreparedRequest,
        operation_request: crate::WorthServerOperationRequest,
        mutation_request: crate::WorthServerCompatibilityMutationRequest,
        upload: WorthServerMultipartUpload,
        integrity_digest: WorthServerIngressIntegrityDigest,
        performance_receipt: WorthServerIngressPerformanceReceipt,
    ) -> WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload> {
        match crate::surfaces::compat_http::mutation_execution::execute_compatibility_mutation_request(
            self,
            prepared_request,
            operation_request,
            mutation_request,
        ) {
        TransitionOutcome::Success(mutation) => {
            let file_envelope = match project_upload_envelope(session, &mutation) {
                Ok(envelope) => envelope,
                Err(cause) => return TransitionOutcome::Denied(WorthServerQueryHandoffDenial::new(
                    crate::WorthServerQueryHandoffDenialCode::EvidenceIdentityDerivationDenied(cause),
                    mutation.envelope().response_envelope().diagnostics_profile(),
                    "upload truth envelope identity was refused",
                )),
            };
            let certification_bundle = build_upload_certification_bundle(
                &self.operator_evidence,
                mutation.envelope().support_posture(),
                &file_envelope,
                mutation.envelope().response_envelope(),
                &performance_receipt,
            );
            TransitionOutcome::Success(WorthServerCompatibilityUpload::new(
                upload,
                integrity_digest,
                performance_receipt,
                mutation,
                file_envelope,
                certification_bundle,
            ))
        },
        TransitionOutcome::Denied(value) => TransitionOutcome::Denied(value),
        TransitionOutcome::Deferred(value) => TransitionOutcome::Deferred(value),
        TransitionOutcome::Stale(value) => TransitionOutcome::Stale(value),
        TransitionOutcome::RebindRequired(value) => TransitionOutcome::RebindRequired(value),
        TransitionOutcome::Failed(value) => TransitionOutcome::Failed(value),
        }
    }

    pub fn execute_upload(
        &self,
        prepared: WorthServerPreparedMultipartUpload,
    ) -> WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload> {
        match self.begin_binary_ingress(prepared) {
            TransitionOutcome::Success(session) => match self.verify_binary_ingress(session) {
                TransitionOutcome::Success(verified) => self.finalize_upload(verified),
                TransitionOutcome::Denied(value) => TransitionOutcome::Denied(value),
                TransitionOutcome::Deferred(value) => TransitionOutcome::Deferred(value),
                TransitionOutcome::Stale(value) => TransitionOutcome::Stale(value),
                TransitionOutcome::RebindRequired(value) => {
                    TransitionOutcome::RebindRequired(value)
                }
                TransitionOutcome::Failed(value) => TransitionOutcome::Failed(value),
            },
            TransitionOutcome::Denied(value) => TransitionOutcome::Denied(value),
            TransitionOutcome::Deferred(value) => TransitionOutcome::Deferred(value),
            TransitionOutcome::Stale(value) => TransitionOutcome::Stale(value),
            TransitionOutcome::RebindRequired(value) => TransitionOutcome::RebindRequired(value),
            TransitionOutcome::Failed(value) => TransitionOutcome::Failed(value),
        }
    }

    pub fn upload(
        &self,
        input: super::contract::WorthServerCompatibilityUploadExecutionInput,
    ) -> WorthServerCompatibilityUploadOutcome<WorthServerCompatibilityUpload> {
        match self.prepare_upload(input) {
            TransitionOutcome::Success(value) => self.execute_upload(value),
            TransitionOutcome::Denied(value) => TransitionOutcome::Denied(value),
            TransitionOutcome::Deferred(value) => TransitionOutcome::Deferred(value),
            TransitionOutcome::Stale(value) => TransitionOutcome::Stale(value),
            TransitionOutcome::RebindRequired(value) => TransitionOutcome::RebindRequired(value),
            TransitionOutcome::Failed(value) => TransitionOutcome::Failed(value),
        }
    }
}

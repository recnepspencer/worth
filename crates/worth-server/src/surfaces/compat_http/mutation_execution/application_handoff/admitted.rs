//! Stable transport meaning after Server operation admission.
use super::super::{WorthServerCompatibilityMutationRequest, WorthServerIdempotencyKey};
use crate::{WorthServerCompatibilityPreparedRequest, WorthServerOperationAdmissionPosture};
use sha2::{Digest, Sha256};

/// Admission-owned input to the installed Query application, never a receipt.
#[derive(Debug)]
pub struct WorthServerAdmittedQueryMutation {
    prepared: WorthServerCompatibilityPreparedRequest,
    admission: WorthServerOperationAdmissionPosture,
    operation: String,
    mutation: WorthServerCompatibilityMutationRequest,
    scoped_key: [u8; 32],
    intent: [u8; 32],
}
impl WorthServerAdmittedQueryMutation {
    pub(in super::super) fn new(
        prepared: WorthServerCompatibilityPreparedRequest,
        admission: WorthServerOperationAdmissionPosture,
        operation: String,
        mutation: WorthServerCompatibilityMutationRequest,
        key: &WorthServerIdempotencyKey,
    ) -> Self {
        let context = prepared.admission().request_context();
        // Branch affinity belongs to Query. A sibling must find the same key,
        // then refuse its different recorded branch, rather than execute anew.
        let scope = serde_json::to_vec(&(
            "worth.server.query-mutation-key.v1",
            context.authenticated_principal().principal_id(),
            context.workspace_target().tenant_id(),
            context.workspace_target().workspace_id(),
            key.value(),
        ))
        .expect("string tuple encoding is infallible");
        let scoped_key = Sha256::digest(scope).into();
        let stable = serde_json::to_vec(&(
            "worth.server.query-mutation-intent.v1",
            &operation,
            &mutation,
        ))
        .expect("validated mutation values encode as JSON");
        let intent = Sha256::digest(stable).into();
        Self {
            prepared,
            admission,
            operation,
            mutation,
            scoped_key,
            intent,
        }
    }
    pub fn prepared_request(&self) -> &WorthServerCompatibilityPreparedRequest {
        &self.prepared
    }
    pub fn admission(&self) -> &WorthServerOperationAdmissionPosture {
        &self.admission
    }
    pub fn operation_name(&self) -> &str {
        &self.operation
    }
    pub fn mutation(&self) -> &WorthServerCompatibilityMutationRequest {
        &self.mutation
    }
    pub fn scoped_key(&self) -> &[u8; 32] {
        &self.scoped_key
    }
    pub fn stable_intent(&self) -> &[u8; 32] {
        &self.intent
    }
}

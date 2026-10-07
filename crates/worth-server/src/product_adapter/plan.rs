use crate::{
    WorthServerOperationAdmissionPosture, WorthServerOperationConcurrencyClass,
    WorthServerOperationPreconditionPosture, WorthServerOperationSupportPosture,
};

use super::WorthServerProductOperationAuthorization;
use super::{WorthServerProductOperationDeclaration, WorthServerProductOperationPayload};

#[derive(Clone, Debug)]
pub struct WorthServerLoweredProductOperationPlan {
    operation_admission: WorthServerOperationAdmissionPosture,
    declaration: WorthServerProductOperationDeclaration,
    payload: WorthServerProductOperationPayload,
    application_authorization: Option<WorthServerProductOperationAuthorization>,
    support_posture: WorthServerOperationSupportPosture,
    precondition_posture: WorthServerOperationPreconditionPosture,
    concurrency_class: WorthServerOperationConcurrencyClass,
    canonical_digest: String,
}

impl WorthServerLoweredProductOperationPlan {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use super::execution_pipeline::read_batch_accounting::string;
        self.operation_admission
            .owned_allocation_capacity_bytes()
            .saturating_add(self.declaration.owned_allocation_capacity_bytes())
            .saturating_add(self.payload.owned_allocation_capacity_bytes())
            .saturating_add(
                self.application_authorization
                    .as_ref()
                    .map_or(0, |authorization| {
                        authorization.owned_allocation_capacity_bytes()
                    }),
            )
            .saturating_add(self.support_posture.owned_allocation_capacity_bytes())
            .saturating_add(self.precondition_posture.owned_allocation_capacity_bytes())
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(
        operation_admission: WorthServerOperationAdmissionPosture,
        declaration: WorthServerProductOperationDeclaration,
        payload: WorthServerProductOperationPayload,
        application_authorization: Option<WorthServerProductOperationAuthorization>,
        support_posture: WorthServerOperationSupportPosture,
        precondition_posture: WorthServerOperationPreconditionPosture,
        concurrency_class: WorthServerOperationConcurrencyClass,
    ) -> Self {
        let authority_metadata_digest = operation_admission.authority_metadata().canonical_digest();
        let canonical_digest = crate::canonical_digest::WorthServerCanonicalDigestBuilder::new(
            "worth-server-lowered-product-operation-plan-v1",
        )
        .field(
            "identity",
            operation_admission
                .operation_request()
                .identity()
                .canonical_digest(),
        )
        .field("metadata", &authority_metadata_digest)
        .field(
            "footprint",
            operation_admission.authority_footprint().canonical_digest(),
        )
        .field("support", support_posture.canonical_digest())
        .field("precondition", precondition_posture.canonical_digest())
        .field("payload", payload.envelope().canonical_digest())
        .field(
            "application_authorization",
            application_authorization
                .as_ref()
                .map(WorthServerProductOperationAuthorization::canonical_digest)
                .unwrap_or("none"),
        )
        .field("concurrency", concurrency_label(&concurrency_class))
        .finish();
        Self {
            operation_admission,
            declaration,
            payload,
            application_authorization,
            support_posture,
            precondition_posture,
            concurrency_class,
            canonical_digest,
        }
    }

    pub fn operation_admission(&self) -> &WorthServerOperationAdmissionPosture {
        &self.operation_admission
    }

    pub fn declaration(&self) -> &WorthServerProductOperationDeclaration {
        &self.declaration
    }

    pub fn payload(&self) -> &WorthServerProductOperationPayload {
        &self.payload
    }

    pub fn application_authorization(&self) -> Option<&WorthServerProductOperationAuthorization> {
        self.application_authorization.as_ref()
    }

    pub fn support_posture(&self) -> &WorthServerOperationSupportPosture {
        &self.support_posture
    }

    pub fn precondition_posture(&self) -> &WorthServerOperationPreconditionPosture {
        &self.precondition_posture
    }

    pub fn concurrency_class(&self) -> WorthServerOperationConcurrencyClass {
        self.concurrency_class.clone()
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

fn concurrency_label(concurrency_class: &WorthServerOperationConcurrencyClass) -> &'static str {
    match concurrency_class {
        WorthServerOperationConcurrencyClass::ConcurrentSharedRead => "shared-read",
        WorthServerOperationConcurrencyClass::SerializeDeterministically => {
            "serialize-deterministically"
        }
    }
}

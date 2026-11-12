use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    declaration_intake::WorthServerDirectDeclarationIntakeFacade,
    product_operation_contract::WorthServerStoredProductOperation, CompatHttpSurfaceRoot,
    WorthServerMiddlewareFacade, WorthServerOperationDenial, WorthServerOperationFamily,
    WorthServerOperationRegistry, WorthServerOperatorEvidenceFacade,
    WorthServerProductAdapterRegistry, WorthServerProductSessionRegistry,
    WorthServerQueryHandoffDenial, WorthServerQueryHandoffDenialCode,
    WorthServerQueryHandoffFacade, WorthServerRequestContextFacade, WorthServerResponseFacade,
    WorthServerSurfaceFamily,
};

use super::upload_execution::WorthServerStoredBinaryIngress;

#[derive(Clone, Debug)]
pub struct WorthServerCompatibilityFacade {
    pub(super) root: CompatHttpSurfaceRoot,
    pub(super) operation_registry: WorthServerOperationRegistry,
    pub(super) product_adapter_registry: WorthServerProductAdapterRegistry,
    pub(super) product_session_registry: WorthServerProductSessionRegistry,
    pub(super) request_contexts: WorthServerRequestContextFacade,
    pub(super) middleware: WorthServerMiddlewareFacade,
    pub(super) declaration_intake: WorthServerDirectDeclarationIntakeFacade,
    pub(super) query_handoff: WorthServerQueryHandoffFacade,
    pub(super) responses: WorthServerResponseFacade,
    pub(super) operator_evidence: WorthServerOperatorEvidenceFacade,
    pub(super) product_operation_retry_store:
        Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
    pub(super) binary_ingress_store: Arc<Mutex<HashMap<String, WorthServerStoredBinaryIngress>>>,
    pub(super) counters: Arc<crate::diagnostics::WorthServerCounters>,
}

pub(crate) struct WorthServerCompatibilityFacadeParts {
    pub(crate) root: CompatHttpSurfaceRoot,
    pub(crate) operation_registry: WorthServerOperationRegistry,
    pub(crate) product_adapter_registry: WorthServerProductAdapterRegistry,
    pub(crate) product_session_registry: WorthServerProductSessionRegistry,
    pub(crate) request_contexts: WorthServerRequestContextFacade,
    pub(crate) middleware: WorthServerMiddlewareFacade,
    pub(crate) declaration_intake: WorthServerDirectDeclarationIntakeFacade,
    pub(crate) query_handoff: WorthServerQueryHandoffFacade,
    pub(crate) responses: WorthServerResponseFacade,
    pub(crate) operator_evidence: WorthServerOperatorEvidenceFacade,
    pub(crate) product_operation_retry_store:
        Arc<Mutex<HashMap<String, WorthServerStoredProductOperation>>>,
    pub(crate) binary_ingress_store: Arc<Mutex<HashMap<String, WorthServerStoredBinaryIngress>>>,
    pub(crate) counters: Arc<crate::diagnostics::WorthServerCounters>,
}

impl WorthServerCompatibilityFacade {
    pub(crate) fn new(parts: WorthServerCompatibilityFacadeParts) -> Self {
        let WorthServerCompatibilityFacadeParts {
            root,
            operation_registry,
            product_adapter_registry,
            product_session_registry,
            request_contexts,
            middleware,
            declaration_intake,
            query_handoff,
            responses,
            operator_evidence,
            product_operation_retry_store,
            binary_ingress_store,
            counters,
        } = parts;
        Self {
            root,
            operation_registry,
            product_adapter_registry,
            product_session_registry,
            request_contexts,
            middleware,
            declaration_intake,
            query_handoff,
            responses,
            operator_evidence,
            product_operation_retry_store,
            binary_ingress_store,
            counters,
        }
    }

    pub fn root(&self) -> &CompatHttpSurfaceRoot {
        &self.root
    }

    pub fn operation_registry(&self) -> &WorthServerOperationRegistry {
        &self.operation_registry
    }

    pub fn product_operations(&self) -> super::WorthServerCompatibilityProductOperationFacade {
        super::WorthServerCompatibilityProductOperationFacade::new(
            self.operation_registry.clone(),
            self.product_adapter_registry.clone(),
            self.query_handoff.config().clone(),
            self.product_session_registry.clone(),
            self.product_operation_retry_store.clone(),
            self.counters.clone(),
        )
    }

    pub fn product_sessions(&self) -> super::WorthServerCompatibilityProductSessionFacade {
        super::WorthServerCompatibilityProductSessionFacade::new(
            self.operation_registry.clone(),
            self.product_adapter_registry.clone(),
            self.product_session_registry.clone(),
        )
    }

    pub(super) fn admit_operation_family_for_query(
        &self,
        diagnostics_profile: worth_foundational::DiagnosticRichnessProfile,
        family: WorthServerOperationFamily,
    ) -> Result<(), WorthServerQueryHandoffDenial> {
        self.operation_registry
            .admit(WorthServerSurfaceFamily::CompatHttp, family)
            .map(|_| ())
            .map_err(|denial| {
                let detail = denial.detail();
                let query_denial = WorthServerQueryHandoffDenial::new(
                    map_operation_denial_to_query_code(&denial),
                    diagnostics_profile,
                    detail,
                );
                match denial {
                    WorthServerOperationDenial::UnknownOperationName { operation_name, .. } => {
                        query_denial.with_facts(
                            crate::WorthServerQueryHandoffDenialFacts::default()
                                .with_rejected_operation_name(operation_name),
                        )
                    }
                    _ => query_denial,
                }
            })
    }
}

pub(crate) fn map_operation_admission_denial(
    denial: crate::WorthServerOperationAdmissionDenial,
) -> WorthServerQueryHandoffDenial {
    let code = match denial.code() {
        crate::WorthServerOperationAdmissionDenialCode::AuthorityDenied => {
            WorthServerQueryHandoffDenialCode::AuthorityDenied
        }
        crate::WorthServerOperationAdmissionDenialCode::AuthorizationDenied => {
            WorthServerQueryHandoffDenialCode::AuthorizationDenied
        }
    };
    WorthServerQueryHandoffDenial::new(code, denial.diagnostics_profile(), denial.detail())
}

fn map_operation_denial_to_query_code(
    denial: &WorthServerOperationDenial,
) -> WorthServerQueryHandoffDenialCode {
    match denial {
        WorthServerOperationDenial::UnregisteredFamily { .. } => {
            WorthServerQueryHandoffDenialCode::OperationFamilyNotRegistered
        }
        WorthServerOperationDenial::DisabledFamily { .. } => {
            WorthServerQueryHandoffDenialCode::OperationFamilyDisabled
        }
        WorthServerOperationDenial::SurfaceFamilyNotExposed { .. } => {
            WorthServerQueryHandoffDenialCode::OperationFamilyNotExposedOnSurface
        }
        WorthServerOperationDenial::UnknownOperationName { .. } => {
            WorthServerQueryHandoffDenialCode::UnknownOperationName
        }
    }
}

use crate::{
    request_context::DiagnosticRichnessProfile, WorthServerSurfaceFamily, WorthServerTransportClass,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationRequestReceipt {
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
    diagnostics_profile: DiagnosticRichnessProfile,
    request_context_digest: String,
    source_contract_digest: Option<String>,
    browser_origin: Option<String>,
    canonical_digest: String,
}

impl WorthServerOperationRequestReceipt {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{
            option_string, string,
        };
        string(&self.request_context_digest)
            .saturating_add(option_string(&self.source_contract_digest))
            .saturating_add(option_string(&self.browser_origin))
            .saturating_add(string(&self.canonical_digest))
    }

    pub(crate) fn new(
        surface_family: WorthServerSurfaceFamily,
        transport_class: WorthServerTransportClass,
        diagnostics_profile: DiagnosticRichnessProfile,
        request_context_digest: String,
        source_contract_digest: Option<String>,
    ) -> Self {
        let mut receipt = Self {
            surface_family,
            transport_class,
            diagnostics_profile,
            request_context_digest,
            source_contract_digest,
            browser_origin: None,
            canonical_digest: String::new(),
        };
        receipt.refresh_canonical_digest();
        receipt
    }

    pub(crate) fn with_browser_origin(mut self, browser_origin: Option<String>) -> Self {
        self.browser_origin = browser_origin;
        self.refresh_canonical_digest();
        self
    }

    pub fn diagnostics_profile(&self) -> DiagnosticRichnessProfile {
        self.diagnostics_profile
    }

    pub fn source_contract_digest(&self) -> Option<&str> {
        self.source_contract_digest.as_deref()
    }

    pub fn browser_origin(&self) -> Option<&str> {
        self.browser_origin.as_deref()
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }

    fn refresh_canonical_digest(&mut self) {
        self.canonical_digest = format!(
            "worth-server-operation-request-receipt-v2|surface={:?}|transport={:?}|diagnostics={:?}|request_context={}|source_contract={}|browser_origin={}",
            self.surface_family,
            self.transport_class,
            self.diagnostics_profile,
            self.request_context_digest,
            self.source_contract_digest.as_deref().unwrap_or("none"),
            self.browser_origin.as_deref().unwrap_or("none"),
        );
    }
}

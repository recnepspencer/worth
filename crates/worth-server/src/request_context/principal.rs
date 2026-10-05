#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorthServerAuthenticatedPrincipal {
    principal_id: String,
    admitted_transport_caller: Option<crate::WorthServerAdmittedTransportCaller>,
    application_authority_proof_identity: Option<String>,
}

impl WorthServerAuthenticatedPrincipal {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{
            option_string, string,
        };
        string(&self.principal_id)
            .saturating_add(option_string(&self.application_authority_proof_identity))
            .saturating_add(if self.admitted_transport_caller.is_some() {
                u64::MAX
            } else {
                0
            })
    }
    pub(crate) fn new(
        principal_id: String,
        admitted_transport_caller: Option<crate::WorthServerAdmittedTransportCaller>,
        application_authority_proof_identity: Option<String>,
    ) -> Self {
        Self {
            principal_id,
            admitted_transport_caller,
            application_authority_proof_identity,
        }
    }

    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    pub fn admitted_transport_caller(&self) -> Option<&crate::WorthServerAdmittedTransportCaller> {
        self.admitted_transport_caller.as_ref()
    }

    pub fn application_authority_proof_identity(&self) -> Option<&str> {
        self.application_authority_proof_identity.as_deref()
    }
}

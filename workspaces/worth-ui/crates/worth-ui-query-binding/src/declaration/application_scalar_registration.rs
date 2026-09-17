use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;

use super::{
    UiProjectionFieldRequirement, UiProjectionLifecycleRequirement, UiScalarSchemaRequirement,
    WorthUiQueryViewIdentity, WorthUiQueryViewIdentityError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiApplicationScalarProjectionRegistration {
    identity: WorthUiQueryViewIdentity,
    initial_query_receipt: WorthQueryApplicationQueryPublicationReceipt,
    requirement: UiScalarSchemaRequirement,
}

impl UiApplicationScalarProjectionRegistration {
    pub(crate) fn query_issued(
        publication: &crate::WorthUiStatusPublication,
    ) -> Result<Self, WorthUiQueryViewIdentityError> {
        Ok(Self {
            identity: WorthUiQueryViewIdentity::new(publication.value().identity.clone())?,
            initial_query_receipt: publication.query_receipt().clone(),
            requirement: UiScalarSchemaRequirement::text(
                UiProjectionFieldRequirement::query_text_status(),
                UiProjectionLifecycleRequirement::Live,
            ),
        })
    }

    pub fn identity(&self) -> &WorthUiQueryViewIdentity {
        &self.identity
    }

    pub fn requirement(&self) -> &UiScalarSchemaRequirement {
        &self.requirement
    }

    pub fn admits(&self, fact: &crate::UiApplicationScalarProjectionFactReceipt) -> bool {
        if self.identity != *fact.projection_identity() {
            return false;
        }
        let installed = self.initial_query_receipt.inspect();
        let candidate = fact.query_receipt().inspect();
        installed.query_identity() == candidate.query_identity()
            && installed.parameter_binding_identity() == candidate.parameter_binding_identity()
            && installed.basis().runtime_instance() == candidate.basis().runtime_instance()
            && installed.basis().branch() == candidate.basis().branch()
    }
}

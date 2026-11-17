use crate::projection_consumption::ApplicationProjectionInputIdentity;

use super::{
    UiProjectionFieldRequirement, UiProjectionLifecycleRequirement, UiScalarSchemaRequirement,
    WorthUiQueryViewIdentity, WorthUiQueryViewIdentityError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiApplicationScalarProjectionRegistration {
    identity: WorthUiQueryViewIdentity,
    publication_identity: ApplicationProjectionInputIdentity,
    requirement: UiScalarSchemaRequirement,
}

impl UiApplicationScalarProjectionRegistration {
    pub(crate) fn query_issued(
        publication: &crate::WorthUiStatusPublication,
    ) -> Result<Self, WorthUiQueryViewIdentityError> {
        Ok(Self {
            identity: WorthUiQueryViewIdentity::new(publication.value().identity.clone())?,
            publication_identity: ApplicationProjectionInputIdentity::capture(
                publication.query_receipt(),
            ),
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
        self.publication_identity
            .has_same_authority(fact.publication_identity())
    }
}

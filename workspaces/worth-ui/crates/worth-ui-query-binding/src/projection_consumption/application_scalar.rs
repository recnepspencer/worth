use super::ApplicationProjectionInputIdentity;
use worth_query_host::facade::publication::domain_computation::{
    WorthQueryPublishedApplicationBasis, WorthQueryPublishedApplicationQueryTerminalRelease,
};

use crate::{
    WorthUiQueryViewIdentity, WorthUiQueryViewIdentityError, WorthUiStatusPublication,
    WorthUiStatusQueryResult,
};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiApplicationScalarProjectionFactReceipt {
    projection_identity: WorthUiQueryViewIdentity,
    owner_order: u64,
    value: WorthUiStatusQueryResult,
    publication_identity: ApplicationProjectionInputIdentity,
    terminal_release: WorthQueryPublishedApplicationQueryTerminalRelease,
}

impl UiApplicationScalarProjectionFactReceipt {
    pub(crate) fn query_issued(
        publication: WorthUiStatusPublication,
    ) -> Result<Self, WorthUiQueryViewIdentityError> {
        let (value, query_receipt) = publication.into_parts();
        let projection_identity = WorthUiQueryViewIdentity::new(value.identity.clone())?;
        let owner_order = query_receipt.inspect().basis().version();
        let publication_identity = ApplicationProjectionInputIdentity::capture(&query_receipt);
        let terminal_release = query_receipt.inspect().terminal_release();
        Ok(Self {
            projection_identity,
            owner_order,
            value,
            publication_identity,
            terminal_release,
        })
    }

    pub fn projection_identity(&self) -> &WorthUiQueryViewIdentity {
        &self.projection_identity
    }

    pub fn owner_order(&self) -> u64 {
        self.owner_order
    }

    pub fn value(&self) -> &WorthUiStatusQueryResult {
        &self.value
    }

    /// Descriptive basis captured from the genuine publication; no read authority.
    pub fn publication_basis(&self) -> &WorthQueryPublishedApplicationBasis {
        self.publication_identity.basis()
    }

    pub fn terminal_resources_released(&self) -> bool {
        self.terminal_release.resources_released()
    }

    pub(crate) fn publication_identity(&self) -> &ApplicationProjectionInputIdentity {
        &self.publication_identity
    }

    pub fn into_observation(self) -> crate::UiApplicationScalarProjectionObservation {
        crate::UiApplicationScalarProjectionObservation::query_issued(self)
    }
}

impl WorthUiStatusPublication {
    pub fn into_projection_observation(
        self,
    ) -> Result<crate::UiProjectionObservation, WorthUiQueryViewIdentityError> {
        Ok(crate::UiProjectionObservation::ApplicationScalar(
            UiApplicationScalarProjectionFactReceipt::query_issued(self)?.into_observation(),
        ))
    }
}

#[cfg(test)]
mod tests;

use worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt;

use crate::{
    WorthUiQueryViewIdentity, WorthUiQueryViewIdentityError, WorthUiStatusPublication,
    WorthUiStatusQueryResult,
};

#[must_use]
#[derive(Debug, Eq, PartialEq)]
pub struct UiApplicationScalarProjectionFactReceipt {
    projection_identity: WorthUiQueryViewIdentity,
    owner_order: u64,
    value: WorthUiStatusQueryResult,
    query_receipt: WorthQueryApplicationQueryPublicationReceipt,
}

impl UiApplicationScalarProjectionFactReceipt {
    pub(crate) fn query_issued(
        publication: WorthUiStatusPublication,
    ) -> Result<Self, WorthUiQueryViewIdentityError> {
        let (value, query_receipt) = publication.into_parts();
        let projection_identity = WorthUiQueryViewIdentity::new(value.identity.clone())?;
        let owner_order = query_receipt.inspect().basis().version();
        Ok(Self {
            projection_identity,
            owner_order,
            value,
            query_receipt,
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

    pub fn query_receipt(&self) -> &WorthQueryApplicationQueryPublicationReceipt {
        &self.query_receipt
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

use worth_query::facade::{installed::collection, runtime};

use super::{
    UiCollectionProjectionRefreshError, UiCollectionProjectionRefreshOutcome,
    UiCollectionProjectionRefreshReceipt, UiLiveCollectionProjection,
};

impl UiLiveCollectionProjection {
    pub fn refresh(
        &mut self,
        execution: &worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
        workspace: &mut runtime::WorthQueryWorkspace,
    ) -> Result<UiCollectionProjectionRefreshOutcome, UiCollectionProjectionRefreshError> {
        let delivery = self
            .lease
            .drain(workspace)
            .map_err(|stop| UiCollectionProjectionRefreshError::Drain(Box::new(stop)))?;
        if delivery.delivery().is_empty() {
            return Ok(UiCollectionProjectionRefreshOutcome::NoSemanticDelivery);
        }
        let delta = self
            .lease
            .consumer_invalidation_delta(delivery)
            .map_err(|stop| UiCollectionProjectionRefreshError::Delta(Box::new(stop)))?;
        let admitted = self
            .lease
            .admit_consumer_invalidation_delta(delta, workspace)
            .map_err(|stop| UiCollectionProjectionRefreshError::Readmission(Box::new(stop)))?;
        self.consumer
            .bind_shared_target(&admitted, workspace)
            .map_err(|stop| UiCollectionProjectionRefreshError::Delivery(Box::new(stop)))?;
        let patch = match self.consumer.plan_patch(execution, &admitted, workspace) {
            collection::WorthQueryCollectionDeliveryOutcome::Patch(patch) => patch,
            collection::WorthQueryCollectionDeliveryOutcome::NoDelivery(stop) => {
                return Err(UiCollectionProjectionRefreshError::Delivery(Box::new(stop)));
            }
        };
        let receipt = self
            .consumer
            .apply_patch(patch)
            .map_err(|stop| UiCollectionProjectionRefreshError::Delivery(Box::new(stop)))?;
        let fact = crate::projection_consumption::derive_applied_collection_projection(
            crate::projection_consumption::UiCollectionDerivationContext {
                binding: &self.binding,
                consumer: &self.consumer,
                text_accesses: &self.text_accesses,
                application_item_key_access: self.application_item_key_access.as_ref(),
                budget: self.budget,
            },
            &receipt,
        );
        Ok(UiCollectionProjectionRefreshOutcome::Applied(
            UiCollectionProjectionRefreshReceipt {
                fact,
                query_work: crate::WorthUiCollectionQueryWorkInspection::from_query(
                    receipt.counters(),
                ),
            },
        ))
    }
}

impl std::fmt::Debug for UiCollectionProjectionRefreshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Drain(stop) => formatter.debug_tuple("Drain").field(stop.error()).finish(),
            Self::Delta(stop) => formatter.debug_tuple("Delta").field(&stop.kind()).finish(),
            Self::Readmission(stop) => formatter
                .debug_tuple("Readmission")
                .field(&stop.kind())
                .finish(),
            Self::Delivery(stop) => formatter
                .debug_tuple("Delivery")
                .field(&stop.kind())
                .finish(),
        }
    }
}

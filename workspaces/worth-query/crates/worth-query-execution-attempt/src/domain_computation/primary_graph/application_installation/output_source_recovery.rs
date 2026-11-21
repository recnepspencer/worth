//! One-use recovered program output source carrier.
use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryPerformedRelationalProductChange, WorthQueryProductPublicationReceipt,
};
use std::sync::Arc;

/// Query projection or history retirement. Missing parts remain owned and
/// refuse promotion; a partial transfer never drops an already-taken witness.
pub struct WorthQueryRecoveredProgramOutputSource {
    pub publication:
        crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    pub change: Option<Arc<WorthQueryPerformedRelationalProductChange>>,
    pub observation: Option<worth_runtime_world::facade::ProductBranchObservation>,
}

impl WorthQueryRecoveredProgramOutputSource {
    pub fn publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.publication
    }
    pub fn product_branch(&self) -> crate::basis::WorthQueryProductBranch {
        crate::basis::WorthQueryProductBranch::from_occurrence(
            self.publication.product_incarnation(),
        )
    }

    /// Join the original one-use delivery to the genuine admitted keyed receipt.
    /// A mismatch or shared delivery returns the complete original carrier.
    pub fn into_producer_delivery(
        mut self,
        publication: &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    ) -> Result<
        (
            WorthQueryPerformedRelationalProductChange,
            worth_runtime_world::facade::ProductBranchObservation,
            Self,
        ),
        Self,
    > {
        if self.publication.composite_commit() != publication.composite_commit()
            || self.publication.product_incarnation() != publication.product_incarnation()
            || !self.has_original_delivery()
            || self
                .observation
                .as_ref()
                .is_none_or(|observation| !self.publication.is_selected_at(observation))
        {
            return Err(self);
        }
        let change = self.change.take().expect("original delivery checked");
        match Arc::try_unwrap(change) {
            Ok(change) => Ok((
                change,
                self.observation
                    .take()
                    .expect("original observation checked"),
                self,
            )),
            Err(change) => {
                self.change = Some(change);
                Err(self)
            }
        }
    }

    /// A source can be promoted only while both original one-shot parts remain.
    pub fn has_original_delivery(&self) -> bool {
        self.change.is_some() && self.observation.is_some()
    }

    pub fn from_committed(
        receipt: &mut crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Self {
        let change = receipt
            .take_performed_relational_product_change()
            .map(Arc::new);
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation();
        Self {
            publication: receipt.committed_product_publication().clone(),
            change,
            observation,
        }
    }

    pub fn take_original(publication: &WorthQueryProductPublicationReceipt) -> Self {
        Self {
            publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(publication.clone()),
            change: publication.take_fresh_delivery().map(Arc::new),
            observation: publication.take_output_demand_observation(),
        }
    }
}

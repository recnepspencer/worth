//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_output_demand::required_work::ReplacedRequiredWorkHint;

impl SelectedRequiredWork {
    #[doc(hidden)]
    pub fn acknowledge_current_admitted_for_test<T>(
        self,
        proof: CurrentProof<T>,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        Self::acknowledge_current_admitted::<T>(self, proof)
    }
}

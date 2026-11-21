//! A correspondence cached only after its computation passed certification.
use crate::domain_computation::primary_graph::{
    output_lineage::CurrentComputation, WorthQueryApplicationOutputCorrespondence,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct CertifiedOutputCorrespondence
{
    _computation: CurrentComputation,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    role: String,
}
impl CertifiedOutputCorrespondence {
    pub fn new(
        computation: CurrentComputation,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
        role: String,
    ) -> Self {
        Self {
            _computation: computation,
            correspondence,
            role,
        }
    }
    pub fn pair(
        &self,
    ) -> (Arc<WorthQueryApplicationOutputCorrespondence>, String) {
        (Arc::clone(&self.correspondence), self.role.clone())
    }
}

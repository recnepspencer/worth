//! Passive observation of the existing record and wake final-owner charge.
use super::Ordering;
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Charged record/wake retention, including wakes outliving their registry row.
    /// This is neither total heap usage nor peak allocation and grants no authority.
    #[doc(hidden)]
    pub fn registry_record_retained_bytes_for_test(&self) -> usize {
        self.output_demands
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record_retained_bytes
            .load(Ordering::Acquire)
    }
}

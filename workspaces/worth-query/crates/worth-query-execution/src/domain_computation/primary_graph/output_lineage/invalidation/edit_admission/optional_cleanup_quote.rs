//! A failed pre-effect maintenance quote returns its exact conditional spend.
use super::InvalidationEditAdmission;
impl InvalidationEditAdmission {
    /// Only preparation may refuse: an admitted mutation receipt executes
    /// infallibly. This meter is request-local, so no concurrent claimant can
    /// change the carried totals during the quote. Success keeps every charge;
    /// an unsuccessful quote returns work, navigation and discarded scratch.
    pub(in crate::domain_computation::primary_graph) fn optional_cleanup_quote<T, E>(
        &mut self,
        prepare: impl FnOnce(&mut Self) -> Result<T, E>,
    ) -> Option<T> {
        let before = self.with_totals(|totals| *totals);
        match prepare(self) {
            Ok(value) => Some(value),
            Err(_) => {
                self.with_totals_mut(|totals| *totals = before);
                None
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};
    #[test]
    fn registry_optional_cleanup_quote_returns_failed_quote_counters() {
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(20),
            maximum_preparation_bytes: 100,
        });
        admission.charge_external_work(5).unwrap();
        admission.admit_read_scratch(7).unwrap();
        let quote =
            admission.optional_cleanup_quote(|meter| -> Result<(), CompanionPreflightStop> {
                meter.charge_ordered_operations(3, 13)?;
                meter.admit_read_scratch(8)?;
                meter.charge_external_work(30)?;
                Ok(())
            });
        assert!(quote.is_none());
        assert_eq!(admission.charged_work(), 5);
        assert_eq!(admission.charged_bytes(), 7);
        assert_eq!(admission.charged_navigation(), 0);
        let quote =
            admission.optional_cleanup_quote(|meter| -> Result<(), CompanionPreflightStop> {
                meter.charge_ordered_operations(3, 13)?;
                meter.admit_read_scratch(8)?;
                Ok(())
            });
        assert!(quote.is_some());
        assert_eq!(admission.charged_work(), 8);
        assert_eq!(admission.charged_bytes(), 15);
        assert_eq!(admission.charged_navigation(), 13);
    }
}

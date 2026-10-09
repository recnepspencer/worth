//! The request's logical and bounded ordered-navigation counters.
use super::*;

impl InvalidationEditAdmission {
    /// Navigation is only a keyed descent's path in a capacity-bounded tree.
    /// Payload, validation and iterated rows spend logical work. Every descent
    /// is limited to H; misclassified walks return the existing WorkExhausted.
    pub(in crate::domain_computation) fn charge_navigation(
        &mut self,
        units: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.record_navigation(1, units)
    }
    fn record_navigation(
        &mut self,
        operations: u64,
        navigation: u64,
    ) -> Result<(), CompanionPreflightStop> {
        let maximum = operations.saturating_mul(
            worth_relational::facade::indexes::SelectedIndexReadWork::MAXIMUM_ORDERED_DESCENT_WORK,
        );
        if navigation > maximum {
            return Err(CompanionPreflightStop::WorkExhausted {
                required: navigation,
                maximum,
            });
        }
        self.with_totals_mut(|totals| {
            totals.navigation = totals
                .navigation
                .checked_add(navigation)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            totals.ordered_operations = totals
                .ordered_operations
                .checked_add(operations)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            Ok(())
        })
    }
    pub(in crate::domain_computation) fn charge_ordered_operations(
        &mut self,
        operations: u64,
        navigation: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.work(operations)?;
        self.record_navigation(operations, navigation)
    }
    #[cfg(any(test, feature = "test-query-execution-observer"))]
    pub(in crate::domain_computation::primary_graph) fn charged_navigation(&self) -> u64 {
        self.with_totals(|totals| totals.navigation)
    }
    #[cfg(feature = "test-query-execution-observer")]
    pub(in crate::domain_computation::primary_graph) fn charged_ordered_operations(&self) -> u64 {
        self.with_totals(|totals| totals.ordered_operations)
    }
}

impl IndexAdmission for InvalidationEditAdmission {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        let maximum = self.budget.maximum_work_visits;
        self.with_totals_mut(|totals| {
            let required = totals
                .work
                .checked_add(visits)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            if required > maximum {
                return Err(CompanionPreflightStop::WorkExhausted { required, maximum });
            }
            totals.work = required;
            Ok(())
        })
    }

    fn navigation(&mut self, units: u64) -> Result<(), CompanionPreflightStop> {
        self.charge_navigation(units)
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        let maximum = self.budget.maximum_preparation_bytes;
        self.with_totals_mut(|totals| {
            let required = totals
                .bytes
                .checked_add(bytes)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            if required > maximum {
                return Err(CompanionPreflightStop::PreparationMemoryExhausted {
                    required,
                    maximum,
                });
            }
            totals.bytes = required;
            Ok(())
        })
    }
}

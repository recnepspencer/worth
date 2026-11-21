//! Borrowed field observation owns native work and stop precedence once.
use super::*;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
impl NativeRead<'_, '_> {
    pub fn observe_field(
        &mut self,
        identity: NativeIdentity<'_>,
        locator: &AspectFieldLocator,
        observe: &mut dyn FnMut(Option<&AspectValue>) -> Result<(), HandlerExecutionDenial>,
    ) -> Result<(), HandlerExecutionDenial> {
        if !self.budget.can_afford(1) {
            return Err(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ));
        }
        assert!(
            self.budget.consume(1),
            "the same owner preflighted one work unit"
        );
        self.scope.record(identity.entity_id);
        let reads = &mut self.work;
        match self.native.observe_field(
            self.execution,
            self.snapshot,
            identity.entity_id,
            identity.kind,
            locator,
            |raw| {
                reads.record_fields(1);
                observe(raw.value())
            },
        ) {
            Ok(Some(value)) => value,
            Ok(None) => Err(HandlerExecutionDenial::new(
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    "operation body field",
                ),
            )),
            Err(cause) => {
                self.stopped.get_or_insert(cause);
                Err(HandlerExecutionDenial::new(
                    WorthQueryApplicationAttemptDenial::from(cause),
                ))
            }
        }
    }
}

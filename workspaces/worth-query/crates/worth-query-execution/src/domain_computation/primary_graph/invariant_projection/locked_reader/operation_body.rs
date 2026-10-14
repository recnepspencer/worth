//! Native body scalar read at the existing projection's observation and budget.
use super::{
    ApplicationSchema, WorthQueryApplicationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::operation_body::BodyFieldExpectation;
use crate::domain_computation::primary_graph::operation_body::PreparedBodySource;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_scalar_matches<
        Entity,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        locator: &AspectFieldLocator,
        expected: BodyFieldExpectation<'_>,
    ) -> Result<bool, HandlerExecutionDenial> {
        if !self.work_budget.can_afford(1) {
            return Err(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ));
        }
        self.work_budget.consume(1);
        self.realized_scope.record(identity.entity_id);
        self.work.record_field();
        match crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed_admitted(
            self.runtime, self.execution, self.snapshot, identity.entity_id, identity.kind,
            locator, |value| expected.matches(value),
        ) {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(HandlerExecutionDenial::new(
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    "operation body field",
                ),
            )),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                Err(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::from(cause)))
            }
        }
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_scalar_capture<
        Entity,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        locator: &AspectFieldLocator,
        capture: impl FnOnce(&AspectValue) -> Result<PreparedBodySource, HandlerExecutionDenial>,
    ) -> Result<Option<PreparedBodySource>, HandlerExecutionDenial> {
        if !self.work_budget.can_afford(1) {
            return Err(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ));
        }
        self.work_budget.consume(1);
        self.realized_scope.record(identity.entity_id);
        self.work.record_field();
        match crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed_admitted(
            self.runtime, self.execution, self.snapshot, identity.entity_id, identity.kind,
            locator, |value| value.map(capture).transpose(),
        ) {
            Ok(value) => value.unwrap_or(Ok(None)),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                Err(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::from(cause)))
            }
        }
    }
}

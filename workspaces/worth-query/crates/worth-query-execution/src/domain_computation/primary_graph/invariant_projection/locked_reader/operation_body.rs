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
        self.body_scalar_observe(identity, locator, |value| Ok(expected.matches(value)))
    }

    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_scalar_observe<
        Entity,
        Output,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        locator: &AspectFieldLocator,
        observe: impl FnOnce(Option<&AspectValue>) -> Result<Output, HandlerExecutionDenial>,
    ) -> Result<Output, HandlerExecutionDenial> {
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
            locator, observe,
        ) {
            Ok(Some(value)) => value,
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
        self.body_scalar_observe(identity, locator, |value| value.map(capture).transpose())
    }
}

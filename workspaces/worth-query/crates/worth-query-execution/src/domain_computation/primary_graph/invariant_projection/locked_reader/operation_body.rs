//! Native body scalar read at the existing projection's observation and budget.
use super::{
    ApplicationSchema, WorthQueryApplicationInvariantProjectionReader,
    WorthQueryInvariantEntityIdentity,
};
use crate::domain_computation::primary_graph::operation_body::PreparedBodySource;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryAdvancementDenial, WorthQueryApplicationAttemptDenial,
    WorthQueryManagedComputationResourceDenial,
};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_scalar_matches<
        Entity,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        locator: &AspectFieldLocator,
        expected: Option<&AspectValue>,
    ) -> bool {
        if !self.work_budget.can_afford(1) {
            return false;
        }
        self.work_budget.consume(1);
        self.realized_scope.record(identity.entity_id);
        self.work.record_field();
        match crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed_admitted(
            self.runtime, self.execution, self.snapshot, identity.entity_id, identity.kind,
            locator, |value| expected.is_none_or(|expected| value == Some(expected)),
        ) {
            Ok(value) => value.unwrap_or(false),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                false
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

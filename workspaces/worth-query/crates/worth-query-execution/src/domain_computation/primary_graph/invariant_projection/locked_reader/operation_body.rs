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
use worth_relational::facade::identity::{EntityId, KindId};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Same canonical equivalence owner as native signing consequences. Both present
    /// values are funded before encoding; absence is observed by the ordinary reader.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn body_scalar_equivalent<
        Entity,
    >(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        locator: &AspectFieldLocator,
        expected: Option<&AspectValue>,
        required: bool,
        execution: &crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>,
    ) -> Result<bool, HandlerExecutionDenial> {
        use worth_relational::facade::storage::AuthoritativeFieldComparisonKey as Comparison;
        let resource =
            |cause| HandlerExecutionDenial::new(WorthQueryAdvancementDenial::Resource(cause));
        execution
            .decision_stage(|context| {
                self.body_scalar_observe(identity, locator, |observed| {
                    if required && observed.is_none() {
                        return Err(HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
                            WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                            "reuse predecessor field",
                        )));
                    }
                    let (Some(observed), Some(expected)) = (observed, expected) else {
                        return Ok(observed.is_none() && expected.is_none());
                    };
                    let bytes = Comparison::required_encoded_capacity_bytes(observed)
                        .and_then(|n| n.checked_add(Comparison::required_encoded_capacity_bytes(expected)?))
                        .ok_or_else(|| resource(WorthQueryManagedComputationResourceDenial::CapacityOverflow))?;
                    let _memory = execution.reserve(bytes).map_err(resource)?;
                    context.checkpoint(bytes).map_err(|cause| {
                        use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
                        HandlerExecutionDenial::new(advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause)))
                    })?;
                    Ok(Comparison::from_aspect_value(observed) == Comparison::from_aspect_value(expected))
                })
            })
            .map_err(HandlerExecutionDenial::new)?
    }
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
        let mut output = None;
        let mut observe = Some(observe);
        self.observe_named_body_scalar(
            identity.entity_id,
            identity.kind,
            locator,
            &mut |raw| assign_body_output(&mut observe, &mut output, raw),
        )?;
        Ok(output.expect("body scalar observation stored its output"))
    }

    /// Work exhaustion and a missing fact stay errors. The caller's value stays in the shell.
    #[inline(never)]
    fn observe_named_body_scalar(
        &mut self,
        entity_id: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
        observe: &mut dyn for<'raw> FnMut(
            Option<&'raw AspectValue>,
        ) -> Result<(), HandlerExecutionDenial>,
    ) -> Result<(), HandlerExecutionDenial> {
        if !self.work_budget.can_afford(1) {
            return Err(HandlerExecutionDenial::new(
                WorthQueryAdvancementDenial::Resource(
                    WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ));
        }
        assert!(
            self.work_budget.consume(1),
            "the same owner preflighted one work unit"
        );
        self.realized_scope.record(entity_id);
        let observed = {
            let reads = &mut self.work;
            crate::domain_computation::primary_graph::application_attempt::observe_field_value_borrowed_admitted(
                self.runtime,
                self.execution,
                self.snapshot,
                entity_id,
                kind,
                locator,
                |raw| {
                    reads.record_fields(1);
                    observe(raw)
                },
            )
        };
        match observed {
            Ok(Some(Ok(()))) => Ok(()),
            Ok(Some(Err(error))) => Err(error),
            Ok(None) => Err(HandlerExecutionDenial::new(
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    "operation body field",
                ),
            )),
            Err(cause) => {
                self.native_read_stop.get_or_insert(cause);
                Err(HandlerExecutionDenial::new(
                    WorthQueryApplicationAttemptDenial::from(cause),
                ))
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

fn assign_body_output<'raw, Output>(
    observe: &mut Option<
        impl for<'value> FnOnce(Option<&'value AspectValue>) -> Result<Output, HandlerExecutionDenial>,
    >,
    output: &mut Option<Output>,
    raw: Option<&'raw AspectValue>,
) -> Result<(), HandlerExecutionDenial> {
    let observe = observe.take().expect("single body scalar observation");
    *output = Some(observe(raw)?);
    Ok(())
}

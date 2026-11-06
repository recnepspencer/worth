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
        self.native_read()
            .observe_field(identity.native_identity(), locator, &mut |raw| {
                output = Some(observe.take().expect("one physical field observation")(
                    raw,
                )?);
                Ok(())
            })?;
        Ok(output.expect("a completed physical read invoked its observer"))
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

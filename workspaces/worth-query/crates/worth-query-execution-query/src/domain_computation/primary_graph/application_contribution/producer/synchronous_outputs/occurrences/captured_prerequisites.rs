//! Every fixed captured port drains its complete typed parent roster.
use super::*;
use worth_query_declaration::facade::application_program::{
    ApplicationCapturedPrerequisites, WorthQueryProducerInputCaptureDefinition,
};

impl<S, C, Capture> PrerequisiteExecution<S, C> for ApplicationCapturedPrerequisites<Capture>
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    Capture: WorthQueryProducerInputCaptureDefinition<S>,
    Binding<S, C>: WorthQueryApplicationDependentOutputConnection<S>,
    Parent<S, C>: Clone + Send,
    Child<S, C>: WorthQueryApplicationOutputDemand<S, OutputFamily = Capture::OutputFamily>,
    source::SourceInput<S, Parent<S, C>>:
        ApplicationQueryIntent<S, Binding = source::Source<S, Parent<S, C>>>,
    <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    fn incoming(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        _candidate: &RelationalAuthoredCoupledCandidate<'_>,
        _pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        inputs: &PrerequisiteInputs<'_>,
        next: &mut SynchronousOutputWave<S>,
        whole: &'static ComponentExecution<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        if !demand.is::<Child<S, C>>() {
            if let super::super::input_preparation::InputState::Captured(inputs) = inputs.0
                && inputs.definition() == crate::equality_type_id::EqualityTypeId::of::<Capture>()
            {
                return Err(refusal(
                    Binding::<S, C>::IDENTITY,
                    "captured target demand differs",
                ));
            }
            return Ok(());
        }
        let connection = Binding::<S, C>::IDENTITY;
        let captured = match inputs.0 {
            super::super::input_preparation::InputState::Ready => return Ok(()),
            super::super::input_preparation::InputState::Captured(inputs) => inputs,
            super::super::input_preparation::InputState::Stored => {
                return Err(refusal(connection, "captured input preparation missing"))
            }
        };
        if captured.definition() != crate::equality_type_id::EqualityTypeId::of::<Capture>()
            || captured.affinity().source != source
            || captured.affinity().component != whole.identity()
        {
            return Err(refusal(connection, "foreign captured input preparation"));
        }
        let port = C::declaration()
            .input_capture_port_type()
            .ok_or_else(|| refusal(connection, "captured input port missing"))?;
        for parent in captured.parents() {
            context.checkpoint(connection)?;
            if parent.connection != port {
                continue;
            }
            let demand = parent
                .demand
                .downcast_ref::<Parent<S, C>>()
                .ok_or_else(|| refusal(connection, "foreign captured parent demand"))?;
            capture::schedule_continued(
                context,
                next,
                connection,
                C::declaration(),
                demand.clone(),
                whole,
                Some(parent.source),
            )?;
        }
        Ok(())
    }
}

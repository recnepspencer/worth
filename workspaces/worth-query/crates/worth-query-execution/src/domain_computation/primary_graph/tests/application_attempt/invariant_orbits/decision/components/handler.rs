//! An installed ordinary handler consumes the scoped component selection API.
use super::{
    super::{
        super::schema::*,
        binding::{CheckOrbitDecision, CheckOrbitOutcome},
    },
    binding::*,
};
use crate::domain_computation::primary_graph::application_entry::mutation::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap;
struct CheckComponentsHandler;
impl<S: OrbitModel + Clone> OperationHandler<S, CheckComponentsBinding<S>>
    for CheckComponentsHandler
{
    fn decide(
        &self,
        input: &CheckComponentsInput,
        reader: &mut DecisionReader<'_, '_, '_, S, CheckComponentsBinding<S>>,
    ) -> HandlerResult<CheckOrbitDecision, CheckOrbitOutcome> {
        let result = (|| {
            let root = reader.scope().clone();
            let component = reader
                .component_of(Shells::reference(), &root)
                .map_err(HandlerExecutionDenial::new)?;
            let census = reader
                .components(Shells::reference())
                .map_err(HandlerExecutionDenial::new)?;
            let _count = reader
                .component_count(Shells::reference())
                .map_err(HandlerExecutionDenial::new)?;
            // Membership comes from the same original Native cohort, without a
            // local graph walk or a fabricated component identity.
            let count = census
                .iter()
                .filter(|item| item.identity() == component.identity())
                .map(|item| item.len() as u64)
                .sum();
            Ok::<_, HandlerExecutionDenial>(CheckOrbitDecision {
                root: input.root,
                count,
            })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
    fn build_candidate(
        &self,
        _: &CheckComponentsInput,
        decision: CheckOrbitDecision,
        _: &mut CandidateWriter<'_, S, CheckComponentsBinding<S>>,
    ) -> HandlerResult<CheckOrbitOutcome, CheckOrbitOutcome> {
        HandlerResult::Completed(CheckOrbitOutcome(decision.count))
    }
}
pub(in super::super::super) fn install<S: OrbitModel + Clone>(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<S>,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<S>,
) {
    let binding = schema
        .installed_mutation_binding::<CheckComponentsBinding<S>>()
        .unwrap();
    bootstrap
        .install_handler(&binding, CheckComponentsHandler)
        .unwrap();
}

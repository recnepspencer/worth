//! Both read values travel through the ordinary installed handler and both seals.
use super::{super::schema::*, binding::*};
use crate::domain_computation::primary_graph::application_entry::mutation::*;
pub(super) struct CheckOrbitHandler;
impl<S: OrbitModel + Clone> OperationHandler<S, CheckOrbitBinding<S>> for CheckOrbitHandler {
    fn decide(
        &self,
        input: &CheckOrbitInput,
        reader: &mut DecisionReader<'_, '_, '_, S, CheckOrbitBinding<S>>,
    ) -> HandlerResult<CheckOrbitDecision, CheckOrbitOutcome> {
        let result = (|| {
            let root = reader.scope().clone();
            reader.field(&root, NodeKey::reference())?;
            reader.field(&root, Marked::reference())?;
            let orbit = reader
                .orbit(Next::reference(), &root)
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(
                reader
                    .orbit_of(Next::reference(), &root)
                    .map_err(HandlerExecutionDenial::new)?,
                *orbit.identity()
            );
            // Privileged inspection is available to a handler, but the raw
            // reader marks opaque context and grants no decision reuse proof.
            let history = reader
                .reader()
                .lineage(Next::reference(), orbit.identity())
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(history.identity(), orbit.identity());
            assert_eq!(history.latest_identity().as_ref(), Some(orbit.identity()));
            assert_eq!(history.retired_at(), None);
            Ok::<_, HandlerExecutionDenial>(CheckOrbitDecision {
                root: input.root,
                count: orbit.len() as u64,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
    fn build_candidate(
        &self,
        _: &CheckOrbitInput,
        decision: CheckOrbitDecision,
        candidate: &mut CandidateWriter<'_, S, CheckOrbitBinding<S>>,
    ) -> HandlerResult<CheckOrbitOutcome, CheckOrbitOutcome> {
        let result = (|| {
            let root = candidate
                .resolve_entity(NodeKey::reference(), decision.root)
                .map_err(HandlerExecutionDenial::new)?;
            candidate
                .write_field(&root, Marked::reference(), true)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(CheckOrbitOutcome(decision.count))
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
}
pub(in super::super) fn install<S: OrbitModel + Clone>(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<S>,
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<S>,
) {
    let installed = schema
        .installed_mutation_binding::<CheckOrbitBinding<S>>()
        .unwrap();
    bootstrap
        .install_handler(&installed, CheckOrbitHandler)
        .unwrap();
}

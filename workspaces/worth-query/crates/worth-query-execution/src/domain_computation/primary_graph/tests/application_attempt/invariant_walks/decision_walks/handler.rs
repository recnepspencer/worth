//! A real installed handler uses each complete-read method as a decision input.
use super::{super::schema::*, binding::*};
use crate::domain_computation::primary_graph::application_entry::mutation::*;
use worth_query_declaration::facade::application_schema::RelationDirection::Forward;
pub(super) struct CheckWalkHandler;
impl OperationHandler<WalkSchema, CheckWalkBinding> for CheckWalkHandler {
    fn decide(
        &self,
        input: &CheckWalkInput,
        reader: &mut DecisionReader<'_, '_, '_, WalkSchema, CheckWalkBinding>,
    ) -> HandlerResult<CheckWalkDecision, CheckWalkOutcome> {
        let result = (|| {
            let root = reader.scope().clone();
            reader.field(&root, NodeKey::reference())?;
            reader.field(&root, Active::reference())?;
            let walk = reader
                .walk(Next::reference(), Forward, &root)
                .map_err(HandlerExecutionDenial::new)?;
            let last = walk.entities().last().expect("the valid root is visited");
            let duplicate = reader
                .closure(Next::reference(), Forward, &[root.clone(), root.clone()])
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(walk.len(), duplicate.len());
            assert!(reader
                .reaches(Next::reference(), &root, &last)
                .map_err(HandlerExecutionDenial::new)?);
            Ok::<_, HandlerExecutionDenial>(CheckWalkDecision {
                root: input.root,
                visited: walk.len() as u64,
            })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
    fn build_candidate(
        &self,
        _: &CheckWalkInput,
        decision: CheckWalkDecision,
        candidate: &mut CandidateWriter<'_, WalkSchema, CheckWalkBinding>,
    ) -> HandlerResult<CheckWalkOutcome, CheckWalkOutcome> {
        let result = (|| {
            let root = candidate
                .resolve_entity(NodeKey::reference(), decision.root)
                .map_err(HandlerExecutionDenial::new)?;
            candidate
                .write_field(&root, Active::reference(), true)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(CheckWalkOutcome(decision.visited))
        })();
        match result {
            Ok(outcome) => HandlerResult::Completed(outcome),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
}
pub(in super::super) fn install(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<WalkSchema>,
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<
        WalkSchema,
    >,
) {
    let installed = schema
        .installed_mutation_binding::<CheckWalkBinding>()
        .unwrap();
    bootstrap
        .install_handler(&installed, CheckWalkHandler)
        .unwrap();
}

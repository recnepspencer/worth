//! A real installed handler uses each complete-read method as a decision input.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::schema::*;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks::decision_walks::binding::*;
use worth_query_execution::facade::domain_computation::CandidateWriter;
use worth_query_execution::facade::domain_computation::DecisionReader;
use worth_query_execution::facade::domain_computation::HandlerExecutionDenial;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_query_execution::facade::domain_computation::OperationHandler;
use worth_query_declaration::facade::application_schema::RelationDirection::Forward;
pub struct CheckWalkHandler;
impl OperationHandler<WalkSchema, CheckWalkBinding> for CheckWalkHandler {
    fn decide(
        &self,
        input: &CheckWalkInput,
        reader: &mut DecisionReader<'_, '_, '_, WalkSchema, CheckWalkBinding>,
    ) -> HandlerResult<CheckWalkDecision, CheckWalkOutcome> {
        let result = (|| {
            let root = reader.scope().clone();
            reader.field(&root, NodeKey::at("NodeKey"))?;
            reader.field(&root, Active::at("Active"))?;
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
                .resolve_entity(NodeKey::at("NodeKey"), decision.root)
                .map_err(HandlerExecutionDenial::new)?;
            candidate
                .write_field(&root, Active::at("Active"), true)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(CheckWalkOutcome(decision.visited))
        })();
        match result {
            Ok(outcome) => HandlerResult::Completed(outcome),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
}
pub fn install(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<WalkSchema>,
    bootstrap: &mut worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap<
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

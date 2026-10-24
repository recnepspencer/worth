//! The declared correction uses the same handler as the authored definition edit.
use super::{
    composite_definition_contract::ReadyNode,
    composite_definition_model::*,
    composite_definition_runtime::{Clock, Conditional, Predicate},
    schema::*,
};
use worth_query_host::facade::primary_graph::*;
pub struct Handler;
static DECISIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static CANDIDATES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn handler_contacts() -> (usize, usize) {
    (
        DECISIONS.load(std::sync::atomic::Ordering::Relaxed),
        CANDIDATES.load(std::sync::atomic::Ordering::Relaxed),
    )
}
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &Change,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<Decision, Missing> {
        DECISIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result = (|| {
            let root = reader.resolve_entity(Key::reference(), input.key.clone())?;
            reader.field(&root, Primary::reference())?;
            let target = reader.mutation_target(&root)?;
            let definition = reader
                .prepare_conditional_definition::<Conditional, ReadyNode, Clock, Predicate>(
                    std::sync::Arc::new(Predicate(input.value)),
                )?;
            Ok::<_, HandlerExecutionDenial>(Decision { target, definition })
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Change,
        decision: Decision,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Changed, Missing> {
        CANDIDATES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let result = (|| {
            let target = writer.projected_entity(&decision.target)?;
            writer.write_field(&target, Primary::reference(), input.value)?;
            writer.advance_conditional_definition(decision.definition)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

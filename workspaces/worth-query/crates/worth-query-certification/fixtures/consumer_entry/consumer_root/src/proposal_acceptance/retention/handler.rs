//! Scope and membership are observed before minting the actual mutation target.
use super::super::schema::data::Coordinates;
use super::*;

fn coordinates<Entity>(entity: &WorthQueryInvariantEntityIdentity<Schema, Entity>) -> Coordinates {
    let id = entity.entity_id();
    Coordinates {
        partition: id.partition_value(),
        slot: id.local_slot_value(),
        generation: id.generation_value(),
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &RetentionEdit,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Target>, Denial> {
        // This ordinary opaque read preserves the owner's decision-reuse restriction.
        if reader.reader().runtime_instance_id() != input.model.runtime {
            return HandlerResult::DomainDenied(Denial::ForeignRuntime);
        }
        let model = reader.scope().clone();
        if coordinates(&model) != input.model.occurrence {
            return HandlerResult::DomainDenied(Denial::ModelOccurrence);
        }
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            reader.field(&model, ModelName::reference())?;
            for relation in reader
                .relations_from(ModelTarget::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                let target = relation.to();
                if coordinates(target) == input.target.occurrence {
                    if reader.field(target, TargetName::reference())?
                        != Some(input.target.label.clone())
                    {
                        return Ok(None);
                    }
                    reader.field(target, Retention::reference())?;
                    return reader.mutation_target(target).map(Some);
                }
            }
            Ok(None)
        })();
        match observed {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(Denial::TargetMembership),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &RetentionEdit,
        decision: WorthQueryInvariantMutationTarget<Schema, Target>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let effect = writer
            .projected_entity(&decision)
            .map_err(HandlerExecutionDenial::new)
            .and_then(|target| {
                writer
                    .write_field(&target, Retention::reference(), input.days)
                    .map_err(HandlerExecutionDenial::new)
            });
        match effect {
            Ok(()) => HandlerResult::Completed(Changed { days: input.days }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

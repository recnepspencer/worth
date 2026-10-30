//! Complete observed claim/result facts bind publication to the original issued claim.
use super::super::super::schema::data::Coordinates;
use super::*;
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &RecordResult,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Model>, Denial> {
        let model = reader.scope().clone();
        let id = model.entity_id();
        let model_ref = &input.data.claim.data.input.proposal.model;
        if reader.reader().runtime_instance_id() != model_ref.runtime
            || model_ref.occurrence
                != (Coordinates {
                    partition: id.partition_value(),
                    slot: id.local_slot_value(),
                    generation: id.generation_value(),
                })
        {
            return HandlerResult::DomainDenied(Denial::ModelMismatch);
        }
        let principal = reader.operation_scope_binding().principal();
        if input.data.claim.data.principal
            != (Coordinates {
                partition: principal.partition_id(),
                slot: principal.local_slot(),
                generation: principal.generation(),
            })
        {
            return HandlerResult::DomainDenied(Denial::PrincipalMismatch);
        }
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            if reader.field(&model, ModelName::reference())? != Some(model_ref.label.clone()) {
                return Ok(Err(Denial::ModelMismatch));
            }
            let mut matches = 0;
            for relation in reader
                .relations_from(ModelClaim::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                let claim = relation.to();
                let id = claim.entity_id();
                let occurrence = Coordinates {
                    partition: id.partition_value(),
                    slot: id.local_slot_value(),
                    generation: id.generation_value(),
                };
                if occurrence == input.data.claim.occurrence {
                    if reader.field(claim, ClaimData::reference())?.as_ref()
                        != Some(&input.data.claim.data)
                    {
                        return Ok(Err(Denial::ClaimMismatch));
                    }
                    matches += 1;
                }
            }
            if matches != 1 {
                return Ok(Err(Denial::ClaimMismatch));
            }
            for relation in reader
                .relations_from(ModelResult::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                if let Some(result) = reader.field(relation.to(), ResultData::reference())? {
                    if result.claim.occurrence == input.data.claim.occurrence {
                        return Ok(Err(Denial::AlreadyRecorded));
                    }
                    if result.claim.data.input.attempt == input.data.claim.data.input.attempt {
                        return Ok(Err(Denial::ConflictingResult));
                    }
                }
            }
            reader.mutation_target(&model).map(Ok)
        })();
        match observed {
            Ok(Ok(target)) => HandlerResult::Completed(target),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &RecordResult,
        decision: WorthQueryInvariantMutationTarget<Schema, Model>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Recorded, Denial> {
        let applied = (|| -> Result<(), HandlerExecutionDenial> {
            let model = writer
                .projected_entity(&decision)
                .map_err(HandlerExecutionDenial::new)?;
            let key = format!("result-{}", input.data.claim.data.input.attempt);
            let result = writer
                .create_entity_in_context(
                    &model,
                    AcceptanceResult::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&result, ResultName::reference(), key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&result, ResultData::reference(), input.data.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(ModelResult::reference(), key, &model, &result)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match applied {
            Ok(()) => HandlerResult::Completed(Recorded),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

//! Native observed membership/body facts protect claim creation and concurrent absence.
use super::super::super::schema::data::Coordinates;
use super::*;
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &AttemptInput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        if *reader.idempotency_key() != input.attempt {
            return HandlerResult::DomainDenied(Denial::AttemptKeyMismatch);
        }
        if input.proposal.format_version != 1 {
            return HandlerResult::DomainDenied(Denial::UnsupportedVersion);
        }
        if input.proposal.intents.is_empty() {
            return HandlerResult::DomainDenied(Denial::EmptyProposal);
        }
        let model = reader.scope().clone();
        let id = model.entity_id();
        let occurrence = Coordinates {
            partition: id.partition_value(),
            slot: id.local_slot_value(),
            generation: id.generation_value(),
        };
        if reader.reader().runtime_instance_id() != input.proposal.model.runtime
            || occurrence != input.proposal.model.occurrence
            || input.reference.model != input.proposal.model
        {
            return HandlerResult::DomainDenied(Denial::ModelMismatch);
        }
        let observed = (|| -> Result<_, HandlerExecutionDenial> {
            if reader.field(&model, ModelName::reference())?
                != Some(input.proposal.model.label.clone())
            {
                return Ok(Err(Denial::ModelMismatch));
            }
            let mut matching = 0;
            for relation in reader
                .relations_from(ModelProposal::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                let proposal = relation.to();
                let id = proposal.entity_id();
                let occurrence = Coordinates {
                    partition: id.partition_value(),
                    slot: id.local_slot_value(),
                    generation: id.generation_value(),
                };
                if occurrence == input.reference.occurrence {
                    let name = reader.field(proposal, ProposalName::reference())?;
                    let data = reader.field(proposal, ProposalData::reference())?;
                    if name != Some(input.reference.key.clone())
                        || data.as_ref() != Some(&input.proposal)
                        || input.reference.revision != input.proposal.revision
                    {
                        return Ok(Err(Denial::ProposalChanged));
                    }
                    matching += 1;
                }
            }
            if matching != 1 {
                return Ok(Err(Denial::ProposalChanged));
            }
            let mut existing = 0;
            for relation in reader
                .relations_from(ModelClaim::reference(), &model)
                .map_err(HandlerExecutionDenial::new)?
            {
                if reader
                    .field(relation.to(), ClaimData::reference())?
                    .is_some_and(|data| data.input.attempt == input.attempt)
                {
                    existing += 1;
                }
            }
            if existing > 1 {
                return Ok(Err(Denial::AmbiguousClaim));
            }
            if existing == 1 {
                return Ok(Err(Denial::AlreadyClaimed));
            }
            let principal = reader.operation_scope_binding().principal();
            Ok(Ok(Decision {
                model: reader.mutation_target(&model)?,
                principal: Coordinates {
                    partition: principal.partition_id(),
                    slot: principal.local_slot(),
                    generation: principal.generation(),
                },
            }))
        })();
        match observed {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &AttemptInput,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Registered, Denial> {
        let applied = (|| -> Result<(), HandlerExecutionDenial> {
            let model = writer
                .projected_entity(&decision.model)
                .map_err(HandlerExecutionDenial::new)?;
            let key = format!("attempt-{}", input.attempt);
            let claim = writer
                .create_entity_in_context(
                    &model,
                    AcceptanceClaim::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&claim, ClaimName::reference(), key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &claim,
                    ClaimData::reference(),
                    StoredClaim {
                        input: input.clone(),
                        principal: decision.principal,
                    },
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(ModelClaim::reference(), key, &model, &claim)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match applied {
            Ok(()) => HandlerResult::Completed(Registered),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

//! Generation/frontier sealing is an ordinary authored stage over proposed data.
use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Intent {
    pub generation: u64,
    pub frontier: u64,
    pub dependent: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Denial {
    GenerationChanged,
    FrontierChanged,
    DependentChanged,
}
#[derive(Clone, Debug, Eq, PartialEq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Sealed {
    pub generation: u64,
    pub frontier: u64,
    pub dependent: u64,
    pub first_child: u64,
    pub ordinal: usize,
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for Intent { identity: "owned-outputs.interleaved.seal" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "owned-outputs.interleaved.seal-denial" });
worth_query_structured_value_binding!(pub ResultBinding for Sealed { identity: "owned-outputs.interleaved.seal-result" });
worth_query_operation!(pub Seal for Schema, input Input; access {
    read_fields: [MemberKey::at("MemberKey"), EnvelopeDependent::at("Dependent"), EnvelopeGeneration::at("Generation"), EnvelopeFrontier::at("Frontier"), SecondKey::at("SecondKey"), SecondValue::at("SecondValue")];
    writes: [EnvelopeGeneration::at("Generation"), EnvelopeFrontier::at("Frontier")];
});
fn scope(_: &Intent) -> String {
    "progress:envelope-calibration".into()
}
worth_query_mutation_binding!(pub Binding for Intent, schema Schema,
    identity "owned-outputs.interleaved.seal", input Input, operation Seal, result ResultBinding,
    idempotency String, identity "owned-outputs.interleaved.seal", decision Decision, denial DenialBinding,
    handler identity "owned-outputs.interleaved.seal", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope MemberKey, field MemberKey::at("MemberKey"), value scope,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Member>,
    result: Sealed,
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Intent,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let decide = (|| {
            let target = reader.resolve_entity(MemberKey::at("MemberKey"), scope(input))?;
            let generation = reader
                .field(&target, EnvelopeGeneration::at("Generation"))?
                .unwrap();
            let frontier = reader
                .field(&target, EnvelopeFrontier::at("Frontier"))?
                .unwrap();
            if generation != input.generation {
                return Ok(Err(Denial::GenerationChanged));
            }
            if frontier != input.frontier {
                return Ok(Err(Denial::FrontierChanged));
            }
            let dependent =
                reader.resolve_entity(MemberKey::at("MemberKey"), "progress:m".to_owned())?;
            let dependent = reader
                .field(&dependent, EnvelopeDependent::at("Dependent"))?
                .unwrap();
            if dependent != input.dependent {
                return Ok(Err(Denial::DependentChanged));
            }
            let child = reader
                .resolve_entity(SecondKey::at("SecondKey"), "second:dependent:0".to_owned())?;
            let first_child = reader
                .field(&child, SecondValue::at("SecondValue"))?
                .unwrap();
            Ok::<_, HandlerExecutionDenial>(Ok(Decision {
                target: reader.mutation_target(&target)?,
                result: Sealed {
                    generation: generation + 1,
                    frontier: frontier + 1,
                    dependent,
                    first_child,
                    ordinal: reader.stage_ordinal().unwrap(),
                    key: reader.idempotency_key().clone(),
                },
            }))
        })();
        match decide {
            Ok(Ok(value)) => HandlerResult::Completed(value),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Intent,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Sealed, Denial> {
        let write = (|| {
            let target = writer.projected_entity(&decision.target)?;
            writer.write_field(
                &target,
                EnvelopeGeneration::at("Generation"),
                decision.result.generation,
            )?;
            writer.write_field(
                &target,
                EnvelopeFrontier::at("Frontier"),
                decision.result.frontier,
            )
        })();
        match write {
            Ok(()) => HandlerResult::Completed(decision.result),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Seal>(
            Seal::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}

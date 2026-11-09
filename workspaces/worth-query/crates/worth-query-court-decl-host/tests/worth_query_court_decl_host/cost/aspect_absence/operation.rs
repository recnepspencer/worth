#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Mode {
    Present,
    Absent,
    Discover,
}
#[derive(Clone, Debug)]
pub struct Copied(pub u64);
worth_query_structured_value_binding!(pub ResultBinding for Copied { identity: "class-read-cost.result.v1" });
#[derive(Clone, Debug)]
pub enum Denial {
    Absent,
    NotOneClass,
}
worth_query_structured_value_binding!(pub Input for Mode { identity: "class-read-cost.input.v1" });
worth_query_structured_value_binding!(pub Refusal for Denial { identity: "class-read-cost.denial.v1" });
worth_query_operation!(pub CopySelected for Schema, input Input);

fn scope(_: &Mode) -> String {
    "record".into()
}
worth_query_mutation_binding!(pub Binding for Mode, schema Schema,
    identity "class-read-cost.binding.v1", input Input, operation CopySelected, result ResultBinding,
    idempotency u64, identity "class-read-cost.command.v1", decision Decision, denial Refusal,
    handler identity "class-read-cost.handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value scope,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Record>,
    value: u64,
}
impl ApplicationDeclaredOperation<Schema> for CopySelected {
    fn declare_access(
        mut schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema = schema
            .operation_projection_work_budget(Self::reference(), 32)
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), Sink::at("Sink"))
            .operation_write(Self::reference(), Sink::at("Sink"));
        schema = class_access::grant(schema);
        schema
    }
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        mode: &Mode,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            let root = reader.resolve_entity(Key::at("Key"), "record".to_owned())?;
            let value = match mode {
                Mode::Present => reader.field(&root, Value3::at("value"))?,
                Mode::Absent => reader.field(&root, Value1::at("value"))?,
                Mode::Discover => {
                    let mut found = None;
                    let mut count = 0;
                    class_access::probe(reader, &root, &mut found, &mut count)?;
                    if count != 1 {
                        return Ok(Err(Denial::NotOneClass));
                    }
                    found
                }
            };
            let Some(value) = value else {
                return Ok(Err(Denial::Absent));
            };
            reader.field(&root, Sink::at("Sink"))?;
            Ok(Ok(Decision {
                target: reader.mutation_target(&root)?,
                value,
            }))
        })();
        match result {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(denial)) => HandlerResult::DomainDenied(denial),
            Err(denial) => HandlerResult::ExecutionDenied(denial),
        }
    }
    fn build_candidate(
        &self,
        _: &Mode,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Copied, Denial> {
        let result = writer
            .projected_entity(&decision.target)
            .and_then(|root| writer.write_field(&root, Sink::at("Sink"), decision.value));
        match result {
            Ok(()) => HandlerResult::Completed(Copied(decision.value)),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

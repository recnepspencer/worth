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
worth_query_operation_reads!(CopySelected => [Key, Sink, Value1, Value2, Value3, Value4, Value5, Value6, Value7, Value8, Value9, Value10, Value11, Value12]);
worth_query_operation_writes!(CopySelected => [Sink]);
fn scope(_: &Mode) -> String {
    "record".into()
}
worth_query_mutation_binding!(pub Binding for Mode, schema Schema,
    identity "class-read-cost.binding.v1", input Input, operation CopySelected, result ResultBinding,
    idempotency u64, identity "class-read-cost.command.v1", decision Decision, denial Refusal,
    handler identity "class-read-cost.handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Record, Identity, Key, String, ReadOnly, NoApplicationUnit, field Key::reference(), value scope,
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
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Sink::reference())
            .operation_write(Self::reference(), Sink::reference());
        macro_rules! grant {
            ($n:literal, $field:ident) => {
                if CLASSES >= $n {
                    schema = schema.operation_read_field(Self::reference(), $field::reference());
                }
            };
        }
        grant!(1, Value1);
        grant!(2, Value2);
        grant!(3, Value3);
        grant!(4, Value4);
        grant!(5, Value5);
        grant!(6, Value6);
        grant!(7, Value7);
        grant!(8, Value8);
        grant!(9, Value9);
        grant!(10, Value10);
        grant!(11, Value11);
        grant!(12, Value12);
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
            let root = reader.resolve_entity(Key::reference(), "record".to_owned())?;
            let value = match mode {
                Mode::Present => reader.field(&root, Value3::reference())?,
                Mode::Absent => reader.field(&root, Value1::reference())?,
                Mode::Discover => {
                    let mut found = None;
                    let mut count = 0;
                    macro_rules! probe {
                        ($n:literal, $field:ident) => {
                            if CLASSES >= $n {
                                if let Some(value) = reader.field(&root, $field::reference())? {
                                    found = Some(value);
                                    count += 1;
                                }
                            }
                        };
                    }
                    probe!(1, Value1);
                    probe!(2, Value2);
                    probe!(3, Value3);
                    probe!(4, Value4);
                    probe!(5, Value5);
                    probe!(6, Value6);
                    probe!(7, Value7);
                    probe!(8, Value8);
                    probe!(9, Value9);
                    probe!(10, Value10);
                    probe!(11, Value11);
                    probe!(12, Value12);
                    if count != 1 {
                        return Ok(Err(Denial::NotOneClass));
                    }
                    found
                }
            };
            let Some(value) = value else {
                return Ok(Err(Denial::Absent));
            };
            reader.field(&root, Sink::reference())?;
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
            .and_then(|root| writer.write_field(&root, Sink::reference(), decision.value));
        match result {
            Ok(()) => HandlerResult::Completed(Copied(decision.value)),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

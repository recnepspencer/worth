use super::*;
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Edit {
    pub mode: u64,
    pub samples: Vec<i64>,
    pub reference: Vec<u8>,
}
#[derive(Debug)]
pub struct Edited {
    pub before: Option<(ContentRefId, Vec<u8>)>,
    pub other: Option<ContentRefId>,
    pub version: Option<ContentRefId>,
    pub optional: Option<ContentRefId>,
}
#[derive(Clone, Debug)]
pub struct Denial;
worth_query_structured_value_binding!(pub InputBinding for Edit { identity: "content-door.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Edited { identity: "content-door.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "content-door.denial.v1" });
worth_query_operation!(pub Change for Schema, input InputBinding);
worth_query_operation_reads!(Change => [UnboundValues, Key, Values, OptionalValues, OtherValues, VersionedValues]);
worth_query_operation_writes!(Change => [UnboundValues, Key, Values, OptionalValues, OtherValues, VersionedValues]);
worth_query_operation_creates!(Change => [Record]);
worth_query_operation_removes_aspects!(Change => [Samples]);
worth_query_operation_links!(Change=>[ContainsSamples]);
fn key(_: &Edit) -> String {
    "record".into()
}
worth_query_mutation_binding!(pub Binding for Edit, schema Schema,
    identity "content-door.binding.v1", input InputBinding, operation Change, result ResultBinding,
    idempotency String, identity "content-door.idempotency.v1", decision Decision,
    denial DenialBinding, handler identity "content-door.handler.v1", program required, outputs NoApplicationMutationOutputs,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Record, Identity, Key, String, ReadOnly, NoApplicationUnit, field Key::reference(), value key,
);
pub struct Decision {
    target: WorthQueryInvariantMutationTarget<Schema, Record>,
    before: Option<(ContentRefId, Vec<u8>)>,
    other: Option<ContentRefId>,
    version: Option<ContentRefId>,
    optional: Option<ContentRefId>,
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .declared_operation::<Change>(
            Change::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<Binding>()
}
impl ApplicationDeclaredOperation<Schema> for Change {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Change::reference(), 32)
            .operation_read_field(Change::reference(), Key::reference())
            .operation_read_field(Change::reference(), Values::reference())
            .operation_read_field(Change::reference(), UnboundValues::reference())
            .operation_read_field(Change::reference(), OptionalValues::reference())
            .operation_write(Change::reference(), OptionalValues::reference())
            .operation_remove_aspect(Change::reference(), Samples::reference())
            .operation_read_field(Change::reference(), OtherValues::reference())
            .operation_read_field(Change::reference(), VersionedValues::reference())
            .operation_write(Change::reference(), OtherValues::reference())
            .operation_write(Change::reference(), VersionedValues::reference())
            .operation_write(Change::reference(), Key::reference())
            .operation_write(Change::reference(), Values::reference())
            .operation_write(Change::reference(), UnboundValues::reference())
            .operation_create(Change::reference(), Record::reference())
            .operation_link(Change::reference(), ContainsSamples::reference())
    }
}
pub struct Handler {
    pub cancellation: Option<WorthQueryCancellationSource>,
}
impl OperationHandler<Schema, Binding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &Edit,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            let root = reader.resolve_entity(
                Key::reference(),
                if input.mode == 19 {
                    String::from_utf8(input.reference.clone()).unwrap()
                } else if input.mode == 16 {
                    "created".into()
                } else {
                    "record".into()
                },
            )?;
            if input.mode == 15 {
                let mut stopped = false;
                for _ in 0..64 {
                    match reader.content(&root,samples()) {
                        Ok(Some(_))=>{},
                        Err(ContentReadDenial::Admission(WorthQueryValueDecodeDenial::ContentProjectionWorkExceeded))=>{stopped=true;break;},
                        other=>panic!("an exhausted content read cannot become absence or another value: {other:?}"),
                    }
                }
                assert!(
                    stopped,
                    "the declared projection bound must stop repeated content reads"
                );
            }
            let before = reader
                .content(&root, samples())
                .map_err(HandlerExecutionDenial::new)?
                .map(|content| (content.identity(), content.payload().to_vec()));
            let optional = reader
                .content(
                    &root,
                    content_field(OptionalValues::reference(), codec::GridFormat),
                )
                .map_err(HandlerExecutionDenial::new)?
                .map(|content| content.identity());
            let _unbound = reader.field(&root, UnboundValues::reference())?;
            let other = reader.field(&root, OtherValues::reference())?;
            let version = reader.field(&root, VersionedValues::reference())?;
            if input.mode == 6 {
                let _refused =
                    reader.content(&root, content_field(Values::reference(), codec::Impostor));
            }
            Ok(Decision {
                target: reader.mutation_target(&root)?,
                before,
                other,
                version,
                optional,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Edited, Denial> {
        let result = (|| {
            let target = writer
                .projected_entity(&decision.target)
                .map_err(HandlerExecutionDenial::new)?;
            // A small operation input reaches Native's own frame bound, rather
            // than exhausting unrelated structured-input serialization first.
            let generated;
            let values = if input.mode == 20 {
                let [count] = input.samples.as_slice() else {
                    panic!("exact count stimulus")
                };
                generated = vec![0; usize::try_from(*count).expect("nonnegative court count")];
                &generated
            } else {
                &input.samples
            };
            let grid = codec::Grid {
                rows: 1,
                columns: values.len().try_into().expect("bounded court dimensions"),
                samples: values.clone(),
                corrupt_result: input.mode == 8,
                swallow_sink_refusal: input.mode == 9,
                cancel_during_encoding: if input.mode == 14 {
                    self.cancellation.clone()
                } else {
                    None
                },
            };
            match input.mode {
                0 | 8 | 9 | 14 | 19 | 20 => writer
                    .write_content(&target, samples(), &grid)
                    .map_err(HandlerExecutionDenial::new)?,
                18 => {
                    let key = String::from_utf8(input.reference.clone()).unwrap();
                    let child = writer
                        .create_entity_in_context(
                            &target,
                            Record::reference(),
                            WorthQueryApplicationEntityKey::new(&key).unwrap(),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_field(&child, Key::reference(), key.clone())
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_content(&child, samples(), &grid)
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .link(ContainsSamples::reference(), key, &target, &child)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                1 => {
                    let new = writer
                        .create_entity(
                            Record::reference(),
                            WorthQueryApplicationEntityKey::new("created").unwrap(),
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_field(&new, Key::reference(), "created".to_owned())
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_content(&new, samples(), &grid)
                        .map_err(HandlerExecutionDenial::new)?;
                }
                2 | 17 => {
                    let identity = ContentRefId(
                        input
                            .reference
                            .as_slice()
                            .try_into()
                            .expect("court reference has exact width"),
                    );
                    if input.mode == 17 {
                        writer
                            .write_optional_field(
                                &target,
                                UnboundValues::reference(),
                                Some(identity),
                            )
                            .map_err(HandlerExecutionDenial::new)?;
                    } else {
                        writer
                            .write_field(&target, Values::reference(), identity)
                            .map_err(HandlerExecutionDenial::new)?;
                    }
                }
                3 => {
                    let _refused = writer.write_content(&target, samples(), &grid);
                }
                4 => writer
                    .write_content(
                        &target,
                        content_field(OtherValues::reference(), codec::OtherGrid),
                        &grid,
                    )
                    .map_err(HandlerExecutionDenial::new)?,
                5 => writer
                    .write_content(
                        &target,
                        content_field(VersionedValues::reference(), codec::GridV2),
                        &grid,
                    )
                    .map_err(HandlerExecutionDenial::new)?,
                7 => {
                    let _refused = writer.write_content(
                        &target,
                        content_field(Values::reference(), codec::Impostor),
                        &grid,
                    );
                }
                11 => writer
                    .remove_aspect(&target, Samples::reference())
                    .map_err(HandlerExecutionDenial::new)?,
                12 => writer
                    .write_content(
                        &target,
                        content_field(OptionalValues::reference(), codec::GridFormat),
                        &grid,
                    )
                    .map_err(HandlerExecutionDenial::new)?,
                13 => writer
                    .write_optional_field(&target, OptionalValues::reference(), None)
                    .map_err(HandlerExecutionDenial::new)?,
                _ => return Ok(()),
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Edited {
                before: decision.before,
                other: decision.other,
                version: decision.version,
                optional: decision.optional,
            }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

//! One actual birth implementation for producer and ordinary action bindings.
use super::*;
macro_rules! birth_handler {
    ($binding:ty, $input:ident, $reader:ident, $writer:ident, $decision:ident, $prepare:block, $apply:block) => {
impl<Schema: BirthSchema> OperationHandler<Schema, $binding> for Handler {
    fn decide(
        &self,
         $input: &Input,
         $reader: &mut DecisionReader<'_, '_, '_, Schema, $binding>,
    ) -> HandlerResult<Decision<Schema>, PlanarMutationDenial> {
        let source = match (|| -> Result<_, HandlerExecutionDenial> { $prepare })() {
            Ok(source) => source,
            Err(stop) => return HandlerResult::ExecutionDenied(stop),
        };
        HandlerResult::Completed(Decision {
            source,
            node: None,
            present: None,
            fieldless: None,
            strict: None,
        })
    }


    fn build_candidate(
        &self,
         $input: &Input,
         $decision: Decision<Schema>,
         $writer: &mut CandidateWriter<'_, Schema, $binding>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            $apply
            let key = |suffix| format!("{}:{suffix}", $input.scope_key);
            let node = $writer
                .create_entity(
                    OutputNode::reference(),
                    WorthQueryApplicationEntityKey::new(key("node"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            let present = $writer
                .create_entity(
                    PresentNode::reference(),
                    WorthQueryApplicationEntityKey::new(key("present"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            let empty = $writer
                .create_entity(
                    FieldlessNode::reference(),
                    WorthQueryApplicationEntityKey::new(key("fieldless"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            let strict = $writer
                .create_entity(
                    StrictNode::reference(),
                    WorthQueryApplicationEntityKey::new(key("strict"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            $writer
                .initialize_field(&present, Value::reference(), "native-present".to_owned())
                .map_err(HandlerExecutionDenial::new)?;
            if $input.incomplete {
                $writer
                    .initialize_field(
                        &strict,
                        Extra::reference(),
                        "present-but-incomplete".to_owned(),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            $writer
                .link(
                    Incidence::reference(),
                    "optional-native-incidence",
                    &node,
                    &empty,
                )
                .map_err(HandlerExecutionDenial::new)?;
            $writer
                .create_output::<BornNode<Schema>>(&node)
                .map_err(HandlerExecutionDenial::new)?;
            $writer
                .create_output::<BornPresent<Schema>>(&present)
                .map_err(HandlerExecutionDenial::new)?;
            $writer
                .create_output::<BornFieldless<Schema>>(&empty)
                .map_err(HandlerExecutionDenial::new)?;
            $writer
                .create_output::<BornStrict<Schema>>(&strict)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 4,
            }),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
}
    };
}
birth_handler!(Binding<Schema, false>, input, reader, writer, decision, {
    super::super::tracked_reads::birth(reader, &input.scope_key)?;
    super::super::own_effect::source_target(reader, &input.scope_key)
}, { super::super::own_effect::apply(writer, decision.source.as_ref())?; });
birth_handler!(
    OrdinaryBinding<Schema>,
    input,
    reader,
    writer,
    _decision,
    {
        let root = reader
            .resolve_entity(BodyKey::reference(), input.scope_key.clone())
            .map_err(HandlerExecutionDenial::new)?;
        let _key = reader.field(&root, BodyKey::reference())?;
        Ok(None)
    },
    {}
);

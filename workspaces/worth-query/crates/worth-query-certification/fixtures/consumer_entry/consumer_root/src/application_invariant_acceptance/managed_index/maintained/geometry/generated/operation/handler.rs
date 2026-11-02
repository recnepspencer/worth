//! Ordinary typed observations and output authoring supply actual manifest custody.
use super::*;
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Generate,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<Decision, Denial> {
        HandlerResult::from_result((|| {
            let a =
                reader.resolve_entity(DirectoryIdentity::reference(), input.directory.clone())?;
            let b = reader.resolve_entity(DirectoryIdentity::reference(), "B".to_owned())?;
            Ok(Decision {
                a: reader.mutation_target(&a)?,
                b: reader.mutation_target(&b)?,
            })
        })())
    }
    fn build_candidate(
        &self,
        _input: &Generate,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Generated, Denial> {
        let result = (|| {
            let a = writer
                .projected_entity(&decision.a)
                .map_err(HandlerExecutionDenial::new)?;
            let b = writer
                .projected_entity(&decision.b)
                .map_err(HandlerExecutionDenial::new)?;
            let start = writer
                .create_entity_in_context(
                    &a,
                    Vertex::reference(),
                    WorthQueryApplicationEntityKey::new("generated-start").unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            let end = writer
                .create_entity_in_context(
                    &a,
                    Vertex::reference(),
                    WorthQueryApplicationEntityKey::new("generated-end").unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            for (vertex, identity, x) in
                [(&start, "generated-start", 0), (&end, "generated-end", 10)]
            {
                writer
                    .initialize_field(vertex, VertexIdentity::reference(), identity.to_owned())
                    .map_err(HandlerExecutionDenial::new)?;
                writer
                    .initialize_field(
                        vertex,
                        VertexRegion::reference(),
                        super::super::super::super::bounds::Coordinates::new(x, 0, x, 0),
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            let edge = writer
                .create_entity_in_context(
                    &a,
                    Edge::reference(),
                    WorthQueryApplicationEntityKey::new("generated-edge").unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &edge,
                    EdgeIdentity::reference(),
                    "generated-edge".to_owned(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(ContainsEdge::reference(), "generated-geometry-A", &a, &edge)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(ContainsEdge::reference(), "generated-geometry-B", &b, &edge)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    Starts::reference(),
                    "generated-geometry-start",
                    &edge,
                    &start,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Ends::reference(), "generated-geometry-end", &edge, &end)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<EdgeOutput>(&edge)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<StartOutput>(&start)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<EndOutput>(&end)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(Generated)
        })();
        match result {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

use super::*;
use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryInvariantEntityIdentity, WorthQueryInvariantMutationTarget,
};

pub struct ReplacementDecision<Schema: TopologySchemaBinding> {
    pub(super) vertices: [WorthQueryInvariantMutationTarget<Schema, Body>; 3],
}

pub(super) fn observe_replacement<Schema: TopologySchemaBinding>(
    input: &VertexReplacement,
    reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
) -> HandlerResult<ReplacementDecision<Schema>, PlanarReplacementDenial> {
    let replacement = &input.replacement;
    let keys = [
        &input.scope_key,
        &replacement.retired_key,
        &replacement.next_key,
    ];
    if keys[0] == keys[1] || keys[1] == keys[2] || keys[0] == keys[2] {
        return HandlerResult::DomainDenied(PlanarReplacementDenial::CoincidentVertices);
    }
    if keys.contains(&&replacement.replacement.body_key) {
        return HandlerResult::DomainDenied(PlanarReplacementDenial::ReplacementKeyNotFresh);
    }
    let vertices = match resolve_vertices(keys, reader) {
        Ok(vertices) => vertices,
        Err(error) => return HandlerResult::ExecutionDenied(error),
    };
    if let Err(error) = reader
        .reader()
        .require_decision_entity(&vertices[1], Body::reference())
    {
        return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
    }
    for vertex in &vertices {
        for result in [
            reader.field(vertex, PositionX::reference()),
            reader.field(vertex, PositionY::reference()),
        ] {
            match result {
                Ok(Some(_)) => {}
                Ok(None) => {
                    return HandlerResult::DomainDenied(PlanarReplacementDenial::MissingCoordinate)
                }
                Err(error) => return HandlerResult::ExecutionDenied(error),
            }
        }
    }
    match observe_incident_edges(&vertices, reader) {
        HandlerResult::Completed(()) => {}
        HandlerResult::DomainDenied(denial) => return HandlerResult::DomainDenied(denial),
        HandlerResult::ExecutionDenied(denial) => return HandlerResult::ExecutionDenied(denial),
        HandlerResult::Cancelled => return HandlerResult::Cancelled,
        HandlerResult::DeadlineExceeded => return HandlerResult::DeadlineExceeded,
    }
    let targets = [
        reader.mutation_target(&vertices[0]),
        reader.mutation_target(&vertices[1]),
        reader.mutation_target(&vertices[2]),
    ];
    match targets {
        [Ok(anchor), Ok(retired), Ok(next)] => HandlerResult::Completed(ReplacementDecision {
            vertices: [anchor, retired, next],
        }),
        [Err(error), _, _] | [_, Err(error), _] | [_, _, Err(error)] => {
            HandlerResult::ExecutionDenied(error)
        }
    }
}

fn resolve_vertices<Schema: TopologySchemaBinding>(
    keys: [&String; 3],
    reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
) -> Result<[WorthQueryInvariantEntityIdentity<Schema, Body>; 3], HandlerExecutionDenial> {
    Ok([
        reader.resolve_entity(BodyKey::reference(), keys[0].clone())?,
        reader.resolve_entity(BodyKey::reference(), keys[1].clone())?,
        reader.resolve_entity(BodyKey::reference(), keys[2].clone())?,
    ])
}

fn observe_incident_edges<Schema: TopologySchemaBinding>(
    vertices: &[WorthQueryInvariantEntityIdentity<Schema, Body>; 3],
    reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
) -> HandlerResult<(), PlanarReplacementDenial> {
    for edge in vertices.windows(2) {
        let successor = match reader.related_one(PlanarSuccessor::reference(), &edge[0]) {
            Ok(successor) => successor,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if successor != edge[1] {
            return HandlerResult::DomainDenied(PlanarReplacementDenial::UnexpectedSuccessor);
        }
        if let Err(error) = reader.reader().require_decision_relation(
            PlanarSuccessor::reference(),
            &edge[0],
            &edge[1],
        ) {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
    }
    HandlerResult::Completed(())
}

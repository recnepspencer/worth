use super::*;
use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryInvariantEntityIdentity,
};

pub(super) fn observe_replacement<Schema: TopologySchemaBinding>(
    input: &VertexReplacement,
    reader: &mut DecisionReader<'_, '_, '_, Schema, VertexReplacementBinding<Schema>>,
) -> HandlerResult<(), PlanarReplacementDenial> {
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
    observe_incident_edges(&vertices, reader)
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
        let relations = match reader
            .reader()
            .decision_relations_from(PlanarSuccessor::reference(), &edge[0])
        {
            Ok(relations) => relations,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        // The installed successor contract requires exactly one outgoing edge.
        if relations.len() != 1 || relations[0].to() != &edge[1] {
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

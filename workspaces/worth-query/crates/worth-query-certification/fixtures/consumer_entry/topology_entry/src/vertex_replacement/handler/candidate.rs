use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationEffectEntity, WorthQueryApplicationEntityKey,
    WorthQueryApplicationOutputRole,
};

pub(super) fn replace_vertex<Schema: TopologySchemaBinding>(
    input: &VertexReplacement,
    writer: &mut CandidateWriter<'_, Schema, VertexReplacementBinding<Schema>>,
) -> Result<PlanarVertexReplacementResult, HandlerExecutionDenial> {
    let change = &input.replacement;
    let anchor = writer
        .resolve_entity(BodyKey::reference(), input.scope_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    let retired = writer
        .resolve_entity(BodyKey::reference(), change.retired_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    let next = writer
        .resolve_entity(BodyKey::reference(), change.next_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    let replacement = allocate_replacement(&change.replacement, writer)?;
    bind_correspondence(writer, &anchor, &replacement, &retired)?;
    writer
        .unlink(PlanarSuccessor::reference(), &anchor, &retired)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .unlink(PlanarSuccessor::reference(), &retired, &next)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .delete_entity(Body::reference(), &retired)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .link(
            PlanarSuccessor::reference(),
            format!("replacement:{}:incoming", change.replacement.body_key),
            &anchor,
            &replacement,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .link(
            PlanarSuccessor::reference(),
            format!("replacement:{}:outgoing", change.replacement.body_key),
            &replacement,
            &next,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(PlanarVertexReplacementResult {
        replacement_key: change.replacement.body_key.clone(),
    })
}

fn allocate_replacement<Schema: TopologySchemaBinding>(
    vertex: &worth_query_consumer_values::PlanarVertex,
    writer: &mut CandidateWriter<'_, Schema, VertexReplacementBinding<Schema>>,
) -> Result<WorthQueryApplicationEffectEntity<Schema, Body>, HandlerExecutionDenial> {
    let key = WorthQueryApplicationEntityKey::new(&vertex.body_key)
        .map_err(HandlerExecutionDenial::new)?;
    let replacement = writer
        .candidate()
        .create_entity(Body::reference(), key)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .initialize_field(&replacement, BodyKey::reference(), vertex.body_key.clone())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .initialize_field(&replacement, PositionX::reference(), vertex.x)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .initialize_field(&replacement, PositionY::reference(), vertex.y)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .candidate()
        .initialize_field(
            &replacement,
            Length::reference(),
            PositiveLength::new(1).unwrap(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(replacement)
}

fn bind_correspondence<Schema: TopologySchemaBinding>(
    writer: &mut CandidateWriter<'_, Schema, VertexReplacementBinding<Schema>>,
    anchor: &WorthQueryApplicationEffectEntity<Schema, Body>,
    replacement: &WorthQueryApplicationEffectEntity<Schema, Body>,
    retired: &WorthQueryApplicationEffectEntity<Schema, Body>,
) -> Result<(), HandlerExecutionDenial> {
    writer
        .preserve_output(WorthQueryApplicationOutputRole::new("anchor"), anchor)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .create_output(
            WorthQueryApplicationOutputRole::new("replacement"),
            replacement,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .retire_output(WorthQueryApplicationOutputRole::new("retired"), retired)
        .map_err(HandlerExecutionDenial::new)?;
    Ok(())
}

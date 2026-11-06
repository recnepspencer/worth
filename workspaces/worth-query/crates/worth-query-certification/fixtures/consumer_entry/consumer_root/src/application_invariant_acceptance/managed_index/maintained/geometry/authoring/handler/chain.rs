//! Compact input authors every real endpoint, edge and membership through verbs.
use super::super::super::super::bounds::Coordinates;
use super::*;

pub(super) fn apply(
    edit: &Edit,
    directory: Target<Directory>,
    shared_directory: Option<Target<Directory>>,
    writer: &mut CandidateWriter<'_, Catalog, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    let Edit::Chain {
        directory: key,
        shared_directory: shared_key,
        prefix,
        vertices,
        origin,
    } = edit
    else {
        unreachable!("only the installed Chain decision reaches chain authoring")
    };
    let directory = writer
        .projected_entity(&directory)
        .map_err(HandlerExecutionDenial::new)?;
    let shared = shared_directory
        .as_ref()
        .map(|target| writer.projected_entity(target))
        .transpose()
        .map_err(HandlerExecutionDenial::new)?;
    let mut previous = None;
    for index in 0..*vertices {
        let identity = format!("{prefix}-v-{index}");
        let vertex = writer
            .create_entity_in_context(
                &directory,
                Vertex::reference(),
                WorthQueryApplicationEntityKey::new(format!("geometry-vertex-{identity}"))
                    .map_err(HandlerExecutionDenial::new)?,
            )
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .initialize_field(&vertex, VertexIdentity::at("VertexIdentity"), identity)
            .map_err(HandlerExecutionDenial::new)?;
        // u32 counts times two fit i64; checked origin keeps authoring exact.
        let x = origin.checked_add(i64::from(index) * 2).ok_or_else(|| {
            HandlerExecutionDenial::new(
                worth_query_consumer_values::BoxIndexDenial::InvalidTopology,
            )
        })?;
        writer
            .initialize_field(
                &vertex,
                VertexRegion::at("VertexRegion"),
                Coordinates::new(x, 0, x, 0),
            )
            .map_err(HandlerExecutionDenial::new)?;
        if let Some(start) = previous.as_ref() {
            let identity = format!("{prefix}-e-{}", index - 1);
            let edge = writer
                .create_entity_in_context(
                    &directory,
                    Edge::reference(),
                    WorthQueryApplicationEntityKey::new(format!("geometry-edge-{identity}"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&edge, EdgeIdentity::at("EdgeIdentity"), identity.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ContainsEdge::reference(),
                    format!("{key}-{identity}"),
                    &directory,
                    &edge,
                )
                .map_err(HandlerExecutionDenial::new)?;
            if let Some(shared) = shared.as_ref() {
                writer
                    .link(
                        ContainsEdge::reference(),
                        format!("{}-{identity}", shared_key.as_ref().unwrap()),
                        shared,
                        &edge,
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            writer
                .link(
                    Starts::reference(),
                    format!("{identity}-start"),
                    &edge,
                    start,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Ends::reference(), format!("{identity}-end"), &edge, &vertex)
                .map_err(HandlerExecutionDenial::new)?;
        }
        previous = Some(vertex);
    }
    Ok(())
}

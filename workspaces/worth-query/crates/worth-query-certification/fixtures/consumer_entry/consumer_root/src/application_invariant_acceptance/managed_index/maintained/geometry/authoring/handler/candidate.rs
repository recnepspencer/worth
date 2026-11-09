//! Ordinary candidate verbs preserve the prepared target and relation observations.
use super::super::super::super::bounds::Coordinates;
use super::*;
pub(super) fn apply(
    edit: &Edit,
    decision: Decision,
    writer: &mut CandidateWriter<'_, Catalog, Binding>,
) -> Result<(), HandlerExecutionDenial> {
    match (edit, decision) {
        (
            Edit::Chain { .. },
            Decision::Chain {
                directory,
                shared_directory,
            },
        ) => {
            super::chain::apply(edit, directory, shared_directory, writer)?;
        }
        (Edit::Vertex { identity, x, y, .. }, Decision::Vertex(context)) => {
            let context = writer
                .projected_entity(&context)
                .map_err(HandlerExecutionDenial::new)?;
            let entity = writer
                .create_entity_in_context(
                    &context,
                    Vertex::reference(),
                    WorthQueryApplicationEntityKey::new(format!("geometry-vertex-{identity}"))
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &entity,
                    VertexIdentity::at("VertexIdentity"),
                    identity.clone(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &entity,
                    VertexRegion::at("VertexRegion"),
                    Coordinates::new(*x, *y, *x, *y),
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        (
            Edit::Edge {
                directory: key,
                identity,
                ..
            },
            Decision::Edge {
                directory,
                start,
                end,
            },
        ) => {
            let directory = writer
                .projected_entity(&directory)
                .map_err(HandlerExecutionDenial::new)?;
            let start = writer
                .projected_entity(&start)
                .map_err(HandlerExecutionDenial::new)?;
            let end = writer
                .projected_entity(&end)
                .map_err(HandlerExecutionDenial::new)?;
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
            writer
                .link(
                    Starts::reference(),
                    format!("{identity}-start"),
                    &edge,
                    &start,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Ends::reference(), format!("{identity}-end"), &edge, &end)
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Move { x, y, .. }, Decision::Move(vertex)) => {
            let vertex = writer
                .projected_entity(&vertex)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(
                    &vertex,
                    VertexRegion::at("VertexRegion"),
                    Coordinates::new(*x, *y, *x, *y),
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        (
            Edit::Share {
                directory: key,
                edge: identity,
            },
            Decision::Share { directory, edge },
        ) => {
            let directory = writer
                .projected_entity(&directory)
                .map_err(HandlerExecutionDenial::new)?;
            let edge = writer
                .projected_entity(&edge)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ContainsEdge::reference(),
                    format!("{key}-{identity}"),
                    &directory,
                    &edge,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::Unshare { .. }, Decision::Unshare { directory, edge }) => {
            let directory = writer
                .projected_entity(&directory)
                .map_err(HandlerExecutionDenial::new)?;
            let edge = writer
                .projected_entity(&edge)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(ContainsEdge::reference(), &directory, &edge)
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::RewireEnd { edge: key, .. }, Decision::Rewire { edge, old, new }) => {
            let edge = writer
                .projected_entity(&edge)
                .map_err(HandlerExecutionDenial::new)?;
            let old = writer
                .projected_entity(&old)
                .map_err(HandlerExecutionDenial::new)?;
            let new = writer
                .projected_entity(&new)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .unlink(Ends::reference(), &edge, &old)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(Ends::reference(), format!("{key}-rewired-end"), &edge, &new)
                .map_err(HandlerExecutionDenial::new)?;
        }
        (Edit::DuplicateEnd { edge: key, .. }, Decision::Duplicate { edge, end }) => {
            let edge = writer
                .projected_entity(&edge)
                .map_err(HandlerExecutionDenial::new)?;
            let end = writer
                .projected_entity(&end)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    Ends::reference(),
                    format!("{key}-duplicate-end"),
                    &edge,
                    &end,
                )
                .map_err(HandlerExecutionDenial::new)?;
        }
        _ => unreachable!("the prepared decision belongs to this unchanged geometry input"),
    }
    Ok(())
}

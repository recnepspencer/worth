//! Committed Parent reads and individually observed Native targets share one admission.
use super::super::{provider, query, rule};
use super::*;
use std::sync::Arc;
type Reader<'borrow, 'reader, 'runtime> =
    DecisionReader<'borrow, 'reader, 'runtime, Catalog, Binding>;
mod candidate;
mod chain;
pub struct Handler(pub Arc<provider::Calls>);

impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        edit: &Edit,
        reader: &mut Reader<'_, '_, '_>,
    ) -> HandlerResult<Decision, Denial> {
        let result = (|| {
            if matches!(
                edit,
                Edit::Move { .. } | Edit::RewireEnd { .. } | Edit::DuplicateEnd { .. }
            ) {
                let rows = reader.index_query::<query::BoxBinding>(query::FindBoxes {
                    directory: edit.directory().to_owned(),
                    probe: rule::probe(),
                })?;
                // The values are real committed reads; this observation grants
                // no target or proposed-state mutation authority.
                *self.0.last_decision.lock().unwrap() = Some(rows);
            }
            match edit {
                Edit::Chain {
                    directory,
                    shared_directory,
                    ..
                } => Ok(Decision::Chain {
                    directory: directory_target(reader, directory)?,
                    shared_directory: shared_directory
                        .as_deref()
                        .map(|key| directory_target(reader, key))
                        .transpose()?,
                }),
                Edit::Vertex { directory, .. } => {
                    Ok(Decision::Vertex(directory_target(reader, directory)?))
                }
                Edit::Edge {
                    directory,
                    start,
                    end,
                    ..
                } => Ok(Decision::Edge {
                    directory: directory_target(reader, directory)?,
                    start: vertex_target(reader, start)?,
                    end: vertex_target(reader, end)?,
                }),
                Edit::Move { vertex, .. } => Ok(Decision::Move(vertex_target(reader, vertex)?)),
                Edit::Share { directory, edge } => {
                    let entity = edge_identity(reader, edge)?;
                    Ok(Decision::Share {
                        directory: directory_target(reader, directory)?,
                        edge: reader.mutation_target(&entity)?,
                    })
                }
                Edit::Unshare { directory, edge } => {
                    let entity = edge_identity(reader, edge)?;
                    reader
                        .relations_to(ContainsEdge::reference(), &entity)
                        .map_err(HandlerExecutionDenial::new)?;
                    Ok(Decision::Unshare {
                        directory: directory_target(reader, directory)?,
                        edge: reader.mutation_target(&entity)?,
                    })
                }
                Edit::RewireEnd { edge, end, .. } => {
                    let (edge, old) = end_targets(reader, edge)?;
                    Ok(Decision::Rewire {
                        edge,
                        old,
                        new: vertex_target(reader, end)?,
                    })
                }
                Edit::DuplicateEnd { edge, .. } => {
                    let (edge, end) = end_targets(reader, edge)?;
                    Ok(Decision::Duplicate { edge, end })
                }
            }
        })();
        match result {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        edit: &Edit,
        decision: Decision,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        match candidate::apply(edit, decision, writer) {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
fn directory_target(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<Target<Directory>, HandlerExecutionDenial> {
    let entity =
        reader.resolve_entity(DirectoryIdentity::at("DirectoryIdentity"), key.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Directory::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader.mutation_target(&entity)
}
fn vertex_target(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<Target<Vertex>, HandlerExecutionDenial> {
    let entity = reader.resolve_entity(VertexIdentity::at("VertexIdentity"), key.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Vertex::reference())
        .map_err(HandlerExecutionDenial::new)?;
    reader.field(&entity, VertexRegion::at("VertexRegion"))?;
    reader.mutation_target(&entity)
}
fn edge_identity(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<WorthQueryInvariantEntityIdentity<Catalog, Edge>, HandlerExecutionDenial> {
    let entity = reader.resolve_entity(EdgeIdentity::at("EdgeIdentity"), key.to_owned())?;
    reader
        .reader()
        .require_decision_entity(&entity, Edge::reference())
        .map_err(HandlerExecutionDenial::new)?;
    Ok(entity)
}
fn end_targets(
    reader: &mut Reader<'_, '_, '_>,
    key: &str,
) -> Result<(Target<Edge>, Target<Vertex>), HandlerExecutionDenial> {
    let entity = edge_identity(reader, key)?;
    let mut ends = reader
        .relations_from(Ends::reference(), &entity)
        .map_err(HandlerExecutionDenial::new)?;
    if ends.len() != 1 {
        return Err(HandlerExecutionDenial::new(
            worth_query_consumer_values::BoxIndexDenial::InvalidTopology,
        ));
    }
    let end = ends.pop().expect("one actual observed endpoint").into_to();
    Ok((
        reader.mutation_target(&entity)?,
        reader.mutation_target(&end)?,
    ))
}

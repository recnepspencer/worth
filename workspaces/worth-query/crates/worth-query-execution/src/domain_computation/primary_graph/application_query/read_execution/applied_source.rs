//! Exactly one producer source from a real applied candidate and before-image
//! read permission. This phase never creates a Current observation or receipt.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::{
        observed_source::WorthQueryObservedSourceFootprint, WorthQueryApplicationProjectionDenial,
    },
    value_decode::ValueDecodeState,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

pub(in crate::domain_computation::primary_graph::application_query) struct AppliedSourceProjection<
    Value,
> {
    pub(in crate::domain_computation::primary_graph::application_query) identity:
        RelationalAppliedCandidateReadIdentity,
    pub(in crate::domain_computation::primary_graph::application_query) value: Value,
    pub(in crate::domain_computation::primary_graph::application_query) footprint:
        WorthQueryObservedSourceFootprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum AppliedSourceReadStop {
    Read(WorthQueryApplicationReadExecutionDenial),
    Projection(WorthQueryApplicationProjectionDenial),
    Native(worth_relational::facade::transactions::RelationalExecutionDenialCause),
}

pub(in crate::domain_computation::primary_graph::application_query) fn read_applied_source<
    Schema,
    Query,
    Value,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    permission: &read_plan::ReadPlan<'_>,
    interruption: ReadInterruption<'_>,
) -> Result<AppliedSourceProjection<Value>, AppliedSourceReadStop>
where
    Schema: ApplicationSchema,
    Value: super::super::WorthQueryApplicationProjection<Schema, Query>,
{
    use AppliedSourceReadStop::{Projection, Read};
    let selection =
        root_selection::select_applied_source_root(pin, execution, graph, permission, interruption)
            .map_err(Read)?;
    validate_read_cardinality(
        ApplicationQueryCardinality::ExactlyOne,
        selection.candidates.len(),
        permission,
    )
    .map_err(Read)?;
    let mut tree = tree_materialization::materialize_applied_source_tree(
        execution,
        pin,
        graph,
        permission.contract,
        permission.governance,
        permission.parameters,
        &selection.candidates,
        selection.selected_predicate_source.as_ref(),
        selection.root_path_source.as_ref(),
        interruption,
        None,
        permission.material_source,
    )
    .map_err(Read)?;
    let source_selection = selection.result_set_source.as_ref().ok_or_else(|| {
        Read(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            permission.name,
        ))
    })?;
    complete_single_source_selection(&mut tree.source_footprints, source_selection);
    let mut values =
        kernel_outcome::decode_projection_tree::<Schema, Query, Value, AppliedSourceReadStop>(
            execution,
            decode,
            &tree.rows,
            permission.governance,
            || interruption.checkpoint(permission.name).map_err(Read),
            Projection,
        )?;
    if values.len() != 1 || tree.source_footprints.len() != 1 {
        return Err(Read(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::CardinalityMismatch,
            permission.name,
        )));
    }
    Ok(AppliedSourceProjection {
        identity: pin.candidate_identity().clone(),
        value: values.pop().expect("one decoded source"),
        footprint: tree
            .source_footprints
            .into_iter()
            .next()
            .expect("one actual source footprint"),
    })
}

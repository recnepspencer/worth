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

pub struct AppliedSourceProjection<Value> {
    pub identity: RelationalAppliedCandidateReadIdentity,
    pub value: Value,
    pub footprint: WorthQueryObservedSourceFootprint,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppliedSourceReadStop {
    Read(WorthQueryApplicationReadExecutionDenial),
    Projection(WorthQueryApplicationProjectionDenial),
    Native(worth_relational::facade::transactions::RelationalExecutionDenialCause),
}

pub fn read_applied_source<Schema, Query, Value>(
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
    use AppliedSourceReadStop::Read;
    let (selection, mut tree) = read_applied_tree(
        execution,
        pin,
        graph,
        permission,
        ApplicationQueryCardinality::ExactlyOne,
        interruption,
    )?;
    let source_selection = selection.result_set_source.as_ref().ok_or_else(|| {
        Read(read_execution_denial(
            WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
            permission.name,
        ))
    })?;
    complete_single_source_selection(&mut tree.source_footprints, source_selection);
    let mut values = decode_applied_rows::<Schema, Query, Value>(
        execution,
        decode,
        &tree.rows,
        permission,
        interruption,
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

/// Both source and decision projections consume the same applied root selection
/// and Native result tree. The entry supplies its installed cardinality.
pub fn read_applied_tree(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    permission: &read_plan::ReadPlan<'_>,
    cardinality: ApplicationQueryCardinality,
    interruption: ReadInterruption<'_>,
) -> Result<
    (
        root_selection::BoundedRootSelection,
        tree_materialization::MaterializedApplicationResultTree,
    ),
    AppliedSourceReadStop,
> {
    use AppliedSourceReadStop::Read;
    let selection =
        root_selection::select_applied_source_root(pin, execution, graph, permission, interruption)
            .map_err(Read)?;
    validate_read_cardinality(cardinality, selection.candidates.len(), permission).map_err(Read)?;
    let tree = tree_materialization::materialize_applied_source_tree(
        execution,
        pin,
        graph,
        permission.contract,
        permission.governance,
        permission.parameters,
        &selection.candidates,
        &selection.selected_predicate_sources,
        selection.root_path_source.as_ref(),
        interruption,
        None,
        permission.material_source,
    )
    .map_err(Read)?;
    Ok((selection, tree))
}

pub fn decode_applied_rows<Schema, Query, Value>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    tree: &super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    permission: &read_plan::ReadPlan<'_>,
    interruption: ReadInterruption<'_>,
) -> Result<Vec<Value>, AppliedSourceReadStop>
where
    Schema: ApplicationSchema,
    Value: super::super::WorthQueryApplicationProjection<Schema, Query>,
{
    kernel_outcome::decode_projection_tree::<Schema, Query, Value, AppliedSourceReadStop>(
        execution,
        decode,
        tree,
        permission.governance,
        // An applied candidate has no published lineage to read names at.
        None,
        || {
            interruption
                .checkpoint(permission.name)
                .map_err(AppliedSourceReadStop::Read)
        },
        AppliedSourceReadStop::Projection,
    )
}

impl AppliedSourceReadStop {
    pub fn into_source_query_stop(self, query: &str) -> super::super::SourceQueryObservationStop {
        use super::super::{
            authorized_read::WorthQueryAuthorizedApplicationReadDenial as Read, one_shot,
        };
        let denial = match self {
            Self::Read(cause) => one_shot::map_authorized_read_denial(Read::Read(cause), query),
            Self::Native(cause) => one_shot::map_authorized_read_denial(
                Read::Read(super::relational_read_denial(cause, query)),
                query,
            ),
            Self::Projection(cause) => one_shot::denial(
                one_shot::WorthQueryApplicationOneShotDenialKind::Projection(cause.kind()),
                query,
                format!("{cause:?}"),
            ),
        };
        super::super::SourceQueryObservationStop::Read(
            super::super::WorthQueryAdmittedOneShotStop::Execution(denial),
        )
    }
}

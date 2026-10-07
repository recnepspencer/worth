//! Refused increments follow the map and ordinary Query-read laws independently.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use std::{num::NonZeroUsize, time::Duration};
use worth_execution::{
    ExecutionWorkCeiling, MapOutcome, MapStop, SerialMemoryBudget, SerialRequest,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

fn current_controls(
    scope: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryProductQueryControls<'_> {
    WorthQueryProductQueryControls::new(
        NonZeroUsize::new(10).unwrap(),
        NonZeroUsize::new(10000).unwrap(),
        scope,
    )
}

mod map_ceiling;
mod query_ceiling;

mod parallel_ceiling;

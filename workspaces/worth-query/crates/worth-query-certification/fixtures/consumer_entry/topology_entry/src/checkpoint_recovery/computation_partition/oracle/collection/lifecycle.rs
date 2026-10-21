//! A single installed checked owner across real Create and Preserve producers.
use super::super::*;
use crate::checkpoint_recovery::computation_partition::region_output::{
    RegionOutputInput, RegionOutputInputBinding,
};
use std::marker::PhantomData;
use worth_query_decl::facade::application_operation::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryDecisionPlan,
};

mod binding;
mod custody;
mod handler;
mod inventory;
mod producer;
mod readiness;
use binding::{InitialBinding, PreserveBinding};
use producer::{InitialProducer, PreserveProducer, Provider};
use readiness::Readiness;

pub(in crate::checkpoint_recovery::computation_partition) fn declare<
    Schema: TopologySchemaBinding,
>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .application_mutation_binding::<InitialBinding<Schema>>()
        .application_mutation_binding::<PreserveBinding<Schema>>()
}

/// Unselected declarations have refusing handlers, not fabricated publications.
pub(in crate::checkpoint_recovery::computation_partition) fn idle<Schema: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<InitialBinding<Schema>, _>(handler::InitialHandler::<Schema>::idle())?;
    setup.handler::<PreserveBinding<Schema>, _>(handler::PreserveHandler::<Schema>::idle())
}

pub(in crate::checkpoint_recovery::computation_partition::oracle) fn contracts<const MODE: u8>(
    contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<InitialProducer>()?;
    contracts.producer::<PreserveProducer>()?;
    if MODE == super::ORPHAN_OPERATION {
        contracts.conditional::<Readiness<true, false, 1>>()?;
        contracts.conditional::<Readiness<false, false, 1>>()?;
        return Ok(());
    }
    if MODE == super::ORPHAN_STAGE {
        contracts.conditional::<Readiness<true, false, 2>>()?;
        contracts.conditional::<Readiness<false, false, 2>>()?;
        return Ok(());
    }
    contracts.conditional::<Readiness<true>>()?;
    if MODE == super::CONFLICTING_READINESS {
        contracts.conditional::<Readiness<false, true>>()?;
    } else {
        contracts.conditional::<Readiness<false>>()?;
    }
    Ok(())
}

pub(in crate::checkpoint_recovery::computation_partition::oracle) fn configure<
    const WORK: usize,
    const MODE: u8,
>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    let installed = setup
        .derived_collection::<PlanarFinalOutputFeature, super::RegionCollection<WORK>, _>(
            OracleOwner::<{ super::LIFECYCLE }>,
        )?;
    let initial = installed.clone();
    setup.handler::<InitialBinding<CheckpointSchema>, _>(handler::InitialHandler::running(
        move |reader, set| {
            decision_plan::begin();
            decision_plan::prepared(initial.prepare(reader, set), |_| {})
        },
    ))?;
    setup.handler::<PreserveBinding<CheckpointSchema>, _>(handler::PreserveHandler::running(
        move |reader, set| {
            decision_plan::begin();
            decision_plan::prepared(installed.prepare(reader, set), |_| {})
        },
    ))?;
    setup.producer::<InitialProducer>(Provider)?;
    setup.producer::<PreserveProducer>(Provider)?;
    if MODE == super::ORPHAN_OPERATION {
        setup.conditional::<Readiness<true, false, 1>>(())?;
        return setup.conditional::<Readiness<false, false, 1>>(());
    }
    if MODE == super::ORPHAN_STAGE {
        setup.conditional::<Readiness<true, false, 2>>(())?;
        return setup.conditional::<Readiness<false, false, 2>>(());
    }
    setup.conditional::<Readiness<true>>(())?;
    if MODE == super::CONFLICTING_READINESS {
        setup.conditional::<Readiness<false, true>>(())
    } else {
        setup.conditional::<Readiness<false>>(())
    }
}

pub(super) fn output_key(scope: &str) -> String {
    format!("collection:{scope}")
}

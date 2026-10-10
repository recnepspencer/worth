use std::num::NonZeroUsize;

use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
        WorthQueryApplicationRequestQueryDenial, WorthQueryOutputDemandControls,
    },
    primary_graph::{
        WorthQueryGeneratedOutputReconstructionDenial, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrincipalResolutionDenialKind,
    },
};
use worth_query_topology_entry::{
    AlternatePlanarOutputProducer, PlanarFinalOutputFamily, PlanarFinalOutputProducer,
    PlanarOutputToFinalConnection, PlanarRead, PlanarSourceAdjustment, PlanarSuccessor, PositionY,
};

use super::{length, output_correspondence::observed_source, ProgramApplication, Request};
use crate::ConsumerSchema;

mod expected;
mod manifest;
mod retained;
mod successor;

use expected::{read, vertices};
use manifest::{claim_entity, require_optional_role_reads, write_fields};

#[path = "generated_materialization/typed_reconstruction_preserves_query_authority.rs"]
mod typed_reconstruction_preserves_query_authority;
pub(super) use typed_reconstruction_preserves_query_authority::typed_reconstruction_preserves_query_authority;

fn foreign_runtime_rejection(
    foreign: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
) -> worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput {
    match foreign
        .with_host_advancement(|host_phase| {
            foreign.reconstruct_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(
                &host_phase,
                suspended,
            )
        })
        .expect("installed host call opens")
    {
        Ok(_) => panic!("foreign runtime authority must not reconstruct Query custody"),
        Err(failure) => {
            assert_eq!(
                failure.denial(),
                WorthQueryGeneratedOutputReconstructionDenial::ForeignRuntime
            );
            failure.into_suspended()
        }
    }
}

fn wrong_producer_rejection(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
) -> worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput {
    match application
        .with_host_advancement(|host_phase| {
            application
                .reconstruct_generated_output::<AlternatePlanarOutputProducer<ConsumerSchema>>(
                    &host_phase,
                    suspended,
                )
        })
        .expect("installed host call opens")
    {
        Ok(_) => panic!("another installed producer must not inherit reconstruction custody"),
        Err(failure) => {
            assert_eq!(
                failure.denial(),
                WorthQueryGeneratedOutputReconstructionDenial::ForeignProducer
            );
            failure.into_suspended()
        }
    }
}

fn incomplete_manifest_rejection(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
    vertices: &[PlanarVertex],
) -> worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput {
    let mut reconstruction = match application
        .with_host_advancement(|host_phase| {
            application.reconstruct_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(
                &host_phase,
                suspended,
            )
        })
        .expect("installed host call opens")
    {
        Ok(reconstruction) => reconstruction,
        Err(failure) => panic!(
            "the owning producer starts reconstruction: {:?}",
            failure.denial()
        ),
    };
    let first = claim_entity(&mut reconstruction, &vertices[0], 0);
    write_fields(&mut reconstruction, &first, &vertices[0]);
    let failure = match reconstruction.finish() {
        Ok(_) => panic!("an incomplete custody manifest must not finish"),
        Err(failure) => failure,
    };
    assert_eq!(
        failure.denial(),
        WorthQueryGeneratedOutputReconstructionDenial::IncompleteManifest
    );
    failure.into_suspended()
}

fn claim_denials_preserve_session(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
    vertices: &[PlanarVertex],
) -> worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput {
    let mut reconstruction = application
        .with_host_advancement(|host_phase| {
            application
                .reconstruct_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(
                    &host_phase,
                    suspended,
                )
                .unwrap_or_else(|_| panic!("the owning producer restarts reconstruction"))
        })
        .expect("installed host call opens");
    let entities = vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let entity = claim_entity(&mut reconstruction, vertex, index);
            write_fields(&mut reconstruction, &entity, vertex);
            entity
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reconstruction
            .field(&entities[0], PositionY::reference(), length(9_999))
            .expect_err("a denied duplicate field must not replace the admitted value"),
        WorthQueryGeneratedOutputReconstructionDenial::DuplicateField
    );
    let duplicate = match reconstruction
        .entity::<worth_query_topology_entry::FinalAnchorOutput<ConsumerSchema>>()
    {
        Ok(_) => panic!("one output identity may be claimed only once"),
        Err(denial) => denial,
    };
    assert_eq!(
        duplicate,
        WorthQueryGeneratedOutputReconstructionDenial::DuplicateEntityClaim
    );
    assert_eq!(
        reconstruction
            .relation(PlanarSuccessor::reference(), &entities[0], &entities[2])
            .expect_err("a reversed or invented endpoint pair must be rejected"),
        WorthQueryGeneratedOutputReconstructionDenial::WrongRelationEndpoint
    );
    reconstruction
        .relation(PlanarSuccessor::reference(), &entities[0], &entities[1])
        .expect("the exact relation can be claimed once");
    assert_eq!(
        reconstruction
            .relation(PlanarSuccessor::reference(), &entities[0], &entities[1])
            .expect_err("one retained relation identity may be claimed only once"),
        WorthQueryGeneratedOutputReconstructionDenial::DuplicateRelationClaim
    );
    reconstruction.abort()
}

fn complete(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
    vertices: &[PlanarVertex],
) -> worth_query_host::facade::primary_graph::WorthQueryCompletedGeneratedOutputReconstruction<
    ConsumerSchema,
    PlanarFinalOutputProducer<ConsumerSchema>,
> {
    let mut reconstruction = application
        .with_host_advancement(|host_phase| {
            application
                .reconstruct_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(
                    &host_phase,
                    suspended,
                )
                .unwrap_or_else(|_| panic!("the owning producer completes reconstruction"))
        })
        .expect("installed host call opens");
    require_optional_role_reads(&mut reconstruction);
    let entities = vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let entity = claim_entity(&mut reconstruction, vertex, index);
            write_fields(&mut reconstruction, &entity, vertex);
            entity
        })
        .collect::<Vec<_>>();
    for index in 0..entities.len() {
        reconstruction
            .relation(
                PlanarSuccessor::reference(),
                &entities[index],
                &entities[(index + 1) % entities.len()],
            )
            .expect("the exact typed relation endpoint pair is retained by custody");
    }
    reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("every suspended record was reconstructed exactly once"))
}

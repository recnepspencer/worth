use worth_query_consumer_values::{PlanarOperation, PlanarVertex};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestQueryDenial,
    },
    primary_graph::{
        WorthQueryApplicationOutputRole, WorthQueryGeneratedOutputReconstructionDenial,
        WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionDenialKind,
        WorthQueryCreateOutput,
    },
};
use worth_query_topology_entry::{
    AlternatePlanarOutputProducer, Body, BodyKey, InitialPlanarProducer, Length, PlanarMutation,
    PlanarMutationBinding, PlanarRead, PlanarSuccessor, PositionX, PositionY,
};

use super::{length, mutate, Request};
use crate::ConsumerSchema;

pub(super) fn typed_reconstruction_preserves_query_authority(
    application: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    foreign: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    request: &Request<'_>,
    scope: &WorthQueryRequestScope,
) {
    let vertices = vertices();
    let outcome = mutate(
        request,
        PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::CreateCycle(vertices.clone()),
            validator_work: 4_096,
        },
        980,
    );
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("the reconstruction source cycle must publish: {outcome:?}")
    };
    let branch = receipt.product_branch();
    let retained_result = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the producer source query is admitted");
    let source = retained_result.observed_sources()[0].clone();
    let wrong_source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the unrelated producer source query is admitted")
        .observed_sources()[0]
        .clone();
    let retained = read(request, &vertices[0].body_key);
    match application
        .on_branch(branch)
        .select()
        .expect("the Query-issued output occurrence remains selectable")
        .suspend_current_generated_output::<InitialPlanarProducer<ConsumerSchema>>(
            scope,
            wrong_source,
        )
    {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure::Qualification(
            worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
        )) => {}
        _ => panic!("an unrelated observed source must not suspend this output"),
    }
    application.fail_next_durable_append_for_test();
    let recovery = match application
        .on_branch(branch)
        .select()
        .expect("the Query-issued output occurrence remains selectable")
        .suspend_current_generated_output::<InitialPlanarProducer<ConsumerSchema>>(scope, source)
    {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(recovery)) => recovery,
        _ => panic!("the injected owner failure must retain opaque suspension recovery"),
    };
    assert_eq!(recovery.product_branch(), branch);
    assert_eq!(read(request, &vertices[0].body_key), retained);
    let recovery = match foreign
        .continue_generated_output_suspension_recovery::<InitialPlanarProducer<ConsumerSchema>>(
            recovery, scope,
        )
    {
        Ok(_) => panic!("a foreign Query runtime must not settle suspension custody"),
        Err(failure) => {
            assert_eq!(
                failure.stage(),
                worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionRecoveryStage::StaleOccurrence,
            );
            failure.into_recovery()
        }
    };
    let suspended = application
        .continue_generated_output_suspension_recovery::<InitialPlanarProducer<ConsumerSchema>>(
            recovery, scope,
        )
        .unwrap_or_else(|failure| {
            panic!(
                "Query must adopt the settled suspension into World: {:?}",
                failure.stage()
            )
        });

    let unavailable = request
        .query(PlanarRead {
            body_key: vertices[0].body_key.clone(),
        })
        .execute();
    match unavailable {
        Err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
        ),
        Err(denial) => panic!("suspended generated materialization reported {denial:?}"),
        Ok(_) => panic!("suspended generated materialization must not be currently readable"),
    }
    assert_eq!(retained.body_key, vertices[0].body_key);

    let suspended = foreign_runtime_rejection(foreign, suspended);
    let suspended = wrong_producer_rejection(application, suspended);
    let suspended = incomplete_manifest_rejection(application, suspended, &vertices);
    let suspended = claim_denials_preserve_session(application, suspended, &vertices);
    let completed = complete(application, suspended, &vertices);
    application.fail_next_durable_append_for_test();
    let recovery = match application.restore_generated_output(completed, scope) {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(unpublished)) => {
            unpublished.into_recovery()
        }
        _ => panic!("the injected owner failure must retain opaque restoration recovery"),
    };
    assert_eq!(recovery.product_branch(), branch);
    let recovery = match foreign.continue_generated_output_restoration_recovery(recovery, scope) {
        Ok(_) => panic!("a foreign Query runtime must not settle restoration custody"),
        Err(failure) => {
            assert_eq!(
                failure.stage(),
                worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputRestorationRecoveryStage::StaleOccurrence,
            );
            failure.into_recovery()
        }
    };
    application
        .continue_generated_output_restoration_recovery(recovery, scope)
        .unwrap_or_else(|failure| {
            panic!(
                "Query must adopt the settled restoration into World: {:?}",
                failure.stage()
            )
        });

    for vertex in &vertices {
        let restored = read(request, &vertex.body_key);
        assert_eq!(restored.body_key, vertex.body_key);
        assert_eq!(restored.y, vertex.y);
    }
}

fn foreign_runtime_rejection(
    foreign: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
) -> worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput {
    match foreign.reconstruct_generated_output::<InitialPlanarProducer<ConsumerSchema>>(suspended) {
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
        .reconstruct_generated_output::<AlternatePlanarOutputProducer<ConsumerSchema>>(suspended)
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
        .reconstruct_generated_output::<InitialPlanarProducer<ConsumerSchema>>(suspended)
    {
        Ok(reconstruction) => reconstruction,
        Err(failure) => panic!(
            "the owning producer starts reconstruction: {:?}",
            failure.denial()
        ),
    };
    let first = claim_entity(&mut reconstruction, &vertices[0]);
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
        .reconstruct_generated_output::<InitialPlanarProducer<ConsumerSchema>>(suspended)
        .unwrap_or_else(|_| panic!("the owning producer restarts reconstruction"));
    let entities = vertices
        .iter()
        .map(|vertex| {
            let entity = claim_entity(&mut reconstruction, vertex);
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
    let duplicate = match reconstruction.entity(role(&vertices[0]), Body::reference()) {
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
    InitialPlanarProducer<ConsumerSchema>,
> {
    let mut reconstruction = application
        .reconstruct_generated_output::<InitialPlanarProducer<ConsumerSchema>>(suspended)
        .unwrap_or_else(|_| panic!("the owning producer completes reconstruction"));
    let entities = vertices
        .iter()
        .map(|vertex| {
            let entity = claim_entity(&mut reconstruction, vertex);
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

fn claim_entity(
    reconstruction: &mut worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        InitialPlanarProducer<ConsumerSchema>,
    >,
    vertex: &PlanarVertex,
) -> worth_query_host::facade::primary_graph::WorthQueryGeneratedEntity<ConsumerSchema, Body> {
    reconstruction
        .entity(role(vertex), Body::reference())
        .expect("the typed role resolves its retained platform identity")
}

fn write_fields(
    reconstruction: &mut worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputReconstruction<
        '_,
        ConsumerSchema,
        InitialPlanarProducer<ConsumerSchema>,
    >,
    entity: &worth_query_host::facade::primary_graph::WorthQueryGeneratedEntity<
        ConsumerSchema,
        Body,
    >,
    vertex: &PlanarVertex,
) {
    reconstruction
        .field(entity, BodyKey::reference(), vertex.body_key.clone())
        .expect("the typed body key encodes through Query");
    reconstruction
        .field(entity, PositionX::reference(), vertex.x)
        .expect("the typed x coordinate encodes through Query");
    reconstruction
        .field(entity, PositionY::reference(), vertex.y)
        .expect("the typed y coordinate encodes through Query");
    reconstruction
        .field(entity, Length::reference(), length(1))
        .expect("the typed length encodes through Query");
}

fn role(
    vertex: &PlanarVertex,
) -> WorthQueryApplicationOutputRole<
    PlanarMutationBinding<ConsumerSchema>,
    Body,
    WorthQueryCreateOutput,
> {
    WorthQueryApplicationOutputRole::try_new(format!("created.{}", vertex.body_key))
        .expect("fixture keys form valid source-derived output roles")
}

fn read(request: &Request<'_>, key: &str) -> worth_query_topology_entry::PlanarReadResult {
    let result = request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .unwrap_or_else(|denial| panic!("the generated body is readable: {denial:?}"));
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}

fn vertices() -> Vec<PlanarVertex> {
    [(201, 201), (210, 201), (201, 210)]
        .into_iter()
        .enumerate()
        .map(|(index, (x, y))| PlanarVertex {
            body_key: format!("rehydrated-{index}"),
            x: length(x),
            y: length(y),
        })
        .collect()
}

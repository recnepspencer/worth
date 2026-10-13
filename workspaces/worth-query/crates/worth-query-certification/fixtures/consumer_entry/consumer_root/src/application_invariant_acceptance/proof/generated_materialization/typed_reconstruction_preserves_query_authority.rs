//! typed reconstruction preserves query authority.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionRefusalKind as PrincipalRefusal;

pub(in super::super) fn typed_reconstruction_preserves_query_authority(
    application: &ProgramApplication,
    foreign: &WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>,
    request: &Request<'_>,
    scope: &WorthQueryRequestScope,
) {
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(observed_source(request, "anchor-b"))
        .idempotency(&980)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the source adjustment reaches publication");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the reconstruction source adjustment must publish")
    };
    let branch = performed.receipt().product_branch();
    let mut performed = performed
        .start_required_outputs(
            application,
            request,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .unwrap_or_else(|failure| {
            panic!("the declared output graph starts: {:?}", failure.denial())
        });
    let WorthQueryApplicationProgramOutputProgress::Settled(settlement) = performed
        .required_output_mut()
        .settle(application, request)
        .expect("the generated output graph settles within its admitted work bound")
    else {
        panic!("the generated output graph exceeded its admitted work bound");
    };
    assert_eq!(
        settlement
            .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
            .count(),
        1,
        "the program must publish its generated final output"
    );
    drop(performed);
    let retained_source = retained::initial_source_identity(request, application);
    let vertices = vertices();
    for vertex in &vertices {
        assert_eq!(read(request, &vertex.body_key).y, vertex.y);
    }
    let stale_source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the producer source query is admitted")
        .observed_sources()[0]
        .clone();
    successor::publish_unrelated(request, application, branch);
    let retained_result = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the producer source query is admitted");
    let source = retained_result.observed_sources()[0].clone();
    let wrong_source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the unrelated producer source query is admitted")
        .observed_sources()[0]
        .clone();
    let retained = read(request, &vertices[0].body_key);
    successor::require_stale_source_denial(application, scope, branch, stale_source);
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the Query-issued output occurrence remains selectable")
        })
        .expect("installed host call opens");
    match selected.suspend_current_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(
        scope, wrong_source,
    )
    {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure::Qualification(
            worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionDenial::SourceMismatch,
        )) => {}
        _ => panic!("an unrelated observed source must not suspend this output"),
    }
    application
        .with_host_advancement(|phase| {
            application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(application.product_runtime())
                    .expect("matching installed phase"),
            )
        })
        .expect("installed host call opens");
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the Query-issued output occurrence remains selectable")
        })
        .expect("installed host call opens");
    let recovery = match selected
        .suspend_current_generated_output::<PlanarFinalOutputProducer<ConsumerSchema>>(scope, source)
    {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputSuspensionFailure::ProductUnpublished(recovery)) => recovery,
        Err(failure) => panic!(
            "the injected owner failure must retain opaque suspension recovery; got {}",
            successor::failure_name(&failure)
        ),
        Ok(_) => panic!("the injected owner failure unexpectedly published"),
    };
    assert_eq!(recovery.product_branch(), branch);
    assert_eq!(read(request, &vertices[0].body_key), retained);
    let recovery = match foreign
        .resume_generated_output_suspension_recovery::<PlanarFinalOutputProducer<ConsumerSchema>>(
            recovery, scope,
        ) {
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
        .resume_generated_output_suspension_recovery::<PlanarFinalOutputProducer<ConsumerSchema>>(
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
            WorthQueryPrincipalResolutionDenialKind::Refused(
                PrincipalRefusal::BranchMaterializationSuspended
            ),
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
    application
        .with_host_advancement(|phase| {
            application.fail_next_durable_append_for_test(
                &phase
                    .checked_execution_request_for(application.product_runtime())
                    .expect("matching installed phase"),
            )
        })
        .expect("installed host call opens");
    let recovery = match application.restore_generated_output(completed, scope) {
        Err(worth_query_host::facade::primary_graph::WorthQueryGeneratedOutputRestorationFailure::ProductUnpublished(unpublished)) => {
            unpublished.into_recovery()
        }
        _ => panic!("the injected owner failure must retain opaque restoration recovery"),
    };
    assert_eq!(recovery.product_branch(), branch);
    let recovery = match foreign.resume_generated_output_restoration_recovery(recovery, scope) {
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
        .resume_generated_output_restoration_recovery(recovery, scope)
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
    // Restoration republishes the exact retained output, so exact reuse must
    // keep the producer resource profile admitted by its original demand.
    let restored_source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .expect("the restored producer source query is admitted");
    let selected = application
        .with_host_advancement(|host_phase| {
            application
                .select_output_producer::<PlanarFinalOutputFamily>(
                    &host_phase,
                    &restored_source.observed_sources()[0],
                    "planar-final",
                    4_096,
                )
                .unwrap_or_else(|denial| {
                    panic!("the restored output must keep its exact producer profile: {denial:?}")
                })
        })
        .expect("installed host call opens");
    assert_eq!(
        selected.identity(),
        "worth.query.certification.planar-final-output-producer.v2"
    );
    retained::preserved_outputs_remain_read_only(application, request, scope, retained_source);
}

#[cfg(test)]
mod journey_test {
    use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
    #[test]
    fn generated_reconstruction_journey_preserves_query_authority() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::{authentication, installation};
                let schema = crate::installed_schema();
                let world = installation::install(&schema);
                let foreign = installation::install(&schema);
                let scope = authentication::request_scope();
                let adapter = authentication::admit(world.application.installed_schema());
                let principal = authentication::block_on(adapter.authenticate(
                    authentication::LocalCredential::issued_for_model_owner(),
                    &scope,
                ))
                .unwrap();
                let request = world.application.request(&principal, &scope);
                super::typed_reconstruction_preserves_query_authority(
                    &world.application,
                    &foreign.application,
                    &request,
                    &scope,
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

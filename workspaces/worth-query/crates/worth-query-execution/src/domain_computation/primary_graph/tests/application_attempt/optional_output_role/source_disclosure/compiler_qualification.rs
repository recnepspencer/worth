//! A genuine source capture preserves qualification refusal before publication.
use super::*;
use worth_foundational::facade::with_refused_canonical_sort_qualification_for_test;
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryParameterDenialKind;

#[test]
fn original_source_reports_sort_qualification_without_execution_overflow_and_then_commits() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _placement = RestorePlacement::choose(Placement::Serial);
    let world = installed_authorization_world(true);
    let request = live_scope();
    for refuse in [true, false] {
        let before = world
            .application
            .with_host_advancement(|phase| {
                world
                    .selected_product(&phase.test_host_request())
                    .product()
                    .observation()
                    .clone()
            })
            .unwrap();
        world.application.with_application_advancement(&request, |phase| {
            let custody = phase.test_host_request();
            let principal = authenticated_principal(&custody, &world, &request);
            let account = resolved_account(&custody, &world, "open", &request);
            let installed = world.application.installed_schema().installed_mutation_binding::<OptionalOutputMutationBinding>().unwrap();
            assert!(installed.correction_binding().is_none());
            let admission = world.selected_product(&custody).authorize_operation(
                &phase, &principal, &account, installed.operation(), TypedMutationPreconditions::new(), &request,
            ).unwrap();
            let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
            let original = OptionalOutputInput { status: "open".to_owned(), plan: OptionalOutputPlan::RequiredOnly };
            let access = program_publication_access();
            let input = world.application.capture_operation_input_in_advancement::<<OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding>(
                &phase, &access, &request, &original,
            ).unwrap();
            let key = format!("source-compiler-qualification-{refuse}");
            let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, OptionalOutputMutationBinding>::with_encoded_input(&key, &input).unwrap();
            let idempotency = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            let outcome = world.application.execute_mutation_handler::<OptionalOutputMutationBinding>(
                &phase, &identities, principal.principal_identity(), admission, execution.allocation_policy(),
            ).unwrap();
            let HandlerResult::Completed(candidate) = outcome else { panic!("the genuine handler candidate is required") };
            let (program, _) = candidate.into_parts();
            let capture = || program.retain_original_mutation_input::<OptionalOutputMutationBinding>(
                &access, &phase, &world.application, input,
            );
            // Scope only the disputed capture, not earlier requirement preparation.
            let captured = if refuse {
                with_refused_canonical_sort_qualification_for_test(capture)
            } else {
                capture()
            };
            if refuse {
                let Err(denial) = captured else { panic!("the deliberately refused gate cannot capture a source") };
                assert_eq!(denial.kind(), WorthQueryApplicationAttemptDenialKind::SourceDisclosureCaptureDenied);
                let parameters = denial.source_disclosure_parameter_denial().expect("the original parameter stage survives");
                let WorthQueryApplicationQueryParameterDenialKind::UnsupportedSortImplementation(cause) = parameters.kind() else {
                    panic!("compiler qualification cannot become execution/resource overflow")
                };
                assert!(!cause.compiler_release().is_empty());
                assert_eq!(cause.compiler_commit().len(), 40);
                assert_eq!(cause.target_pointer_width(), usize::BITS);
            } else {
                let WorthQueryApplicationCommitOutcome::Committed(receipt) = world.application.compare_and_commit_application_in_advancement(
                    &phase, captured.unwrap(), idempotency, execution.allocation_policy(),
                ) else { panic!("the genuine qualified compiler twin must publish") };
                assert!(receipt.authority_binding().original_inspection_disclosure().is_some());
            }
        }).unwrap();
        let after = world
            .application
            .with_host_advancement(|phase| {
                world
                    .selected_product(&phase.test_host_request())
                    .product()
                    .observation()
                    .clone()
            })
            .unwrap();
        if refuse {
            assert_eq!(
                before, after,
                "qualification refusal publishes no Native truth"
            );
        } else {
            assert_ne!(before, after, "the qualified twin really commits");
        }
    }
}

//! A real non-correctable handler retains inspection parameters, not its business input.
use super::*;
#[path = "source_disclosure/native_funding.rs"]
mod native_funding;
use crate::domain_computation::primary_graph::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
use crate::publication_boundary::program_publication_access;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;

struct RestorePlacement(Placement);
impl RestorePlacement {
    fn choose(placement: Placement) -> Self {
        Self(place(placement))
    }
}
impl Drop for RestorePlacement {
    fn drop(&mut self) {
        place(self.0);
    }
}

#[test]
fn authored_non_correctable_source_disclosure_survives_portable_schema_admission() {
    use worth_query_declaration::facade::application_schema::{
        validate_portable_application_schema_freshly, ApplicationSchema,
    };
    use worth_query_installation::facade::*;
    use worth_query_package_archive::facade::*;

    let declaration = IdentityExecutionSchema::declaration().unwrap();
    let descriptor = OptionalOutputMutationBinding::descriptor();
    assert!(descriptor.description().inspection_disclosure().is_some());
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        IdentityExecutionSchema::OWNER,
        IdentityExecutionSchema::MAJOR,
        IdentityExecutionSchema::MINOR,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let records = package.export_typed_records().unwrap();
    let view = records
        .views()
        .find(|view| view.family() == WorthQueryPortablePackageRecordFamily::ApplicationSchema)
        .unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut bytes = encode_record_frame(view, limits).unwrap();
    assert_eq!(&bytes[..2], &18u16.to_be_bytes());
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&bytes)
        .unwrap();
    assert_eq!(decoded.record(), view.record());
    let WorthQueryPortablePackageRecord::ApplicationSchema(record) = decoded.into_record() else {
        panic!("real schema record required")
    };
    assert_eq!(
        &validate_portable_application_schema_freshly(record).unwrap(),
        declaration.erased()
    );
    bytes[..2].copy_from_slice(&17u16.to_be_bytes());
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(limits)
            .decode_frame(&bytes)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
}

#[test]
fn non_correctable_source_keeps_disclosure_and_releases_its_native_input() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _placement = RestorePlacement::choose(Placement::Leased(std::num::NonZeroUsize::MIN));
    let world = installed_authorization_world(true);
    let request = live_scope();
    let caller = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(64 << 20),
                None,
            ),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let sibling = world.application.with_funded_application_advancement(&request,
        worth_execution::ExecutionAllocationPolicy::Execution(&caller), |phase| {
        let custody = phase.test_host_request();
        let principal = authenticated_principal(&custody, &world, &request);
        let account = resolved_account(&custody, &world, "open", &request);
        let installed = world.application.installed_schema()
            .installed_mutation_binding::<OptionalOutputMutationBinding>().unwrap();
        assert!(installed.correction_binding().is_none(), "inspection has no inverse declaration");
        assert!(installed.retain_inspection_disclosure().is_some());
        let admission = world.selected_product(&custody).authorize_operation(
            &phase, &principal, &account, installed.operation(), TypedMutationPreconditions::new(), &request,
        ).unwrap();
        let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
        let worth_execution::ExecutionAllocationPolicy::Execution(parent) = execution.allocation_policy() else {
            panic!("the actual native source request owns the input reservation")
        };
        let original = OptionalOutputInput { status: "open".to_owned(), plan: OptionalOutputPlan::RequiredOnly };
        let before_input = parent.charged_memory_bytes();
        let input = world.application.capture_operation_input_in_advancement::<
            <OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding,
        >(&phase, &program_publication_access(), &request, &original).unwrap();
        let input_bytes = std::mem::size_of::<OptionalOutputInput>() as u64;
        assert_eq!(parent.charged_memory_bytes() - before_input, input_bytes);
        let key = "source-inspection-custody".to_owned();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, OptionalOutputMutationBinding>::with_encoded_input(&key, &input).unwrap();
        let idempotency = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
        let candidate = world.application.execute_mutation_handler_report::<OptionalOutputMutationBinding>(
            &phase, &identities, principal.principal_identity(), admission,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ).into_outcome().unwrap();
        let HandlerResult::Completed(candidate) = candidate else { panic!("real handler must complete") };
        let (program, result) = candidate.into_parts();
        drop(result);
        let access = program_publication_access();
        assert!(!program.requires_original_input_retention(&access));
        assert!(program.requires_original_source_capture(&access));
        let program = program.retain_original_mutation_input::<OptionalOutputMutationBinding>(
            &access, &phase, &world.application, input.clone(),
        ).unwrap();
        let receipt = match world.application.compare_and_commit_application_in_advancement(&phase, program, idempotency,
            execution.allocation_policy()) {
            WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
            outcome => panic!("real source commit refused: {outcome:?}"),
        };
        let source_commit = receipt.commit_reference().clone();
        let source_branch = receipt.product_branch();
        assert!(world.application.primary_provider.observe_completed_application(&source_commit).is_some());
        let carrier = receipt.authority_binding().clone();
        drop(receipt);
        assert!(carrier.retained_governed_input_carrier().is_none());
        let disclosure = carrier.original_inspection_disclosure().expect("actual source captured inspection meaning");
        assert_eq!(disclosure.scope().subject(), account.entity_id());
        assert_eq!(disclosure.association().source_type(), std::any::TypeId::of::<OptionalOutputMutationBinding>());
        assert_eq!(disclosure.disclosure().query(), world.application.installed_schema().installed_query_binding::<
            super::super::super::fixture::TestAccountSourceBinding,
        >().unwrap().query().identity());
        let sibling = carrier.clone();
        let holding_input = parent.charged_memory_bytes();
        drop(identities);
        drop(input);
        assert_eq!(holding_input - parent.charged_memory_bytes(), input_bytes,
            "the original native input backing releases while source payload and clone remain");
        let holding_payload = parent.charged_memory_bytes();
        drop(carrier);
        assert_eq!(parent.charged_memory_bytes(), holding_payload, "the source clone keeps its funded payload");
        assert!(sibling.original_inspection_disclosure().is_some());
        let closed = world.application.on_branch(source_branch).close(&phase).expect("the ordinary product occurrence closes");
        assert!(closed.is_complete());
        assert!(world.application.primary_provider.observe_completed_application(&source_commit).is_some(), "native history still owns the completed source after ordinary close");
        sibling
    }).unwrap();
    drop(world);
    assert!(
        caller.charged_memory_bytes() > 0,
        "the surviving actual source keeps its payload funded after provider/runtime teardown"
    );
    drop(sibling);
    assert_eq!(
        caller.charged_memory_bytes(),
        0,
        "the final actual source owner releases all original request backing"
    );
}

#[test]
fn serial_placement_source_capture_preserves_success_and_original_cancellation() {
    use crate::domain_computation::execution_runtime::product_world::test_product_world_resources;
    use crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world_with_product_resources;
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };

    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let _placement = RestorePlacement::choose(Placement::Serial);
    let resources =
        test_product_world_resources().with_execution_policy(ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::MIN,
        ));
    let world = installed_authorization_world_with_product_resources(resources);
    for cancel_capture in [true, false] {
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
        let cancellation = WorthQueryCancellationSource::new();
        let request = WorthQueryRequestScope::new(None, cancellation.token());
        let result = world.application.with_application_advancement(&request, |phase| {
            let execution = phase.checked_execution_request_for(world.application.product_runtime()).unwrap();
            let (accounted, _) = execution.run_native_with_checkpoint(|context| context.has_accounting()).unwrap();
            assert!(!accounted, "the original serial placement keeps its actual native cost posture");
            let custody = phase.test_host_request();
            let principal = authenticated_principal(&custody, &world, &request);
            let account = resolved_account(&custody, &world, "open", &request);
            let installed = world.application.installed_schema().installed_mutation_binding::<OptionalOutputMutationBinding>().unwrap();
            let admission = world.selected_product(&custody).authorize_operation(
                &phase, &principal, &account, installed.operation(), TypedMutationPreconditions::new(), &request,
            ).unwrap();
            let original = OptionalOutputInput { status: "open".to_owned(), plan: OptionalOutputPlan::RequiredOnly };
            let access = program_publication_access();
            let input = world.application.capture_operation_input_in_advancement::<<OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding>(
                &phase, &access, &request, &original,
            ).unwrap();
            let key = format!("serial-source-inspection-{cancel_capture}");
            let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, OptionalOutputMutationBinding>::with_encoded_input(&key, &input).unwrap();
            let idempotency = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
            let outcome = world.application.execute_mutation_handler::<OptionalOutputMutationBinding>(
                &phase, &identities, principal.principal_identity(), admission, execution.allocation_policy(),
            ).unwrap();
            let HandlerResult::Completed(candidate) = outcome else { panic!("actual handler must complete") };
            let (program, _) = candidate.into_parts();
            if cancel_capture { cancellation.cancel(); }
            let captured = program.retain_original_mutation_input::<OptionalOutputMutationBinding>(&access, &phase, &world.application, input);
            if cancel_capture {
                let Err(denial) = captured else { panic!("original cancellation must refuse capture") };
                assert_eq!(denial.kind(), WorthQueryApplicationAttemptDenialKind::Cancelled);
            } else {
                let WorthQueryApplicationCommitOutcome::Committed(receipt) = world.application.compare_and_commit_application_in_advancement(&phase, captured.unwrap(), idempotency, execution.allocation_policy()) else { panic!("unaccounted original source remains lawful") };
                assert!(receipt.authority_binding().original_inspection_disclosure().is_some());
                assert!(receipt.authority_binding().retained_governed_input_carrier().is_none());
            }
        });
        if cancel_capture {
            assert!(
                result.is_err(),
                "the canceled original request also refuses final handoff"
            );
        } else {
            result.unwrap();
        }
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
        if cancel_capture {
            assert_eq!(after, before, "capture refusal published no source change");
        } else {
            assert_ne!(
                after, before,
                "the lawful twin published its actual source occurrence"
            );
        }
    }
}

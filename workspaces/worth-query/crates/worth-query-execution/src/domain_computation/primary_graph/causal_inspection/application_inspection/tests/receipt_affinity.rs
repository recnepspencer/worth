//! Original receipt affinity consumes genuine published sources, not query stand-ins.
use super::{admission::InspectionAdmissionDenial, source_original::SourceOriginal};
use crate::domain_computation::primary_graph::application_entry::mutation::HandlerResult;
use crate::domain_computation::primary_graph::{
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, AuthorizationWorld, IdentityExecutionSchema,
            OptionalOutputInput, OptionalOutputMutationBinding, OptionalOutputPlan,
        },
    },
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationIdempotencyBinding, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::publication_boundary::program_publication_access;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIdentities,
        ApplicationSourceDisclosureBinding, ApplicationSourceDisclosureVisitor,
    },
    application_schema::TypedMutationPreconditions,
};

struct OriginalReceipt<'a> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<IdentityExecutionSchema>,
    receipt: &'a WorthQueryApplicationCommitReceipt,
}
impl ApplicationSourceDisclosureVisitor<IdentityExecutionSchema, OptionalOutputMutationBinding>
    for OriginalReceipt<'_>
{
    type Output = Result<SourceOriginal, InspectionAdmissionDenial>;
    fn unavailable(self) -> Self::Output {
        panic!("actual installed source declares inspection")
    }
    fn declared<
        Disclosure: ApplicationSourceDisclosureBinding<
            IdentityExecutionSchema,
            Source = OptionalOutputMutationBinding,
        >,
    >(
        self,
    ) -> Self::Output {
        let installed = self
            .runtime
            .installed_schema()
            .installed_query_binding::<Disclosure::Disclosure>()
            .unwrap();
        SourceOriginal::from_receipt::<IdentityExecutionSchema, Disclosure>(
            self.runtime,
            &installed,
            self.receipt,
        )
    }
}

#[test]
fn genuine_source_receipt_refuses_foreign_issuer_and_accepts_its_local_twin() {
    let local = installed_authorization_world(true);
    let foreign = installed_authorization_world(true);
    let local_source = publish_source(&local);
    let foreign_source = publish_source(&foreign);
    let readers = local.application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let original = OptionalOutputMutationBinding::visit_source_disclosure(OriginalReceipt {
        runtime: &local.application,
        receipt: &local_source,
    })
    .unwrap_or_else(|_| panic!("the actually captured local source must pass structural affinity"));
    assert!(original
        .changes
        .shares_original(local_source.committed_changes()));
    let denied = OptionalOutputMutationBinding::visit_source_disclosure(OriginalReceipt {
        runtime: &local.application,
        receipt: &foreign_source,
    });
    assert!(matches!(
        denied,
        Err(InspectionAdmissionDenial::ForeignIssuedAccess)
    ));
    let twin = OptionalOutputMutationBinding::visit_source_disclosure(OriginalReceipt {
        runtime: &foreign.application,
        receipt: &foreign_source,
    })
    .unwrap_or_else(|_| panic!("the same foreign-issued receipt is lawful at its real issuer"));
    assert!(twin
        .changes
        .shares_original(foreign_source.committed_changes()));
    drop((original, twin, local_source, foreign_source));
    assert_eq!(readers.observe().active(), baseline);
}

fn publish_source(world: &AuthorizationWorld) -> WorthQueryApplicationCommitReceipt {
    let request = live_scope();
    world.application.with_application_advancement(&request, |phase| {
        let custody = phase.test_host_request();
        let principal = authenticated_principal(&custody, world, &request);
        let account = resolved_account(&custody, world, "open", &request);
        let installed = world.application.installed_schema()
            .installed_mutation_binding::<OptionalOutputMutationBinding>().unwrap();
        let admission = world.selected_product(&custody).authorize_operation(
            &phase, &principal, &account, installed.operation(),
            TypedMutationPreconditions::new(), &request).unwrap();
        let original = OptionalOutputInput {
            status: "open".to_owned(), plan: OptionalOutputPlan::RequiredOnly,
        };
        let access = program_publication_access();
        let input = world.application.capture_operation_input_in_advancement::
            <<OptionalOutputMutationBinding as ApplicationMutationBinding<IdentityExecutionSchema>>::InputBinding>(
                &phase, &access, &request, &original).unwrap();
        let key = "receipt-affinity-real-source".to_owned();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema,
            OptionalOutputMutationBinding>::with_encoded_input(&key, &input).unwrap();
        let idempotency = WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities);
        let completed = world.application.execute_mutation_handler::<OptionalOutputMutationBinding>(
            &phase, &identities, principal.principal_identity(), admission,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
        let HandlerResult::Completed(completed) = completed else {
            panic!("the installed source handler must complete its actual read/write/output")
        };
        let (candidate, _) = completed.into_parts();
        let candidate = candidate.retain_original_mutation_input::<OptionalOutputMutationBinding>(
            &access, &phase, &world.application, input).unwrap();
        let outcome = world.application.compare_and_commit_application_in_advancement(
            &phase, candidate, idempotency,
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            panic!("actual source publication must commit: {outcome:?}")
        };
        assert!(receipt.authority_binding().original_inspection_disclosure().is_some());
        receipt
    }).unwrap()
}

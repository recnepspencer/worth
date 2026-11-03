//! Real typed source capture, correction derivation and ordinary owner publication.
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_execution::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_operation::*;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::causal_fact::WorthQueryPendingAftermathCausality;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::commit_causality::ApplicationCommitCausality;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::bootstrap;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::facade::domain_computation::HandlerResult;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyResolution;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource;

use crate::domain_computation::primary_graph::tests::fixture::correction_source::label_edit::*;

pub type LabelProgram =
    WorthQueryApplicationEffectProgram<IdentityExecutionSchema, EditLabel, LabelEdit, Account>;
pub struct PreparedLabelCorrection {
    pub program: LabelProgram,
    pub input: ApplicationEncodedInput<LabelInput>,
    pub key: String,
    pub binding: WorthQueryApplicationIdempotencyBinding,
    pub source: std::sync::Arc<WorthQueryPreparedCorrectionSource>,
}
impl PreparedLabelCorrection {
    pub fn pending(&self) -> WorthQueryPendingAftermathCausality {
        WorthQueryPendingAftermathCausality::recorded_correction_for_test(self.source.clone())
    }
}

pub fn install() -> AuthorizationWorld {
    let mut world = crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true);
    crate::domain_computation::primary_graph::tests::fixture::correction_source::program::install(
        &mut world,
    );
    world
}

pub fn install_handler(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        IdentityExecutionSchema,
    >,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
) {
    bootstrap
        .install_handler(
            &schema.installed_mutation_binding::<LabelBinding>().unwrap(),
            LabelHandler,
        )
        .unwrap();
}

pub fn edit(
    world: &AuthorizationWorld,
    key: &str,
    label: &str,
) -> WorthQueryApplicationCommitReceipt {
    let request =
        crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world.application.with_application_advancement(&request, |phase| {
        let input = world.application.capture_operation_input_in_advancement::<LabelInput>(
            &phase, &worth_query_execution::publication_boundary::program_publication_access(), &request,
            &LabelEdit { status: "open".to_owned(), label: label.to_owned() },
        ).unwrap();
        let key = key.to_owned();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(&key, &input).unwrap();
        let program = crate::domain_computation::primary_graph::tests::fixture::correction_source::preparation::candidate(world, &phase, &request, &input, &key);
        let selected = world.selected_product(&phase.test_host_request_for_test());
        let owner = world.application.selected_program_owner(&selected).unwrap();
        committed(owner.commit_program_action_in_advancement::<LabelBinding>(
            &phase, program, &identities, std::convert::identity, ExecutionAllocationPolicy::SystemAllocation,
        ))
    }).unwrap()
}

pub fn prepare(
    world: &AuthorizationWorld,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> PreparedLabelCorrection {
    crate::domain_computation::primary_graph::tests::fixture::correction_source::preparation::derive(
        world, source, key,
    )
}

pub fn publish(
    world: &AuthorizationWorld,
    prepared: PreparedLabelCorrection,
) -> WorthQueryApplicationCommitOutcome {
    let request =
        crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            publish_in_advancement(world, &phase, prepared)
        })
        .unwrap()
}

pub fn publish_in_advancement(
    world: &AuthorizationWorld,
    phase: &WorthQueryAdvancementPhase<'_>,
    prepared: PreparedLabelCorrection,
) -> WorthQueryApplicationCommitOutcome {
    let identities =
        ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(
            &prepared.key,
            &prepared.input,
        )
        .unwrap();
    let selected = world.selected_product(&phase.test_host_request_for_test());
    let owner = world.application.selected_program_owner(&selected).unwrap();
    owner.commit_program_correction_in_advancement::<LabelBinding>(
        &worth_query_execution::publication_boundary::program_publication_access(),
        phase,
        prepared.program,
        &identities,
        prepared.source,
        ExecutionAllocationPolicy::SystemAllocation,
    )
}

/// Deliberately omit only the causal fact while preserving a genuinely derived command.
pub fn publish_without_causality(
    world: &AuthorizationWorld,
    prepared: PreparedLabelCorrection,
) -> WorthQueryApplicationCommitOutcome {
    let request =
        crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let support = world.application.installed_program_support_for_test().unwrap();
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let inspection = selected.inspect_selected_program().unwrap();
            let presented = support.present_for_test(inspection.revision()).unwrap();
            world
                .application
                .compare_and_commit_application_for_program_action(
                    &phase.borrowed_request_for_test(),
                    &phase,
                    &presented,
                    prepared.program,
                    prepared.binding,
                    ApplicationCommitCausality::Ordinary,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
        })
        .unwrap()
}

pub fn committed(
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitReceipt {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        other => panic!("the real label action must commit: {other:?}"),
    }
}

pub fn replay(
    world: &AuthorizationWorld,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationIdempotencyResolution {
    let request =
        crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let (admission, _) = crate::domain_computation::primary_graph::tests::fixture::correction_source::preparation::admitted(world, &phase, &request);
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, binding)
                .unwrap()
                .into_resolution()
        })
        .unwrap()
}

#[path = "label_edit/mod.rs"]
pub mod label_edit;
#[path = "preparation/mod.rs"]
pub mod preparation;
#[path = "program/mod.rs"]
pub mod program;
#[path = "recorded/mod.rs"]
pub mod recorded;

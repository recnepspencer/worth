//! Real typed source capture, correction derivation and ordinary owner publication.
use super::{Account, AuthorizationWorld, IdentityExecutionSchema};
use crate::domain_computation::application_aftermath::{
    ApplicationCommitCausality, WorthQueryPendingAftermathCausality,
};
use crate::domain_computation::primary_graph::application_installation::WorthQueryProgramOwner;
use crate::domain_computation::primary_graph::*;
use crate::facade::runtime::ExecutionAllocationPolicy;
use worth_query_declaration::facade::application_operation::*;

#[path = "correction_source/label_edit.rs"]
pub(in crate::domain_computation::primary_graph) mod label_edit;
#[path = "correction_source/preparation.rs"]
mod preparation;
#[path = "correction_source/program.rs"]
mod program;
#[path = "correction_source/recorded.rs"]
pub(in crate::domain_computation::primary_graph) mod recorded;
use label_edit::*;

pub(in crate::domain_computation::primary_graph) type LabelProgram =
    WorthQueryApplicationEffectProgram<IdentityExecutionSchema, EditLabel, LabelEdit, Account>;
pub(in crate::domain_computation::primary_graph) struct PreparedLabelCorrection {
    pub program: LabelProgram,
    pub input: ApplicationEncodedInput<LabelInput>,
    pub key: String,
    pub binding: WorthQueryApplicationIdempotencyBinding,
    pub source: std::sync::Arc<WorthQueryPreparedCorrectionSource>,
}
impl PreparedLabelCorrection {
    pub fn pending(&self) -> WorthQueryPendingAftermathCausality {
        WorthQueryPendingAftermathCausality::recorded_correction(self.source.clone())
    }
}

pub(in crate::domain_computation::primary_graph) fn install() -> AuthorizationWorld {
    let mut world = super::installed_authorization_world(true);
    program::install(&mut world);
    world
}

pub(in crate::domain_computation::primary_graph) fn install_handler(
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

pub(in crate::domain_computation::primary_graph) fn edit(
    world: &AuthorizationWorld,
    key: &str,
    label: &str,
) -> WorthQueryApplicationCommitReceipt {
    let request = super::live_scope();
    world.application.with_application_advancement(&request, |phase| {
        let input = world.application.capture_operation_input_in_advancement::<LabelInput>(
            &phase, &crate::publication_boundary::program_publication_access(), &request,
            &LabelEdit { status: "open".to_owned(), label: label.to_owned() },
        ).unwrap();
        let key = key.to_owned();
        let identities = ApplicationMutationIdentities::<IdentityExecutionSchema, LabelBinding>::with_encoded_input(&key, &input).unwrap();
        let program = preparation::candidate(world, &phase, &request, &input, &key);
        let selected = world.selected_product(&phase.test_host_request());
        let owner = world.application.selected_program_owner(&selected).unwrap();
        committed(owner.commit_program_action_in_advancement::<LabelBinding>(
            &phase, program, &identities, std::convert::identity, ExecutionAllocationPolicy::SystemAllocation,
        ))
    }).unwrap()
}

pub(in crate::domain_computation::primary_graph) fn prepare(
    world: &AuthorizationWorld,
    source: &WorthQueryApplicationCommitReceipt,
    key: &str,
) -> PreparedLabelCorrection {
    preparation::derive(world, source, key)
}

pub(in crate::domain_computation::primary_graph) fn publish(
    world: &AuthorizationWorld,
    prepared: PreparedLabelCorrection,
) -> WorthQueryApplicationCommitOutcome {
    let request = super::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            publish_in_advancement(world, &phase, prepared)
        })
        .unwrap()
}

pub(in crate::domain_computation::primary_graph) fn publish_in_advancement(
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
    let selected = world.selected_product(&phase.test_host_request());
    let owner = world.application.selected_program_owner(&selected).unwrap();
    owner.commit_program_correction_in_advancement::<LabelBinding>(
        &crate::publication_boundary::program_publication_access(),
        phase,
        prepared.program,
        &identities,
        prepared.source,
        ExecutionAllocationPolicy::SystemAllocation,
    )
}

/// Deliberately omit only the causal fact while preserving a genuinely derived command.
pub(in crate::domain_computation::primary_graph) fn publish_without_causality(
    world: &AuthorizationWorld,
    prepared: PreparedLabelCorrection,
) -> WorthQueryApplicationCommitOutcome {
    let request = super::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let support = world.application.installed_program_support().unwrap();
            let selected = world.selected_product(&phase.test_host_request());
            let inspection = selected.inspect_selected_program().unwrap();
            let presented = support.present(inspection.revision()).unwrap();
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

pub(in crate::domain_computation::primary_graph) fn committed(
    outcome: WorthQueryApplicationCommitOutcome,
) -> WorthQueryApplicationCommitReceipt {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(receipt) => receipt,
        other => panic!("the real label action must commit: {other:?}"),
    }
}

pub(in crate::domain_computation::primary_graph) fn replay(
    world: &AuthorizationWorld,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationIdempotencyResolution {
    let request = super::live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let (admission, _) = preparation::admitted(world, &phase, &request);
            world
                .application
                .resolve_admitted_application_idempotency(&phase, &admission, binding)
                .unwrap()
                .into_resolution()
        })
        .unwrap()
}

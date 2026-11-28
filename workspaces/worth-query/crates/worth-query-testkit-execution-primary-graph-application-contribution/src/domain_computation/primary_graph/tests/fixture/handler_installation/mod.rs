//! The mutation handlers every fixture world installs before it publishes.

use crate::domain_computation::primary_graph::tests::fixture::capability_touch_binding::CapabilityTouchHandler;
use crate::domain_computation::primary_graph::tests::fixture::label_edit::{LabelBinding, LabelHandler};
use crate::domain_computation::primary_graph::tests::fixture::optional_output_binding::OptionalOutputHandler;
use crate::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredHandler;
use crate::domain_computation::primary_graph::tests::fixture::*;
use worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphBootstrap;

pub fn install_fixture_handlers(
    schema: &WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
) {
    let label = schema.installed_mutation_binding::<LabelBinding>().unwrap();
    bootstrap.install_handler(&label, LabelHandler).unwrap();
    let program_required = schema
        .installed_mutation_binding::<ProgramRequiredMutationBinding>()
        .unwrap();
    bootstrap
        .install_handler(&program_required, ProgramRequiredHandler)
        .unwrap();
    let capability_touch = schema
        .installed_mutation_binding::<CapabilityTouchMutationBinding>()
        .unwrap();
    bootstrap
        .install_handler(&capability_touch, CapabilityTouchHandler)
        .unwrap();
    let optional_output = schema
        .installed_mutation_binding::<OptionalOutputMutationBinding>()
        .unwrap();
    bootstrap
        .install_handler(&optional_output, OptionalOutputHandler)
        .unwrap();
}

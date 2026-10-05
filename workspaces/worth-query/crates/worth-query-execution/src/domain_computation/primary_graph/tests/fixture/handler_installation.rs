//! The mutation handlers every fixture world installs before it publishes.

use super::capability_touch_binding::CapabilityTouchHandler;
use super::optional_output_binding::OptionalOutputHandler;
use super::program_required_binding::ProgramRequiredHandler;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap;

pub(super) fn install_fixture_handlers(
    schema: &WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<IdentityExecutionSchema>,
) {
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

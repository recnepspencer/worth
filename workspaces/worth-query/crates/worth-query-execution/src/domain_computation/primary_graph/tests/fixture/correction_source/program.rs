//! Genuine admitted action meaning and World-published activation for correction tests.
use super::super::*;
use super::label_edit::LabelBinding;
use crate::domain_computation::primary_graph::application_installation::WorthQuerySupportedProgramRecord;
use crate::domain_computation::primary_graph::program_occurrence::{
    WorthQueryInstalledProgramSupport, WorthQueryProgramActivationCell,
};
use std::sync::Arc;
use worth_query_declaration::facade::application_program::*;
use worth_query_installation::facade::{
    install_rostered_application_program, WorthQueryProgramSupportAdmission,
};

struct LabelFeature;
impl ApplicationFeature<IdentityExecutionSchema> for LabelFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.label-feature.v1";
}
struct LabelProgram;
impl ApplicationProgramDefinition<IdentityExecutionSchema> for LabelProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.label-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, LabelFeature>()
                .native_action::<LabelBinding>()
                .finish_action()
                .finish(),
        ]
    }
}
pub(super) fn install(world: &mut AuthorizationWorld) {
    let program = ApplicationProgramAuthoring::<IdentityExecutionSchema, LabelProgram>::begin()
        .validated_program()
        .unwrap();
    let revision = *program.revision();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(
        world.application.installed_schema(),
    )
    .support(&program)
    .unwrap()
    .close()
    .unwrap();
    let installed = install_rostered_application_program(
        program,
        world.application.installed_schema(),
        &roster,
    )
    .unwrap();
    let record = WorthQuerySupportedProgramRecord::installed(Arc::new(installed), &[], &[]);
    let mut support = WorthQueryInstalledProgramSupport::installed(
        Arc::new(roster),
        WorthQueryProgramActivationCell::unpublished(),
    );
    support.complete_native_installation(vec![Arc::new(record)].into_boxed_slice());
    world.application.program_support = Some(support);
    world
        .application
        .with_host_advancement(|phase| {
            seed_program_activation(&phase.borrowed_request_for_test(), world, &revision);
        })
        .unwrap();
}

//! One authored program over the execution test schema, admitted through the
//! real support path so a test host carries genuine program support rather
//! than an assembled stand-in.

use super::*;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationNoOutputGraph, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramOutputs, ApplicationProgramRevision,
    ApplicationRuleLeaf, ValidatedApplicationProgram,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportAdmission,
};

use crate::domain_computation::primary_graph::program_occurrence::{
    WorthQueryInstalledProgramSupport, WorthQueryProgramActivationCell,
};

struct ProgramRequiredFeature;

impl ApplicationFeature<IdentityExecutionSchema> for ProgramRequiredFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.program-required-feature.v1";
}

/// The program that owns the one program-required mutation binding this test
/// schema installs.
pub(in crate::domain_computation::primary_graph) struct ProgramRequiredProgram;

impl ApplicationProgramDefinition<IdentityExecutionSchema> for ProgramRequiredProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.program-required-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, ProgramRequiredFeature>()
                .mutation::<ProgramRequiredMutationBinding>()
                .finish(),
        ]
    }
}

struct UnadmittedFeature;

impl ApplicationFeature<IdentityExecutionSchema> for UnadmittedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.unadmitted-feature.v1";
}

/// A second authored program over the same schema that no fixture host ever
/// rosters, so its canonical revision is meaning this host never admitted.
pub(in crate::domain_computation::primary_graph) struct UnadmittedProgram;

impl ApplicationProgramDefinition<IdentityExecutionSchema> for UnadmittedProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.unadmitted-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, UnadmittedFeature>()
                .mutation::<ProgramRequiredMutationBinding>()
                .finish(),
        ]
    }
}

fn validated<Program>() -> ValidatedApplicationProgram<IdentityExecutionSchema, Program>
where
    Program: ApplicationProgramDefinition<IdentityExecutionSchema>,
{
    ApplicationProgramAuthoring::<IdentityExecutionSchema, Program>::begin()
        .validated_program()
        .expect("the fixture programs are declaration-valid")
}

/// The canonical revision of the program the fixture host rosters.
pub(in crate::domain_computation::primary_graph) fn rostered_program_revision(
) -> ApplicationProgramRevision {
    validated::<ProgramRequiredProgram>().revision().clone()
}

/// The canonical revision of a program no fixture host ever rosters.
pub(in crate::domain_computation::primary_graph) fn unadmitted_program_revision(
) -> ApplicationProgramRevision {
    validated::<UnadmittedProgram>().revision().clone()
}

/// Admits the fixture program against a real installed schema and returns the
/// support a published host would retain for it.
///
/// The activation cell is left unpublished because the caller stands in for a
/// host whose branch has not yet seeded its activation record; every gate that
/// consults activation is expected to refuse on that evidence.
pub(in crate::domain_computation::primary_graph) fn installed_program_support(
    installed_schema: &WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
) -> WorthQueryInstalledProgramSupport<IdentityExecutionSchema> {
    let program = validated::<ProgramRequiredProgram>();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(installed_schema)
        .support(&program)
        .expect("the fixture program acts through an installed mutation binding")
        .close()
        .expect("the fixture program owns every installed rule contract");
    WorthQueryInstalledProgramSupport::installed(
        Arc::new(roster),
        WorthQueryProgramActivationCell::unpublished(),
    )
}

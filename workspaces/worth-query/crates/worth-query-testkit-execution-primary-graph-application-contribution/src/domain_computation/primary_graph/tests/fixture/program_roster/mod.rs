//! One authored program over the execution test schema, admitted through the
//! real support path so a test host carries genuine program support rather
//! than an assembled stand-in.

use crate::domain_computation::primary_graph::tests::fixture::*;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::ApplicationFeature;
use worth_query_declaration::facade::application_program::ApplicationFeatureInputLeaf;
use worth_query_declaration::facade::application_program::ApplicationFeatureSpec;
use worth_query_declaration::facade::application_program::ApplicationNoOutputGraph;
use worth_query_declaration::facade::application_program::ApplicationProgramAuthoring;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_declaration::facade::application_program::ApplicationProgramIdentity;
use worth_query_declaration::facade::application_program::ApplicationProgramOutputs;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_declaration::facade::application_program::ApplicationRuleLeaf;
use worth_query_declaration::facade::application_program::ValidatedApplicationProgram;
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;
use worth_query_installation::facade::WorthQueryProgramSupportAdmission;

use worth_query_execution::domain_computation::primary_graph::program_occurrence::activation_cell::WorthQueryProgramActivationCell;
use worth_query_execution::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport;

pub struct ProgramRequiredFeature;

impl ApplicationFeature<IdentityExecutionSchema> for ProgramRequiredFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.program-required-feature.v1";
}

/// The program that owns the one program-required mutation binding this test
/// schema installs.
pub struct ProgramRequiredProgram;

impl ApplicationProgramDefinition<IdentityExecutionSchema> for ProgramRequiredProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.program-required-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, ProgramRequiredFeature>()
                .native_action::<ProgramRequiredMutationBinding>()
                .finish_action()
                .finish(),
        ]
    }
}

pub struct UnadmittedFeature;

impl ApplicationFeature<IdentityExecutionSchema> for UnadmittedFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.test.unadmitted-feature.v1";
}

/// A second authored program over the same schema that no fixture host ever
/// rosters, so its canonical revision is meaning this host never admitted.
pub struct UnadmittedProgram;

impl ApplicationProgramDefinition<IdentityExecutionSchema> for UnadmittedProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.unadmitted-program.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<IdentityExecutionSchema, UnadmittedFeature>()
                .native_action::<ProgramRequiredMutationBinding>()
                .finish_action()
                .finish(),
        ]
    }
}

pub fn validated<Program>() -> ValidatedApplicationProgram<IdentityExecutionSchema, Program>
where
    Program: ApplicationProgramDefinition<IdentityExecutionSchema>,
{
    ApplicationProgramAuthoring::<IdentityExecutionSchema, Program>::begin()
        .validated_program()
        .expect("the fixture programs are declaration-valid")
}

/// The canonical revision of the program the fixture host rosters.
pub fn rostered_program_revision() -> ApplicationProgramRevision {
    *validated::<ProgramRequiredProgram>().revision()
}

/// The canonical revision of a program no fixture host ever rosters.
pub fn unadmitted_program_revision() -> ApplicationProgramRevision {
    *validated::<UnadmittedProgram>().revision()
}

/// Admits the fixture program against a real installed schema and returns the
/// support a published host would retain for it.
///
/// The activation cell is left unpublished because the caller stands in for a
/// host whose branch has not yet seeded its activation record; every gate that
/// consults activation is expected to refuse on that evidence.
pub fn installed_program_support(
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

pub struct ProvenanceSuccessorProgram;
impl ApplicationProgramDefinition<IdentityExecutionSchema> for ProvenanceSuccessorProgram {
    type Contributions = ();
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.test.provenance-successor.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        ProgramRequiredProgram::feature_specs()
    }
}

/// Data-only provenance fixture: both genuine revisions remain in admitted
/// meaning while activation is deliberately unpublished (not a historical grant).
pub fn installed_provenance_support(
    schema: &WorthQueryInstalledApplicationSchema<IdentityExecutionSchema>,
) -> (
    WorthQueryInstalledProgramSupport<IdentityExecutionSchema>,
    ApplicationProgramRevision,
    ApplicationProgramRevision,
) {
    let source = validated::<ProgramRequiredProgram>();
    let successor = validated::<ProvenanceSuccessorProgram>();
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(schema)
        .support(&source)
        .unwrap()
        .support(&successor)
        .unwrap()
        .close()
        .unwrap();
    let revisions = (*source.revision(), *successor.revision());
    (
        WorthQueryInstalledProgramSupport::installed(
            Arc::new(roster),
            WorthQueryProgramActivationCell::unpublished(),
        ),
        revisions.0,
        revisions.1,
    )
}

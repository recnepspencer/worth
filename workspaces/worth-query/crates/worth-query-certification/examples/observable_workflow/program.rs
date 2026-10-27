//! The mixed program composes the existing owning actions with inbound archival.
use crate::document_retention_model::{
    programs::{RetentionProgramP0, WorkflowOwnerProgram},
    schema::{DocumentRetentionContribution, DocumentRetentionSchema},
};
use worth_query_decl::facade::application_program::*;

pub struct MixedProgram;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for MixedProgram {
    type Contributions = (
        DocumentRetentionContribution,
        crate::effect::ArchiveContribution,
        crate::resource_monitor::MonitorContribution,
    );
    type Outputs =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules =
        <RetentionProgramP0 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.examples.observable-workflow.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.capability::<crate::document_retention_model::program_adoption::ProgramAdoptionBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut features = WorkflowOwnerProgram::feature_specs();
        features.push(
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, crate::effect::ArchiveFeature>(
            )
            .action::<crate::effect::ArchiveBinding, crate::effect::ArchiveDelivery>()
            .finish_action()
            .native_action::<crate::resource_monitor::MonitorBinding>()
            .finish_action()
            .finish(),
        );
        features
    }
}

pub fn validated_program() -> ValidatedApplicationProgram<DocumentRetentionSchema, MixedProgram> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, MixedProgram>::begin()
        .validated_program()
        .expect("the mixed workflow program validates")
}

/// A compatible rule upgrade preserves the example's actual workflow features.
pub struct MixedProgramP1;
impl ApplicationProgramDefinition<DocumentRetentionSchema> for MixedProgramP1 {
    type Contributions =
        <MixedProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Contributions;
    type Outputs = <MixedProgram as ApplicationProgramDefinition<DocumentRetentionSchema>>::Outputs;
    type Rules = <crate::document_retention_model::programs::RetentionProgramP1 as ApplicationProgramDefinition<DocumentRetentionSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.examples.observable-workflow.p1.v1");
    fn adoption_authorization<
        Visitor: ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        MixedProgram::adoption_authorization(visitor)
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        MixedProgram::feature_specs()
    }
}

pub fn validated_second_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, MixedProgramP1> {
    ApplicationProgramAuthoring::<DocumentRetentionSchema, MixedProgramP1>::begin()
        .validated_program()
        .expect("the compatible mixed workflow successor validates")
}

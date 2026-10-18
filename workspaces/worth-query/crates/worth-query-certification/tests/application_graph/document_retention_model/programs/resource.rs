//! Resource-governed program variants for artifact succession courts.
use super::*;
macro_rules! resource_program {
    ($program:ty, $artifact:ty, $identity:literal) => {
        impl ApplicationProgramDefinition<DocumentRetentionSchema> for $program {
            type Contributions = (DocumentRetentionContribution,);
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = CommitBoundaryRule<DocumentRetentionV1>;

            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);

            fn adoption_authorization<Visitor: worth_query_host::facade::declaration::application_program::ApplicationProgramAdoptionAuthorizationVisitor<DocumentRetentionSchema>>(visitor: Visitor) -> Visitor::Output {
        visitor.capability::<super::super::program_adoption::ProgramAdoptionBinding>()
    }

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                vec![ApplicationFeatureSpec::root::<
                    DocumentRetentionSchema,
                    DocumentRetentionFeature,
                >()
                .action::<SetRetentionBinding, RetentionCompletion>().finish_action()
                .conditional_operation::<PublishRetentionAssessment>()
                .derived_artifact::<$artifact>()
                .finish()]
            }
        }
    };
}

resource_program!(
    ResourceRetentionProgramP0,
    RetentionArtifactP0,
    "worth.query.certification.document-retention.resource-p0.v1"
);
resource_program!(
    ResourceRetentionProgramP1,
    RetentionArtifactP1,
    "worth.query.certification.document-retention.resource-p1.v1"
);

//! The programs authored over the document-retention schema.
//!
//! P0 and P1 declare the same feature and the same action. They differ in one
//! thing only: which document-retention rule contract they declare. That single
//! difference is what the court holds the platform to.

use worth_query_host::facade::declaration::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationChangePosture,
    ApplicationChangeShape, ApplicationCommitBoundary, ApplicationDerivedArtifact,
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureSpec,
    ApplicationLocalRuleRef, ApplicationLocalityGranule, ApplicationLocalityScope,
    ApplicationNoOutputGraph, ApplicationOutputPort, ApplicationProgramAuthoring,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramOutputs,
    ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList, ValidatedApplicationProgram,
};

use super::assessment_output::PublishRetentionAssessment;
use super::retention_entry::{ReviewedSetRetentionBinding, SetRetentionBinding};
use super::schema::{
    DocumentRetentionContribution, DocumentRetentionRowBinding, DocumentRetentionSchema,
    DocumentRetentionV1, DocumentRetentionV2, DocumentRetentionV3,
};
use super::workflow::{
    ReviewRequirementBinding, UnlinkReviewRequirementBinding, WorkflowAdvanceBinding,
    WorkflowApprovalBinding, WorkflowDefinitionAuthoringBinding, WorkflowGrantStatusBinding,
    WorkflowInstanceStartBinding,
};

#[path = "programs/removed_assessment_supplier.rs"]
mod removed_assessment_supplier;
pub use removed_assessment_supplier::{
    validated_removed_assessment_supplier_program, RemovedAssessmentSupplierProgram,
};

/// The one feature both programs govern.
pub struct DocumentRetentionFeature;
pub struct DocumentRetentionFeatureV2;

impl ApplicationFeature<DocumentRetentionSchema> for DocumentRetentionFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.certification.document-retention.feature.v1";
}

impl ApplicationFeature<DocumentRetentionSchema> for DocumentRetentionFeatureV2 {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.certification.document-retention.feature.v1";
    const MAJOR: u16 = 2;
}

type CommitBoundaryRule<Invariant> = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<DocumentRetentionSchema, DocumentRetentionFeature, Invariant>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

/// The program that reads `document-retention-v1` as the law.
pub struct RetentionProgramP0;

/// The program that reads `document-retention-v2` as the law.
pub struct RetentionProgramP1;
/// A rostered target whose feature meaning cannot be carried without migration.
pub struct ChangedFeatureRetentionProgram;
/// A rostered target that deliberately removes the only ordinary operation.
pub struct RemovedOperationRetentionProgram;
/// A rostered target that keeps the operation identity but changes its
/// locality/change contract, forcing fresh current admission.
pub struct ChangedOperationRetentionProgram;
pub struct ResourceRetentionProgramP0;
pub struct ResourceRetentionProgramP1;

struct ChangedOperationLocality;
struct ChangedOperationShape;
struct RetentionArtifactOutput;
struct RetentionArtifactP0;
struct RetentionArtifactP1;

impl ApplicationLocalityScope for ChangedOperationLocality {
    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.changed-operation-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}

impl ApplicationChangeShape for ChangedOperationShape {
    const IDENTITY: &'static str =
        "worth.query.certification.document-retention.changed-operation-shape.v1";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Replace;
}

impl ApplicationOutputPort<DocumentRetentionSchema, DocumentRetentionFeature>
    for RetentionArtifactOutput
{
    type Value = DocumentRetentionRowBinding;

    const IDENTITY: &'static str = "worth.query.certification.retention-artifact-output.v1";
}

macro_rules! retention_artifact {
    ($artifact:ty, $work:expr, $bytes:expr) => {
        impl ApplicationDerivedArtifact<DocumentRetentionSchema, DocumentRetentionFeature>
            for $artifact
        {
            type Output = RetentionArtifactOutput;
            type Locality = ChangedOperationLocality;

            const IDENTITY: &'static str = "worth.query.certification.retention-artifact.v1";
            const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
            const SUCCESSION: ApplicationArtifactSuccession =
                ApplicationArtifactSuccession::PreserveWhenEquivalent;
            const REQUIRED: bool = false;
            const PRODUCER_FAMILY: &'static str =
                "worth.query.certification.retention-artifact-producer.v1";
            const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
                &[ApplicationArtifactDependency::new(
                    DocumentRetentionFeature::IDENTITY,
                )];
            const REUSE_RULE: &'static str =
                "worth.query.certification.retention-artifact-reuse.v1";
            const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
                ApplicationArtifactResourceCeiling::new($work, $bytes);
            const STOPPED_OUTCOME: &'static str =
                "worth.query.certification.retention-artifact-stopped.v1";
        }
    };
}

retention_artifact!(RetentionArtifactP0, 8, 4096);
retention_artifact!(RetentionArtifactP1, 16, 8192);

/// A program this court never rosters on any host.
pub struct UnrosteredRetentionProgram;

/// A program declaring a rule contract no host installs.
pub struct ForeignRuleRetentionProgram;

impl ApplicationProgramDefinition<DocumentRetentionSchema> for RetentionProgramP0 {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.document-retention.p0.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        retention_feature_specs()
    }
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for RetentionProgramP1 {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV2>;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.document-retention.p1.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        retention_feature_specs()
    }
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for ChangedFeatureRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.changed-feature.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeatureV2>()
                .mutation::<SetRetentionBinding>()
                .conditional_operation::<PublishRetentionAssessment>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for RemovedOperationRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.removed-operation.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .mutation::<ReviewRequirementBinding>()
                .mutation::<UnlinkReviewRequirementBinding>()
                .conditional_operation::<PublishRetentionAssessment>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for ChangedOperationRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.changed-operation.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
                .mutation_with_locality_and_change::<
                    SetRetentionBinding,
                    ChangedOperationLocality,
                    ChangedOperationShape,
                >()
                .mutation::<ReviewRequirementBinding>()
                .mutation::<UnlinkReviewRequirementBinding>()
                .conditional_operation::<PublishRetentionAssessment>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

macro_rules! resource_program {
    ($program:ty, $artifact:ty, $identity:literal) => {
        impl ApplicationProgramDefinition<DocumentRetentionSchema> for $program {
            type Contributions = (DocumentRetentionContribution,);
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = CommitBoundaryRule<DocumentRetentionV1>;

            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);

            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                vec![ApplicationFeatureSpec::root::<
                    DocumentRetentionSchema,
                    DocumentRetentionFeature,
                >()
                .mutation::<SetRetentionBinding>()
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

impl ApplicationProgramDefinition<DocumentRetentionSchema> for UnrosteredRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.unrostered.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        retention_feature_specs()
    }
}

impl ApplicationProgramDefinition<DocumentRetentionSchema> for ForeignRuleRetentionProgram {
    type Contributions = (DocumentRetentionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<DocumentRetentionV3>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.document-retention.foreign-rule.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        retention_feature_specs()
    }
}

pub fn validated_first_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP0> {
    validate::<RetentionProgramP0>()
}

pub fn validated_second_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, RetentionProgramP1> {
    validate::<RetentionProgramP1>()
}

pub fn validated_changed_feature_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ChangedFeatureRetentionProgram> {
    validate::<ChangedFeatureRetentionProgram>()
}

pub fn validated_removed_operation_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, RemovedOperationRetentionProgram> {
    validate::<RemovedOperationRetentionProgram>()
}

pub fn validated_changed_operation_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ChangedOperationRetentionProgram> {
    validate::<ChangedOperationRetentionProgram>()
}

pub fn validated_first_resource_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ResourceRetentionProgramP0> {
    validate::<ResourceRetentionProgramP0>()
}

pub fn validated_second_resource_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ResourceRetentionProgramP1> {
    validate::<ResourceRetentionProgramP1>()
}

pub fn validated_foreign_rule_program(
) -> ValidatedApplicationProgram<DocumentRetentionSchema, ForeignRuleRetentionProgram> {
    validate::<ForeignRuleRetentionProgram>()
}

fn validate<Program>() -> ValidatedApplicationProgram<DocumentRetentionSchema, Program>
where
    Program: ApplicationProgramDefinition<DocumentRetentionSchema>,
    Program::Outputs:
        worth_query_host::facade::declaration::application_program::ApplicationProgramOutputsShape<
            DocumentRetentionSchema,
        >,
{
    ApplicationProgramAuthoring::<DocumentRetentionSchema, Program>::begin()
        .validated_program()
        .expect("the authored document-retention program must validate")
}

fn retention_feature_specs() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<DocumentRetentionSchema, DocumentRetentionFeature>()
            .mutation::<SetRetentionBinding>()
            .mutation::<ReviewedSetRetentionBinding>()
            .mutation::<ReviewRequirementBinding>()
            .mutation::<UnlinkReviewRequirementBinding>()
            .conditional_operation::<PublishRetentionAssessment>()
            .mutation::<WorkflowDefinitionAuthoringBinding>()
            .mutation::<WorkflowInstanceStartBinding>()
            .mutation::<WorkflowAdvanceBinding>()
            .mutation::<WorkflowApprovalBinding>()
            .mutation::<WorkflowGrantStatusBinding>()
            .finish(),
    ]
}

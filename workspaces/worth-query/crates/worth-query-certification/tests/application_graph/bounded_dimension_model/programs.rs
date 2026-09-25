//! The programs authored over the bounded-dimension schema.
//!
//! P0 and P1 declare the same feature and the same action. They differ in one
//! thing only: which bounded-dimension rule contract they declare. That single
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

use super::assessment_output::PublishPartAssessment;
use super::dimension_entry::{ReviewedSetPartDimensionBinding, SetPartDimensionBinding};
use super::schema::{
    BoundedDimensionContribution, BoundedDimensionSchema, BoundedDimensionV1, BoundedDimensionV2,
    BoundedDimensionV3, PartDimensionRowBinding,
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
pub struct BoundedDimensionFeature;
pub struct BoundedDimensionFeatureV2;

impl ApplicationFeature<BoundedDimensionSchema> for BoundedDimensionFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.certification.bounded-dimension.feature.v1";
}

impl ApplicationFeature<BoundedDimensionSchema> for BoundedDimensionFeatureV2 {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.certification.bounded-dimension.feature.v1";
    const MAJOR: u16 = 2;
}

type CommitBoundaryRule<Invariant> = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<BoundedDimensionSchema, BoundedDimensionFeature, Invariant>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

/// The program that reads `bounded-dimension-v1` as the law.
pub struct DimensionProgramP0;

/// The program that reads `bounded-dimension-v2` as the law.
pub struct DimensionProgramP1;
/// A rostered target whose feature meaning cannot be carried without migration.
pub struct ChangedFeatureDimensionProgram;
/// A rostered target that deliberately removes the only ordinary operation.
pub struct RemovedOperationDimensionProgram;
/// A rostered target that keeps the operation identity but changes its
/// locality/change contract, forcing fresh current admission.
pub struct ChangedOperationDimensionProgram;
pub struct ResourceDimensionProgramP0;
pub struct ResourceDimensionProgramP1;

struct ChangedOperationLocality;
struct ChangedOperationShape;
struct DimensionArtifactOutput;
struct DimensionArtifactP0;
struct DimensionArtifactP1;

impl ApplicationLocalityScope for ChangedOperationLocality {
    const IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.changed-operation-locality.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}

impl ApplicationChangeShape for ChangedOperationShape {
    const IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.changed-operation-shape.v1";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Replace;
}

impl ApplicationOutputPort<BoundedDimensionSchema, BoundedDimensionFeature>
    for DimensionArtifactOutput
{
    type Value = PartDimensionRowBinding;

    const IDENTITY: &'static str = "worth.query.certification.dimension-artifact-output.v1";
}

macro_rules! dimension_artifact {
    ($artifact:ty, $work:expr, $bytes:expr) => {
        impl ApplicationDerivedArtifact<BoundedDimensionSchema, BoundedDimensionFeature>
            for $artifact
        {
            type Output = DimensionArtifactOutput;
            type Locality = ChangedOperationLocality;

            const IDENTITY: &'static str = "worth.query.certification.dimension-artifact.v1";
            const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
            const SUCCESSION: ApplicationArtifactSuccession =
                ApplicationArtifactSuccession::PreserveWhenEquivalent;
            const REQUIRED: bool = false;
            const PRODUCER_FAMILY: &'static str =
                "worth.query.certification.dimension-artifact-producer.v1";
            const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
                &[ApplicationArtifactDependency::new(
                    BoundedDimensionFeature::IDENTITY,
                )];
            const REUSE_RULE: &'static str =
                "worth.query.certification.dimension-artifact-reuse.v1";
            const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
                ApplicationArtifactResourceCeiling::new($work, $bytes);
            const STOPPED_OUTCOME: &'static str =
                "worth.query.certification.dimension-artifact-stopped.v1";
        }
    };
}

dimension_artifact!(DimensionArtifactP0, 8, 4096);
dimension_artifact!(DimensionArtifactP1, 16, 8192);

/// A program this court never rosters on any host.
pub struct UnrosteredDimensionProgram;

/// A program declaring a rule contract no host installs.
pub struct ForeignRuleDimensionProgram;

impl ApplicationProgramDefinition<BoundedDimensionSchema> for DimensionProgramP0 {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.bounded-dimension.p0.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        dimension_feature_specs()
    }
}

impl ApplicationProgramDefinition<BoundedDimensionSchema> for DimensionProgramP1 {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV2>;

    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.bounded-dimension.p1.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        dimension_feature_specs()
    }
}

impl ApplicationProgramDefinition<BoundedDimensionSchema> for ChangedFeatureDimensionProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.changed-feature.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<BoundedDimensionSchema, BoundedDimensionFeatureV2>()
                .mutation::<SetPartDimensionBinding>()
                .conditional_operation::<PublishPartAssessment>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<BoundedDimensionSchema> for RemovedOperationDimensionProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.removed-operation.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<BoundedDimensionSchema, BoundedDimensionFeature>()
                .mutation::<ReviewRequirementBinding>()
                .mutation::<UnlinkReviewRequirementBinding>()
                .conditional_operation::<PublishPartAssessment>()
                .mutation::<WorkflowDefinitionAuthoringBinding>()
                .mutation::<WorkflowInstanceStartBinding>()
                .mutation::<WorkflowAdvanceBinding>()
                .mutation::<WorkflowApprovalBinding>()
                .mutation::<WorkflowGrantStatusBinding>()
                .finish(),
        ]
    }
}

impl ApplicationProgramDefinition<BoundedDimensionSchema> for ChangedOperationDimensionProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.changed-operation.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<BoundedDimensionSchema, BoundedDimensionFeature>()
                .mutation_with_locality_and_change::<
                    SetPartDimensionBinding,
                    ChangedOperationLocality,
                    ChangedOperationShape,
                >()
                .mutation::<ReviewRequirementBinding>()
                .mutation::<UnlinkReviewRequirementBinding>()
                .conditional_operation::<PublishPartAssessment>()
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
        impl ApplicationProgramDefinition<BoundedDimensionSchema> for $program {
            type Contributions = (BoundedDimensionContribution,);
            type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
            type Rules = CommitBoundaryRule<BoundedDimensionV1>;

            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);

            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                vec![
                    ApplicationFeatureSpec::root::<
                        BoundedDimensionSchema,
                        BoundedDimensionFeature,
                    >()
                    .mutation::<SetPartDimensionBinding>()
                    .conditional_operation::<PublishPartAssessment>()
                    .derived_artifact::<$artifact>()
                    .finish(),
                ]
            }
        }
    };
}

resource_program!(
    ResourceDimensionProgramP0,
    DimensionArtifactP0,
    "worth.query.certification.bounded-dimension.resource-p0.v1"
);
resource_program!(
    ResourceDimensionProgramP1,
    DimensionArtifactP1,
    "worth.query.certification.bounded-dimension.resource-p1.v1"
);

impl ApplicationProgramDefinition<BoundedDimensionSchema> for UnrosteredDimensionProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV1>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.unrostered.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        dimension_feature_specs()
    }
}

impl ApplicationProgramDefinition<BoundedDimensionSchema> for ForeignRuleDimensionProgram {
    type Contributions = (BoundedDimensionContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = CommitBoundaryRule<BoundedDimensionV3>;

    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.bounded-dimension.foreign-rule.v1",
    );

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        dimension_feature_specs()
    }
}

pub fn validated_first_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, DimensionProgramP0> {
    validate::<DimensionProgramP0>()
}

pub fn validated_second_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, DimensionProgramP1> {
    validate::<DimensionProgramP1>()
}

pub fn validated_changed_feature_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, ChangedFeatureDimensionProgram> {
    validate::<ChangedFeatureDimensionProgram>()
}

pub fn validated_removed_operation_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, RemovedOperationDimensionProgram> {
    validate::<RemovedOperationDimensionProgram>()
}

pub fn validated_changed_operation_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, ChangedOperationDimensionProgram> {
    validate::<ChangedOperationDimensionProgram>()
}

pub fn validated_first_resource_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, ResourceDimensionProgramP0> {
    validate::<ResourceDimensionProgramP0>()
}

pub fn validated_second_resource_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, ResourceDimensionProgramP1> {
    validate::<ResourceDimensionProgramP1>()
}

pub fn validated_foreign_rule_program(
) -> ValidatedApplicationProgram<BoundedDimensionSchema, ForeignRuleDimensionProgram> {
    validate::<ForeignRuleDimensionProgram>()
}

fn validate<Program>() -> ValidatedApplicationProgram<BoundedDimensionSchema, Program>
where
    Program: ApplicationProgramDefinition<BoundedDimensionSchema>,
    Program::Outputs:
        worth_query_host::facade::declaration::application_program::ApplicationProgramOutputsShape<
            BoundedDimensionSchema,
        >,
{
    ApplicationProgramAuthoring::<BoundedDimensionSchema, Program>::begin()
        .validated_program()
        .expect("the authored bounded-dimension program must validate")
}

fn dimension_feature_specs() -> Vec<ApplicationFeatureSpec> {
    vec![
        ApplicationFeatureSpec::root::<BoundedDimensionSchema, BoundedDimensionFeature>()
            .mutation::<SetPartDimensionBinding>()
            .mutation::<ReviewedSetPartDimensionBinding>()
            .mutation::<ReviewRequirementBinding>()
            .mutation::<UnlinkReviewRequirementBinding>()
            .conditional_operation::<PublishPartAssessment>()
            .mutation::<WorkflowDefinitionAuthoringBinding>()
            .mutation::<WorkflowInstanceStartBinding>()
            .mutation::<WorkflowAdvanceBinding>()
            .mutation::<WorkflowApprovalBinding>()
            .mutation::<WorkflowGrantStatusBinding>()
            .finish(),
    ]
}

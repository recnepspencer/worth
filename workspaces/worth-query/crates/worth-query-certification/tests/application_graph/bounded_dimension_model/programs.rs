//! The programs authored over the bounded-dimension schema.
//!
//! P0 and P1 declare the same feature and the same action. They differ in one
//! thing only: which bounded-dimension rule contract they declare. That single
//! difference is what the court holds the platform to.

use worth_query_host::facade::declaration::application_program::{
    ApplicationCommitBoundary, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureSpec, ApplicationLocalRuleRef, ApplicationNoOutputGraph,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRuleAt, ApplicationRuleLeaf, ApplicationRuleList,
    ValidatedApplicationProgram,
};

use super::dimension_entry::SetPartDimensionBinding;
use super::schema::{
    BoundedDimensionContribution, BoundedDimensionSchema, BoundedDimensionV1, BoundedDimensionV2,
    BoundedDimensionV3,
};

/// The one feature both programs govern.
pub struct BoundedDimensionFeature;

impl ApplicationFeature<BoundedDimensionSchema> for BoundedDimensionFeature {
    type Inputs = ApplicationFeatureInputLeaf;

    const IDENTITY: &'static str = "worth.query.certification.bounded-dimension.feature.v1";
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
            .finish(),
    ]
}

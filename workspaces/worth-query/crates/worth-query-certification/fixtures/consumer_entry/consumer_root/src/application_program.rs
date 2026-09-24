use worth_query_decl::facade::application_program::{
    ApplicationCommitBoundary, ApplicationCompositionInstance, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef, ApplicationFeature,
    ApplicationFeatureInputLeaf, ApplicationFeatureInputList, ApplicationFeatureSpec,
    ApplicationInputPort, ApplicationLocalRuleRef, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputEdge, ApplicationOutputGraph, ApplicationOutputLeaf,
    ApplicationProgramAuthoring, ApplicationProgramDefinition, ApplicationProgramIdentity,
    ApplicationProgramOutputs, ApplicationRootComposition, ApplicationRuleAt, ApplicationRuleLeaf,
    ApplicationRuleList, ApplicationSharedRuleRef,
};
use worth_query_parameter_entry::{ParameterFeature, PositiveParameterCount};
use worth_query_topology_entry::{
    MutatePlanar, PlanarAlternateDerivedBodyInput, PlanarAlternateFinalBodyOutput,
    PlanarAlternateFinalOutputFeature, PlanarAlternateFinalToSummaryConnection,
    PlanarAlternateSummaryFeature, PlanarAlternateSummaryInput, PlanarBodyInput, PlanarBodyOutput,
    PlanarDerivedBodyInput, PlanarDerivedBodyOutput, PlanarEditBinding, PlanarFinalBodyOutput,
    PlanarFinalOutputFeature, PlanarFinalToSummaryConnection, PlanarOutputFeature,
    PlanarOutputToAlternateFinalConnection, PlanarOutputToFinalConnection,
    PlanarSourceAdjustmentBinding, PlanarSourceFeature, PlanarSourceToOutputConnection,
    PlanarSummaryFeature, PlanarSummaryInput, PositivePlanarTurn, PreserveFinalPlanarOutput,
    PriorCycleAdjustmentBinding, PublishAlternatePlanarOutput, PublishFinalPlanarOutput,
    VertexReplacementBinding,
};

use crate::ConsumerSchema;
pub(crate) mod correspondence;
pub(crate) mod external_input;
mod features;
mod omitted_program_binding;
mod roots;
mod validation_denials;
pub(crate) use omitted_program_binding::validated_omitted_program_binding;
pub use roots::{
    ConsumerDiscoveredProgramRoot, ConsumerProgramRoot, ConsumerRequiredSharedRoot,
    ConsumerSecondaryProgramRoot, ConsumerTruncatedProgramRoot, ConsumerUndeclaredProgramRoot,
    DiscoveredPlanarRoot, RequiredSharedPlanarRoot, SecondaryPlanarRoot,
};
pub(crate) use validation_denials::{
    duplicate_feature_is_denied, missing_required_input_is_denied, undeclared_input_is_denied,
    unexported_cross_instance_is_denied,
};
pub struct ConsumerProgram;
pub struct OmittedInstalledRuleProgram;
pub struct RequiredSourceAsActionProgram;
struct MissingRequiredInputProgram;
struct DuplicateFeatureProgram;
struct UndeclaredInputProgram;
struct UnexportedCrossInstanceProgram;
struct NestedPlanarInstance;
struct MissingRequiredFeature;
struct UndeclaredTargetFeature;
struct UnconnectedPlanarInput;
struct UndeclaredConnection;
impl ApplicationCompositionInstance for NestedPlanarInstance {
    const PATH: &'static str = "certification.nested";
}
impl ApplicationFeature<ConsumerSchema> for MissingRequiredFeature {
    type Inputs = ApplicationFeatureInputList<UnconnectedPlanarInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "missing-required-feature";
}
impl ApplicationFeature<ConsumerSchema> for UndeclaredTargetFeature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "undeclared-target-feature";
}
impl ApplicationInputPort<ConsumerSchema, MissingRequiredFeature> for UnconnectedPlanarInput {
    type Value = worth_query_topology_entry::PlanarReadResultBinding;

    const IDENTITY: &'static str = "unconnected-required";
    const REQUIRED: bool = true;
}
impl ApplicationInputPort<ConsumerSchema, UndeclaredTargetFeature> for UnconnectedPlanarInput {
    type Value = worth_query_topology_entry::PlanarReadResultBinding;
    const IDENTITY: &'static str = "unconnected-required";
    const REQUIRED: bool = true;
}
impl ApplicationConnectionIdentity for UndeclaredConnection {
    const IDENTITY: &'static str = "undeclared-input-connection";
}
impl
    ApplicationOccurrenceConnectionBinding<
        ConsumerSchema,
        PlanarSourceFeature,
        UndeclaredTargetFeature,
    > for UndeclaredConnection
{
}
type PlanarConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarSourceToOutputConnection,
>;

type PlanarDependentConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;

pub fn validated_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        ConsumerProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<ConsumerSchema, ConsumerProgram>::begin().validated_program()
}

pub fn validated_omitted_installed_rule_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        OmittedInstalledRuleProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    ApplicationProgramAuthoring::<ConsumerSchema, OmittedInstalledRuleProgram>::begin()
        .validated_program()
}

type PlanarAlternateDependentConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateDerivedBodyInput,
    PlanarOutputToAlternateFinalConnection,
>;

type PlanarAlternateSummaryConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarAlternateFinalOutputFeature,
    PlanarAlternateFinalBodyOutput,
    PlanarAlternateSummaryFeature,
    PlanarAlternateSummaryInput,
    PlanarAlternateFinalToSummaryConnection,
>;

type MissingRequiredInputRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationSharedRuleRef<ConsumerSchema, PositivePlanarTurn>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleLeaf,
>;

impl ApplicationProgramDefinition<ConsumerSchema> for MissingRequiredInputProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs =
        ApplicationProgramOutputs<ApplicationOutputGraph<PlanarConnection, ApplicationOutputLeaf>>;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.missing-required-input.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarOutputFeature>().finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, MissingRequiredFeature>().finish(),
        ]
    }
}

impl ApplicationProgramDefinition<ConsumerSchema> for DuplicateFeatureProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs =
        ApplicationProgramOutputs<ApplicationOutputGraph<PlanarConnection, ApplicationOutputLeaf>>;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.duplicate-feature.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarOutputFeature>().finish(),
        ]
    }
}

type UndeclaredInputConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    UndeclaredTargetFeature,
    UnconnectedPlanarInput,
    UndeclaredConnection,
>;

impl ApplicationProgramDefinition<ConsumerSchema> for UndeclaredInputProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<
        ApplicationOutputGraph<UndeclaredInputConnection, ApplicationOutputLeaf>,
    >;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.undeclared-input.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, UndeclaredTargetFeature>().finish(),
        ]
    }
}

type UnexportedCrossInstanceConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    NestedPlanarInstance,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToFinalConnection,
>;

impl ApplicationProgramDefinition<ConsumerSchema> for UnexportedCrossInstanceProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<
        ApplicationOutputGraph<
            PlanarConnection,
            ApplicationOutputEdge<UnexportedCrossInstanceConnection, ApplicationOutputLeaf>,
        >,
    >;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.unexported-cross-instance.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<
                ConsumerSchema,
                NestedPlanarInstance,
                PlanarFinalOutputFeature,
            >()
            .finish(),
        ]
    }
}

type ConsumerRules = ApplicationRuleList<
    ApplicationRuleAt<
        ApplicationLocalRuleRef<ConsumerSchema, ParameterFeature, PositiveParameterCount>,
        ApplicationCommitBoundary,
    >,
    ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<ConsumerSchema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >,
>;

type PlanarSummaryConnection = ApplicationConnectionRef<
    ConsumerSchema,
    PlanarFinalOutputFeature,
    PlanarFinalBodyOutput,
    PlanarSummaryFeature,
    PlanarSummaryInput,
    PlanarFinalToSummaryConnection,
>;

impl ApplicationProgramDefinition<ConsumerSchema> for ConsumerProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<(
        ConsumerProgramRoot,
        (
            ConsumerSecondaryProgramRoot,
            (ConsumerDiscoveredProgramRoot, ConsumerRequiredSharedRoot),
        ),
    )>;
    type Rules = ConsumerRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.consumer-program.v2");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::consumer_feature_specs()
    }
}

impl ApplicationProgramDefinition<ConsumerSchema> for OmittedInstalledRuleProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.omitted-installed-rule.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        features::consumer_feature_specs()
    }
}

impl ApplicationProgramDefinition<ConsumerSchema> for RequiredSourceAsActionProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = ConsumerRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.source-action-overlap.v1");

    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = features::consumer_feature_specs();
        let source = specs
            .iter()
            .position(|spec| {
                spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY
            })
            .expect("the source feature is present");
        specs.remove(source);
        specs.push(
            ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .mutation::<PlanarEditBinding<ConsumerSchema>>()
                .mutation::<PriorCycleAdjustmentBinding<ConsumerSchema>>()
                .mutation::<VertexReplacementBinding<ConsumerSchema>>()
                .mutation::<PlanarSourceAdjustmentBinding<ConsumerSchema>>()
                .finish(),
        );
        specs
    }
}

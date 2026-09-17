use worth_query_decl::facade::application_program::{
    ApplicationActionLeaf, ApplicationActionList, ApplicationActionRef, ApplicationCommitBoundary,
    ApplicationCompositionInstance, ApplicationConditionalOperationActionRef,
    ApplicationConnectionIdentity, ApplicationConnectionInstanceRef, ApplicationConnectionRef,
    ApplicationFeature, ApplicationFeatureInputLeaf, ApplicationFeatureInputList,
    ApplicationFeatureInstanceRef, ApplicationFeatureLeaf, ApplicationFeatureList,
    ApplicationFeatureRef, ApplicationInputPort, ApplicationLocalRuleRef,
    ApplicationOccurrenceConnectionBinding, ApplicationOutputEdge, ApplicationOutputGraph,
    ApplicationOutputLeaf, ApplicationProgramAuthoring, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationRootComposition, ApplicationRuleAt, ApplicationRuleLeaf,
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
    PlanarSummaryFeature, PlanarSummaryInput, PositivePlanarTurn, PriorCycleAdjustmentBinding,
    PublishAlternatePlanarOutput, PublishFinalPlanarOutput, VertexReplacementBinding,
};

use crate::ConsumerSchema;

mod validation_denials;
pub use validation_denials::{
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

type MissingRequiredInputFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, PlanarOutputFeature>,
        ApplicationFeatureList<
            ApplicationFeatureRef<ConsumerSchema, MissingRequiredFeature>,
            ApplicationFeatureLeaf,
        >,
    >,
>;

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
    type Actions = ApplicationActionLeaf;
    type Features = MissingRequiredInputFeatures;
    type OutputGraph = ApplicationOutputGraph<PlanarConnection, ApplicationOutputLeaf>;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.missing-required-input.v1");
}

type DuplicateFeatureFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
        ApplicationFeatureList<
            ApplicationFeatureRef<ConsumerSchema, PlanarOutputFeature>,
            ApplicationFeatureLeaf,
        >,
    >,
>;

impl ApplicationProgramDefinition<ConsumerSchema> for DuplicateFeatureProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Actions = ApplicationActionLeaf;
    type Features = DuplicateFeatureFeatures;
    type OutputGraph = ApplicationOutputGraph<PlanarConnection, ApplicationOutputLeaf>;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.duplicate-feature.v1");
}

type UndeclaredInputFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, UndeclaredTargetFeature>,
        ApplicationFeatureLeaf,
    >,
>;

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
    type Actions = ApplicationActionLeaf;
    type Features = UndeclaredInputFeatures;
    type OutputGraph = ApplicationOutputGraph<UndeclaredInputConnection, ApplicationOutputLeaf>;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.undeclared-input.v1");
}

type UnexportedCrossInstanceFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, PlanarOutputFeature>,
        ApplicationFeatureList<
            ApplicationFeatureInstanceRef<
                ConsumerSchema,
                NestedPlanarInstance,
                PlanarFinalOutputFeature,
            >,
            ApplicationFeatureLeaf,
        >,
    >,
>;

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
    type Actions = ApplicationActionLeaf;
    type Features = UnexportedCrossInstanceFeatures;
    type OutputGraph = ApplicationOutputGraph<
        PlanarConnection,
        ApplicationOutputEdge<UnexportedCrossInstanceConnection, ApplicationOutputLeaf>,
    >;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.unexported-cross-instance.v1");
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

type ConsumerActions = ApplicationActionList<
    ApplicationActionRef<ConsumerSchema, PlanarSourceFeature, PlanarEditBinding<ConsumerSchema>>,
    ApplicationActionList<
        ApplicationActionRef<
            ConsumerSchema,
            PlanarSourceFeature,
            PriorCycleAdjustmentBinding<ConsumerSchema>,
        >,
        ApplicationActionList<
            ApplicationActionRef<
                ConsumerSchema,
                PlanarSourceFeature,
                VertexReplacementBinding<ConsumerSchema>,
            >,
            ApplicationActionList<
                ApplicationConditionalOperationActionRef<
                    ConsumerSchema,
                    PlanarOutputFeature,
                    MutatePlanar,
                >,
                ApplicationActionList<
                    ApplicationConditionalOperationActionRef<
                        ConsumerSchema,
                        PlanarFinalOutputFeature,
                        PublishFinalPlanarOutput,
                    >,
                    ApplicationActionList<
                        ApplicationConditionalOperationActionRef<
                            ConsumerSchema,
                            PlanarAlternateFinalOutputFeature,
                            PublishAlternatePlanarOutput,
                        >,
                        ApplicationActionLeaf,
                    >,
                >,
            >,
        >,
    >,
>;

type ConsumerFeatures = ApplicationFeatureList<
    ApplicationFeatureRef<ConsumerSchema, PlanarSourceFeature>,
    ApplicationFeatureList<
        ApplicationFeatureRef<ConsumerSchema, PlanarOutputFeature>,
        ApplicationFeatureList<
            ApplicationFeatureRef<ConsumerSchema, PlanarFinalOutputFeature>,
            ApplicationFeatureList<
                ApplicationFeatureRef<ConsumerSchema, PlanarAlternateFinalOutputFeature>,
                ApplicationFeatureList<
                    ApplicationFeatureRef<ConsumerSchema, PlanarSummaryFeature>,
                    ApplicationFeatureList<
                        ApplicationFeatureRef<ConsumerSchema, PlanarAlternateSummaryFeature>,
                        ApplicationFeatureList<
                            ApplicationFeatureRef<ConsumerSchema, ParameterFeature>,
                            ApplicationFeatureLeaf,
                        >,
                    >,
                >,
            >,
        >,
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
    type Actions = ConsumerActions;
    type Features = ConsumerFeatures;
    type OutputGraph = ApplicationOutputGraph<
        PlanarConnection,
        (
            ApplicationOutputEdge<
                PlanarDependentConnection,
                ApplicationOutputEdge<PlanarSummaryConnection, ApplicationOutputLeaf>,
            >,
            ApplicationOutputEdge<
                PlanarAlternateDependentConnection,
                ApplicationOutputEdge<PlanarAlternateSummaryConnection, ApplicationOutputLeaf>,
            >,
        ),
    >;
    type Rules = ConsumerRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.consumer-program.v1");
}

impl ApplicationProgramDefinition<ConsumerSchema> for OmittedInstalledRuleProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Actions = ConsumerActions;
    type Features = ConsumerFeatures;
    type OutputGraph =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::OutputGraph;
    type Rules = MissingRequiredInputRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.omitted-installed-rule.v1");
}

impl ApplicationProgramDefinition<ConsumerSchema> for RequiredSourceAsActionProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Actions = ApplicationActionList<
        ApplicationActionRef<
            ConsumerSchema,
            PlanarSourceFeature,
            PlanarSourceAdjustmentBinding<ConsumerSchema>,
        >,
        ConsumerActions,
    >;
    type Features = ConsumerFeatures;
    type OutputGraph =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::OutputGraph;
    type Rules = ConsumerRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.source-action-overlap.v1");
}

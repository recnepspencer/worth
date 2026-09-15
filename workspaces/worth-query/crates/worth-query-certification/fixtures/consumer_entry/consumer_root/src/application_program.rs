use worth_query_decl::facade::application_program::{
    validate_application_program, ApplicationConnectionRef, ApplicationFeatureAvailable,
    ApplicationFeatureUnavailable, ApplicationProgramDefinition,
    ApplicationProgramDependentConnection, ApplicationProgramFeature, ApplicationProgramIdentity,
    ApplicationProgramInventory, ApplicationProgramInventoryIdentity, ApplicationProgramLocalRule,
    ApplicationProgramOutput, ApplicationProgramRequiredConnection, ApplicationProgramSharedRule,
    ApplicationProgramUnavailableConnection, ApplicationProgramUnavailableSharedRule,
    AtCommitBoundary,
};
use worth_query_decl::facade::application_schema::ApplicationInvariantMarkerIdentity;
use worth_query_parameter_entry::{ParameterFeature, PositiveParameterCount};
use worth_query_topology_entry::{
    PlanarBodyInput, PlanarBodyOutput, PlanarDerivedBodyInput, PlanarDerivedBodyOutput,
    PlanarFinalBodyOutput, PlanarFinalOutputFeature, PlanarOutputFeature,
    PlanarOutputToFinalConnection, PlanarSourceFeature, PlanarSourceToOutputConnection,
    PositivePlanarTurn,
};

use crate::ConsumerSchema;

pub struct ConsumerProgram;
pub struct UnavailableConsumerProgram;
pub struct MissingRuleProviderProgram;
pub struct StaleUnavailableRuleProgram;
pub struct ConsumerOutputs;

impl ApplicationProgramInventoryIdentity for ConsumerOutputs {
    const IDENTITY: &'static str = "worth.query.certification.consumer-outputs.v1";
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
    validate_application_program::<ConsumerSchema, ConsumerProgram>()
}

type SourceFeature = ApplicationProgramFeature<
    PlanarSourceFeature,
    (),
    (PlanarBodyOutput,),
    ApplicationFeatureAvailable,
>;
type OutputFeature = ApplicationProgramFeature<
    PlanarOutputFeature,
    (PlanarBodyInput,),
    (PlanarDerivedBodyOutput,),
    ApplicationFeatureAvailable,
>;
type FinalFeature = ApplicationProgramFeature<
    PlanarFinalOutputFeature,
    (PlanarDerivedBodyInput,),
    (PlanarFinalBodyOutput,),
    ApplicationFeatureAvailable,
>;
type UnavailableFinalFeature = ApplicationProgramFeature<
    PlanarFinalOutputFeature,
    (PlanarDerivedBodyInput,),
    (PlanarFinalBodyOutput,),
    ApplicationFeatureUnavailable,
>;
type ParameterProgramFeature =
    ApplicationProgramFeature<ParameterFeature, (), (), ApplicationFeatureAvailable>;
pub type ConsumerProgramRoot = ApplicationProgramRequiredConnection<PlanarConnection>;
pub type ConsumerProgramInventory = ConsumerOutputs;

type ProgramConnections = (
    ConsumerProgramRoot,
    ApplicationProgramDependentConnection<PlanarDependentConnection>,
);
type UnavailableProgramConnections = (
    ConsumerProgramRoot,
    ApplicationProgramUnavailableConnection<PlanarDependentConnection>,
);
type ProgramRules = (
    ApplicationProgramLocalRule<ParameterFeature, PositiveParameterCount, AtCommitBoundary>,
    ApplicationProgramSharedRule<PositivePlanarTurn, AtCommitBoundary>,
);
pub struct PlannedFutureRule;
impl ApplicationInvariantMarkerIdentity<ConsumerSchema> for PlannedFutureRule {
    const IDENTIFIER: &'static str = "PlannedFutureRule";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub struct PlannedPositivePlanarTurnV2;
impl ApplicationInvariantMarkerIdentity<ConsumerSchema> for PlannedPositivePlanarTurnV2 {
    const IDENTIFIER: &'static str =
        <PositivePlanarTurn as ApplicationInvariantMarkerIdentity<ConsumerSchema>>::IDENTIFIER;
    const MAJOR: u16 =
        <PositivePlanarTurn as ApplicationInvariantMarkerIdentity<ConsumerSchema>>::MAJOR + 1;
    const MINOR: u16 = 0;
}
type UnavailableProgramRules = (
    ApplicationProgramLocalRule<ParameterFeature, PositiveParameterCount, AtCommitBoundary>,
    ApplicationProgramSharedRule<PositivePlanarTurn, AtCommitBoundary>,
    ApplicationProgramUnavailableSharedRule<PlannedFutureRule, AtCommitBoundary>,
    ApplicationProgramUnavailableSharedRule<PlannedPositivePlanarTurnV2, AtCommitBoundary>,
);
type MissingRuleProviderRules = (
    ApplicationProgramLocalRule<ParameterFeature, PositiveParameterCount, AtCommitBoundary>,
    ApplicationProgramSharedRule<PositivePlanarTurn, AtCommitBoundary>,
    ApplicationProgramSharedRule<PlannedFutureRule, AtCommitBoundary>,
);
type StaleUnavailableRuleRules = (
    ApplicationProgramLocalRule<ParameterFeature, PositiveParameterCount, AtCommitBoundary>,
    ApplicationProgramUnavailableSharedRule<PositivePlanarTurn, AtCommitBoundary>,
);
type ProgramInventories = (
    ApplicationProgramInventory<
        ConsumerOutputs,
        (ApplicationProgramOutput<PlanarFinalOutputFeature, PlanarFinalBodyOutput>,),
    >,
);

impl ApplicationProgramDefinition<ConsumerSchema> for ConsumerProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Features = (
        SourceFeature,
        OutputFeature,
        FinalFeature,
        ParameterProgramFeature,
    );
    type Connections = ProgramConnections;
    type Rules = ProgramRules;
    type Inventories = ProgramInventories;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.consumer-program.v1");
}

impl ApplicationProgramDefinition<ConsumerSchema> for UnavailableConsumerProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Features = (
        SourceFeature,
        OutputFeature,
        UnavailableFinalFeature,
        ParameterProgramFeature,
    );
    type Connections = UnavailableProgramConnections;
    type Rules = UnavailableProgramRules;
    type Inventories = ProgramInventories;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.unavailable-consumer-program.v1",
    );
}

impl ApplicationProgramDefinition<ConsumerSchema> for MissingRuleProviderProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Features = (
        SourceFeature,
        OutputFeature,
        FinalFeature,
        ParameterProgramFeature,
    );
    type Connections = ProgramConnections;
    type Rules = MissingRuleProviderRules;
    type Inventories = ProgramInventories;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.missing-rule-provider-program.v1",
    );
}

impl ApplicationProgramDefinition<ConsumerSchema> for StaleUnavailableRuleProgram {
    type Contributions = <ConsumerSchema as worth_query_decl::facade::application_schema::ApplicationSchemaComposition>::Contributions;
    type Features = (
        SourceFeature,
        OutputFeature,
        FinalFeature,
        ParameterProgramFeature,
    );
    type Connections = ProgramConnections;
    type Rules = StaleUnavailableRuleRules;
    type Inventories = ProgramInventories;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(
        "worth.query.certification.stale-unavailable-rule-program.v1",
    );
}

pub fn validated_unavailable_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        UnavailableConsumerProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    validate_application_program::<ConsumerSchema, UnavailableConsumerProgram>()
}

pub fn validated_missing_rule_provider_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        MissingRuleProviderProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    validate_application_program::<ConsumerSchema, MissingRuleProviderProgram>()
}

pub fn validated_stale_unavailable_rule_program() -> Result<
    worth_query_decl::facade::application_program::ValidatedApplicationProgram<
        ConsumerSchema,
        StaleUnavailableRuleProgram,
    >,
    worth_query_decl::facade::application_program::ApplicationProgramValidationDenial,
> {
    validate_application_program::<ConsumerSchema, StaleUnavailableRuleProgram>()
}

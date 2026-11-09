//! The actual authored source-to-replacement output connection.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    WorthQueryRequiredOutputConnectionDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Schema;
impl worth_query_decl::facade::application_schema::fragment::Fragment for Schema {
    const NAME: &'static str = "Schema";
    type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
    type Members = (installation::Contribution,);
}
impl worth_query_decl::facade::application_schema::ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.certification.output-availability";
    const NAME: &'static str = "Schema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration() -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclaration<Self>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        worth_query_decl::facade::application_schema::fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        worth_query_decl::facade::application_schema::fragment::visit_correction(
            Self, source, visitor,
        )
    }
}
impl worth_query_decl::facade::application_schema::ApplicationSchemaComposition for Schema {
    type Contributions = (installation::Contribution,);
}

impl TopologySchemaBinding for Schema {
    type PrincipalFragment = crate::PrincipalRecords;
}
pub(super) struct QuantityFeature;
pub(super) struct QuantityInput;
pub(super) struct QuantityOutput;
pub(super) struct SourceToQuantity;
struct SourceChange;
impl worth_query_decl::facade::application_program::ApplicationChangeShape for SourceChange {
    const IDENTITY: &'static str = "availability-source-field-preservation";
    const POSTURE: worth_query_decl::facade::application_program::ApplicationChangePosture =
        worth_query_decl::facade::application_program::ApplicationChangePosture::Preserve;
}

impl ApplicationFeature<Schema> for QuantityFeature {
    type Inputs = ApplicationFeatureInputList<QuantityInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "worth.query.certification.availability-quantity-feature.v1";
}
impl ApplicationInputPort<Schema, QuantityFeature> for QuantityInput {
    type Value = PlanarReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<Schema, QuantityFeature> for QuantityOutput {
    type Value = value::SnapshotResultBinding;
    const IDENTITY: &'static str = "quantity";
}
impl ApplicationConnectionIdentity for SourceToQuantity {
    const IDENTITY: &'static str = "worth.query.certification.source-to-availability-quantity.v1";
}
impl ApplicationOccurrenceConnectionBinding<Schema, PlanarSourceFeature, QuantityFeature>
    for SourceToQuantity
{
}
impl WorthQueryApplicationRequiredOutputConnection<Schema> for SourceToQuantity {
    type Source = PlanarSourceAdjustmentBinding<Schema>;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand(source.scope_key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, SourceToQuantity>
    for PlanarSourceAdjustmentBinding<Schema>
{
    fn demand_from_source(
        source: &PlanarSourceAdjustment,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <SourceToQuantity as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(source)
    }
}

type Connection = ApplicationConnectionRef<
    Schema,
    PlanarSourceFeature,
    PlanarBodyOutput,
    QuantityFeature,
    QuantityInput,
    SourceToQuantity,
>;
pub(super) type QuantityRoot = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub(super) type GeometryRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        Schema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        PlanarOutputFeature,
        PlanarBodyInput,
        PlanarSourceToOutputConnection,
    >,
    (
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                Schema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarFinalOutputFeature,
                PlanarDerivedBodyInput,
                PlanarOutputToFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
        ApplicationOutputEdge<
            ApplicationConnectionRef<
                Schema,
                PlanarOutputFeature,
                PlanarDerivedBodyOutput,
                PlanarAlternateFinalOutputFeature,
                PlanarAlternateDerivedBodyInput,
                PlanarOutputToAlternateFinalConnection,
            >,
            ApplicationOutputLeaf,
        >,
    ),
>;
pub(super) struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = <Schema as ApplicationSchemaComposition>::Contributions;
    type Outputs = ApplicationProgramOutputs<(QuantityRoot, GeometryRoot)>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationSharedRuleRef<Schema, PositivePlanarTurn>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("worth.query.certification.output-availability-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            ApplicationFeatureSpec::root::<Schema, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .native_action::<PlanarSourceAdjustmentBinding<Schema>>()
                .locality_and_change::<
                    crate::checkpoint_recovery::demand_policy::PlanarLocality,
                    SourceChange,
                >()
                .finish_action()
                .native_action::<PlanarEditBinding<Schema>>()
                .finish_action()
                .finish(),
            required_chain::output_feature_spec_for::<Schema>(),
            ApplicationFeatureSpec::root::<Schema, PlanarFinalOutputFeature>()
                .provides::<PlanarFinalBodyOutput>()
                .required_pair::<PlanarFinalFeatureRoles<Schema>>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, PlanarAlternateFinalOutputFeature>()
                .provides::<PlanarAlternateFinalBodyOutput>()
                .conditional_operation::<PublishAlternatePlanarOutput>()
                .finish(),
            ApplicationFeatureSpec::root::<Schema, QuantityFeature>()
                .derived_artifact::<QuantityArtifact>()
                .managed_computation::<collection::Totals>()
                .derived_collection::<collection::Quantities>()
                .conditional_operation::<binding::PublishQuantity>()
                .finish(),
        ]
    }
}

pub(super) struct QuantityArtifact;
impl ApplicationDerivedArtifact<Schema, QuantityFeature> for QuantityArtifact {
    type Output = QuantityOutput;
    type Locality = crate::checkpoint_recovery::demand_policy::PlanarLocality;
    const IDENTITY: &'static str = "availability-quantity-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession =
        ApplicationArtifactSuccession::PreserveWhenEquivalent;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <producer::Family as WorthQueryProducerOutputFamily<Schema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarSourceAdjustmentBinding<Schema> as ApplicationMutationBinding<Schema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "native-source-publication";
    const STOPPED_OUTCOME: &'static str = "availability-owner-stopped";
}

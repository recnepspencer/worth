//! Two ordinary roots; only the successor program selects the relation rule.
use super::{
    super::{edit, program as changed, schema::*},
    producer, query,
};
use worth_query_host::facade::{declaration::application_program::*, primary_graph::*};
#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub struct SourceProgram;
#[allow(
    dead_code,
    reason = "the same schema-typed owner is compiled for both V1 and V2; each journey uses its actual edition"
)]
pub struct TargetProgram;
pub struct Feature;
pub struct Input;
pub struct Connection;
pub type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        CanonicalSchema,
        changed::SourceFeature,
        changed::SourcePort,
        Feature,
        Input,
        Connection,
    >,
    ApplicationOutputLeaf,
>;
impl ApplicationFeature<CanonicalSchema> for Feature {
    type Inputs = ApplicationFeatureInputList<Input, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "canonical-companion.feature.v1";
}
impl ApplicationInputPort<CanonicalSchema, Feature> for Input {
    type Value = query::ResultBinding;
    const IDENTITY: &'static str = "canonical-companion.input.v1";
    const REQUIRED: bool = true;
}
impl ApplicationOccurrenceConnectionBinding<CanonicalSchema, changed::SourceFeature, Feature>
    for Connection
{
}
impl ApplicationConnectionIdentity for Connection {
    const IDENTITY: &'static str = "canonical-companion.connection.v1";
}
impl WorthQueryApplicationRequiredOutputConnection<CanonicalSchema> for Connection {
    type Source = edit::EditBinding;
    type Demand = producer::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Input,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producer::Demand)
    }
}
impl WorthQueryApplicationRequiredOutputSource<CanonicalSchema, Connection> for edit::EditBinding {
    fn demand_from_source(
        source: &edit::Input,
    ) -> Result<producer::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Connection as WorthQueryApplicationRequiredOutputConnection<CanonicalSchema>>::demand_from_source(source)
    }
}
impl ApplicationProgramDefinition<CanonicalSchema> for SourceProgram {
    type Contributions = (CanonicalContribution,);
    type Outputs = ApplicationProgramOutputs<(changed::Root, Root)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("canonical-companion.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <changed::Program as ApplicationProgramDefinition<CanonicalSchema>>::feature_specs()
    }
}
impl ApplicationProgramDefinition<CanonicalSchema> for TargetProgram {
    type Contributions = (CanonicalContribution,);
    type Outputs = ApplicationProgramOutputs<(changed::Root, Root)>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<CanonicalSchema, Feature, super::rule::CompanionRule>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("canonical-companion.program.v2");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <changed::Program as ApplicationProgramDefinition<CanonicalSchema>>::feature_specs()
    }
}

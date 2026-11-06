#![forbid(unsafe_code)]
mod contribution;
mod parameter_invariant;
pub use parameter_invariant::{parameter_count_invariant, PositiveParameterCount};

use worth_query_consumer_values::PositiveCount;
use worth_query_decl::facade::{application_schema::ApplicationSchema, worth_query_value_binding};

pub trait ParameterSchemaBinding: ApplicationSchema {}

pub struct ParameterFeature;

impl<Schema: ParameterSchemaBinding>
    worth_query_decl::facade::application_program::ApplicationFeature<Schema> for ParameterFeature
{
    type Inputs = worth_query_decl::facade::application_program::ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.certification.parameter-feature.v1";
}

worth_query_value_binding! {
    pub ParameterCountBinding for PositiveCount {
        identity: "worth.query.certification.parameter-count.v1",
        scalar: UInt64,
        unit: "CountUnit",
        encode: PositiveCount::get,
        decode: PositiveCount::new,
    }
}

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct ParameterModel;
impl Fragment for ParameterModel {
    const NAME: &'static str = "parameter";
    type Bindings = ParameterBindings;
    type Members = (Parameter,);
}
pub struct ParameterBindings;
impl BindingSet for ParameterBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<PositiveCount> for ParameterBindings {
    type Binding = ParameterCountBinding;
    const EQUALITY_QUERYABLE: bool = true;
}
pub struct Parameter;
impl Entity for Parameter {
    const NAME: &'static str = "Parameter";
    type Aspects = (ParameterValue,);
    type Relations = ();
}
impl Parameter {
    pub fn reference<S: ParameterSchemaBinding>() -> ApplicationEntityRef<S, Self> {
        fragment::entity(ParameterModel, Self)
    }
}
pub struct ParameterValue;
impl Aspect for ParameterValue {
    const NAME: &'static str = "ParameterValue";
    type Row = PositiveCount;
}
impl ParameterValue {
    pub fn reference<S: ParameterSchemaBinding>() -> AspectSelection<ParameterModel, Parameter, Self>
    {
        fragment::aspect(ParameterModel, Parameter, Self)
    }
}
pub type Count = FieldSelection<ParameterModel, Parameter, ParameterValue, PositiveCount>;
pub struct ParameterContribution;
impl<S: ParameterSchemaBinding> ApplicationSchemaContribution<S> for ParameterContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.certification.parameter.v1");
    type Members = (ParameterModel, ParameterContracts);
}
impl ParameterContribution {
    pub fn reference<S: ParameterSchemaBinding>() -> ApplicationSchemaContributionRef<S, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct ParameterContracts;
impl<S: ParameterSchemaBinding> Registration<S> for ParameterContracts {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<S>, ApplicationSchemaDeclarationDenial> {
        Ok(builder.invariant(parameter_count_invariant::<S>()))
    }
}

//! Program selection binds both courtroom demands to declared output roots.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationConnectionIdentity, ApplicationFeature, ApplicationFeatureInputLeaf,
    ApplicationFeatureInputList, ApplicationInputPort, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationRequiredOutputConnection, WorthQueryRequiredOutputConnectionDenial,
};

pub(in super::super) struct RootSource;
pub(in super::super) struct DependentSource;
macro_rules! connection {
    ($name:ident, $feature:ty, $identity:literal, $demand:ty, $value:expr) => {
        impl ApplicationConnectionIdentity for $name {
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationOccurrenceConnectionBinding<Schema, PlanarSourceFeature, $feature>
            for $name
        {
        }
        impl WorthQueryApplicationRequiredOutputConnection<Schema> for $name {
            type Source = PlanarSourceAdjustmentBinding<Schema>;
            type Demand = $demand;
            const IDENTITY: &'static str = $identity;
            fn demand_from_source(
                _: &PlanarSourceAdjustment,
            ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
                Ok($value)
            }
        }
    };
}
connection!(
    RootSource,
    PlanarOutputFeature,
    "courtroom-equal-root-connection",
    Demand,
    Demand(OUTPUT)
);
connection!(
    DependentSource,
    DependentFeature,
    "courtroom-equal-dependent-connection",
    counted_producer::Demand,
    counted_producer::Demand
);
pub(in super::super) type Root = ApplicationOutputGraph<
    ApplicationConnectionRef<
        Schema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        PlanarOutputFeature,
        PlanarBodyInput,
        RootSource,
    >,
    ApplicationOutputLeaf,
>;
pub(in super::super) type DependentRoot = ApplicationOutputGraph<
    ApplicationConnectionRef<
        Schema,
        PlanarSourceFeature,
        PlanarBodyOutput,
        DependentFeature,
        DependentInput,
        DependentSource,
    >,
    ApplicationOutputLeaf,
>;

pub(in super::super) struct DependentFeature;
pub(in super::super) struct DependentInput;
pub(in super::super) struct DependentOutput;
impl ApplicationFeature<Schema> for DependentFeature {
    type Inputs = ApplicationFeatureInputList<DependentInput, ApplicationFeatureInputLeaf>;
    const IDENTITY: &'static str = "courtroom-dependent-feature";
}
impl ApplicationInputPort<Schema, DependentFeature> for DependentInput {
    type Value = PlanarReadResultBinding;
    const IDENTITY: &'static str = "source";
    const REQUIRED: bool = true;
}
impl ApplicationOutputPort<Schema, DependentFeature> for DependentOutput {
    type Value = PlanarOutputReadResultBinding;
    const IDENTITY: &'static str = "output";
}

impl
    worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputSource<
        Schema,
        RootSource,
    > for PlanarSourceAdjustmentBinding<Schema>
{
    fn demand_from_source(
        _: &PlanarSourceAdjustment,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(OUTPUT))
    }
}

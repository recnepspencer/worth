//! One actual parent discovers a healthy or stale child from the installed meaning.
use super::*;
pub struct Start<const CHILD_B: bool>;
impl<const B: bool> ApplicationConnectionIdentity for Start<B> {
    const IDENTITY: &'static str = if B {
        "owned-outputs.healthy.start-b"
    } else {
        "owned-outputs.healthy.start-a"
    };
}
impl<const B: bool>
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::super::program::Entry,
        hierarchy::Parent,
    > for Start<B>
{
}
pub type Connection<const B: bool> = ApplicationConnectionRef<
    Schema,
    super::super::super::super::program::Entry,
    super::super::super::super::program::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Start<B>,
>;
impl<const B: bool> WorthQueryApplicationRequiredOutputConnection<Schema> for Start<B> {
    type Source = edit::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("a".into()))
    }
}
impl<const B: bool> WorthQueryApplicationRequiredOutputSource<Schema, Start<B>> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start<B> as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}
pub struct Discover<const CHILD_B: bool>;
impl<const B: bool> ApplicationConnectionIdentity for Discover<B> {
    const IDENTITY: &'static str = if B {
        "owned-outputs.healthy.discover-b"
    } else {
        "owned-outputs.healthy.discover-a"
    };
}
impl<const B: bool>
    ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for Discover<B>
{
}
pub type ChildConnection<const B: bool> = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    Discover<B>,
>;
impl<const B: bool> WorthQueryApplicationDependentOutputConnection<Schema> for Discover<B> {
    type RootDemand = first::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        assert_eq!(row.key, "first:a");
        assert_eq!(row.value, 30);
        Ok(vec![second::Demand(if B { "b" } else { "a" }.into())])
    }
}
pub type Tree<const B: bool> = ApplicationOutputGraph<
    Connection<B>,
    ApplicationOutputEdge<ChildConnection<B>, ApplicationOutputLeaf>,
>;
pub struct Delivery<const CHILD_B: bool>;
impl<const B: bool> ApplicationActionDelivery<Schema, edit::Binding> for Delivery<B> {
    type Mode = ApplicationFixedDeliveryMode<Tree<B>, ApplicationNoProjection>;
    const IDENTITY: &'static str = if B {
        "owned-outputs.healthy.delivery-b"
    } else {
        "owned-outputs.healthy.delivery-a"
    };
}
impl<const B: bool> ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery<B> {
    type Root = Tree<B>;
    type Projection = ApplicationNoProjection;
}
pub struct Program<const CHILD_B: bool>;
impl<const B: bool> ApplicationProgramDefinition<Schema> for Program<B> {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree<B>>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new(if B {
        "owned-outputs.healthy.program-b"
    } else {
        "owned-outputs.healthy.program-a"
    });
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery<B>>();
        specs
    }
}

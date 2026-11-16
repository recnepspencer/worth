//! The same world retires m while its required read names independent Model b.
use super::super::super::{authored_member, hierarchy::program as features, program as entry};
use super::*;

pub struct ParentRead;
impl ApplicationConnectionIdentity for ParentRead {
    const IDENTITY: &'static str = "interleaved.retirement-independent-parent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, features::Parent> for ParentRead {}
type Connection = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    features::Parent,
    features::ContinuationInput,
    ParentRead,
>;
pub type Tree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for ParentRead {
    type Source = authored_member::retire::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &authored_member::retire::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("b".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, ParentRead>
    for authored_member::retire::Binding
{
    fn demand_from_source(
        input: &authored_member::retire::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <ParentRead as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(
        Tree,
        (
            super::program::ModelTree,
            (
                super::program::ProducedTree,
                super::program::OwnedMemberTree,
            ),
        ),
    )>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.retirement-independent-parent");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::program::feature_specs::<fixture::Delivery<Tree>>()
    }
}

//! A stored captured reader reached from a supplier's tree, not as its own root.
//!
//! Every model edit roots `first:carriage:late`. Its captured-input occurrence
//! connections reach the reader `captured:capture:child`, and the generator
//! connection owes `first:carriage:d`, which the reader captured as a parent.
//! Once the reader is stored, only its own captured parents can order it after
//! that later route (#254). Program connections are unique, so the reader is
//! born through this same tree rather than its own root.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::super::super::{hierarchy::program as hierarchy, program as entry};
use super::*;

pub struct Supplier;
impl ApplicationConnectionIdentity for Supplier {
    const IDENTITY: &'static str = "captured-input.supplier";
}
impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent> for Supplier {}
pub type SupplierConnection = ApplicationConnectionRef<
    Schema,
    entry::Entry,
    entry::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Supplier,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Supplier {
    type Source = fixture::model::Binding;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &fixture::model::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("carriage:late".into()))
    }
}
/// A member edit roots the owed child itself, storing it before any reader
/// that does not capture it is reached.
impl WorthQueryApplicationRequiredOutputSource<Schema, Supplier> for fixture::member::Binding {
    fn demand_from_source(
        _: &fixture::member::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand("carriage:d".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Supplier> for fixture::model::Binding {
    fn demand_from_source(
        input: &fixture::model::Intent,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Supplier as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}
/// The supplier's generated child: `first:carriage:late` owes `first:carriage:d`.
pub struct Generated;
impl ApplicationConnectionIdentity for Generated {
    const IDENTITY: &'static str = "captured-input.generated";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
    for Generated
{
}
type GeneratedConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::ForeignInput,
    Generated,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Generated {
    type RootDemand = first::Demand;
    type Demand = first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(match row.key.as_str() {
            "first:carriage:late" => vec![first::Demand("carriage:d".into())],
            _ => Vec::new(),
        })
    }
}
/// The generated child's strict prerequisite suppliers are the members of
/// model `carriage:d`, as `borrowed_inputs` reads them; it has none, so only
/// the occurrence orders `first:carriage:d` after `first:carriage:late`.
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Generated {
    type Prerequisites = query::members::Read;
    type PrerequisiteRow = query::RowBinding;
    type Selection = ApplicationQueryResultRelationRef<
        query::members::Query,
        query::members::MembersSlot,
        Schema,
        ModelMember,
        Model,
        Member,
        ForwardResultTraversal,
        ManyResults,
    >;
    fn prerequisite_rows() -> Self::Selection {
        query::members::members()
    }
    fn prerequisites_from_target(
        _: &first::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read("carriage:d".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
/// The reader's captured connections and the generator overlap on
/// `hierarchy::Parent`, so they form one occurrence component; a same-feature
/// child edge is an occurrence, never a static cycle.
pub struct Generator;
impl ApplicationOccurrenceComponent<Schema> for Generator {
    type Connections = (
        ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
        ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
        GeneratedConnection,
    );
}
pub type SupplierTree =
    ApplicationOutputGraph<SupplierConnection, ApplicationOutputOccurrences<Generator>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<SupplierTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.supplier");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<SupplierTree>>()
    }
}

//! `W` independent chains of length `L` in one edit (#254, wave cost).
//!
//! Every model edit roots `first:carriage:late` through the supplier. Its
//! discovery reveals the chain heads `first:chain:<w>:0`, and each link
//! `first:chain:<w>:<l>` reveals `first:chain:<w>:<l+1>`. No link has a
//! prerequisite, so each level is one ranked batch.
use super::super::super::hierarchy::program as hierarchy;
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use std::cell::Cell;

thread_local! {
    static WIDTH: Cell<usize> = const { Cell::new(0) };
    static LENGTH: Cell<usize> = const { Cell::new(0) };
}

pub struct Link;
impl ApplicationConnectionIdentity for Link {
    const IDENTITY: &'static str = "captured-input.chains.link";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent> for Link {}
type LinkConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::ForeignInput,
    Link,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Link {
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
        let (width, length) = (WIDTH.with(Cell::get), LENGTH.with(Cell::get));
        if row.key == "first:carriage:late" {
            return Ok((0..width).map(|w| first::Demand(key(w, 0))).collect());
        }
        let next = row
            .key
            .strip_prefix("first:chain:")
            .and_then(|rest| rest.split_once(':'))
            .and_then(|(w, l)| Some((w.parse::<usize>().ok()?, l.parse::<usize>().ok()? + 1)))
            .filter(|(_, l)| *l < length);
        Ok(next
            .map(|(w, l)| first::Demand(key(w, l)))
            .into_iter()
            .collect())
    }
}
/// Links read the members of the memberless model `chains:none`.
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Link {
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
        Ok(query::members::Read("chains:none".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
pub struct Chains;
impl ApplicationOccurrenceComponent<Schema> for Chains {
    type Connections = (
        ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
        ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
        LinkConnection,
    );
}
pub type Tree =
    ApplicationOutputGraph<supplier::SupplierConnection, ApplicationOutputOccurrences<Chains>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.chains");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<Tree>>()
    }
}
pub fn key(w: usize, l: usize) -> String {
    format!("chain:{w}:{l}")
}
/// Seed `width` chains of `length` models; the edit root and the empty model.
pub(in super::super::super) fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    width: usize,
    length: usize,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    WIDTH.with(|cell| cell.set(width));
    LENGTH.with(|cell| cell.set(length));
    let links =
        (0..width).flat_map(|w| (0..length).map(move |l| (key(w, l), (w * length + l) as u64)));
    let fixed = [
        ("carriage:late".to_owned(), 5),
        ("carriage:d".to_owned(), 4),
        ("chains:none".to_owned(), 0),
    ];
    for (key, value) in fixed.into_iter().chain(links) {
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(key.as_str()).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key.clone())
            .field(ModelValue::at("ModelValue"), value),
        )?;
    }
    Ok(())
}

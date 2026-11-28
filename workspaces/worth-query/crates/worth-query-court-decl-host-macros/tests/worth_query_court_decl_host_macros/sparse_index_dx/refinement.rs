//! A conservative index with case-sensitive row-local title refinement.
use super::schema::*;
use super::{add, ids, open, owner, titled, SparseHome};
use std::convert::Infallible;
use worth_query_decl::facade::{
    application_query::{ApplicationManagedIndexMembership, ApplicationManagedIndexRefinement},
    WorthManagedIndex, WorthQuery,
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    ordered_index::WorthQueryOrderedKeyField,
    primary_graph::{
        WorthQueryManagedIndexControl as Control, WorthQueryManagedIndexNodeId as NodeId,
        WorthQueryManagedIndexNodeRegion as Region, WorthQueryManagedIndexNodeWriter as Nodes,
        WorthQueryManagedIndexProvider, WorthQueryManagedIndexProviderStop,
        WorthQueryManagedIndexRefinementProvider, WorthQueryManagedIndexSourceRow,
        WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot,
    },
};
type Stop = WorthQueryManagedIndexProviderStop<Infallible>;
type Row<'a> = WorthQueryManagedIndexSourceRow<'a, SparseSchema, RefinedTitleIndex>;
type Rows<'a> = WorthQueryManagedIndexSourceRows<'a, SparseSchema, RefinedTitleIndex>;

#[derive(WorthManagedIndex)]
#[worth_index(
    schema = SparseSchema, record = Task, source = ReadInboxBinding, probe = String,
    kind = custom(TitleProvider), provider = "sparse.dx.exact_title",
    membership = ApplicationManagedIndexMembership::ExhaustiveSourceTree,
    maintenance = WhenRequired,
    refinement = Some(ApplicationManagedIndexRefinement::row_local("sparse.dx.title_case", 1)),
)]
pub struct RefinedTitleIndex;

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = SparseSchema, context = Desktop, namespace = SPARSE, member,
    scope = InboxId, managed_index = RefinedTitleIndex, managed_index_refined, lanes = OneShot,
    order_by(TaskView::id(), Ascending), limits = 64,
)]
pub struct FindExactTitle {
    #[worth_query(scope)]
    pub inbox: String,
    #[worth_query(probe)]
    pub prefix: String,
}

struct TitleProvider;

impl WorthQueryManagedIndexProvider<SparseSchema, RefinedTitleIndex> for TitleProvider {
    type Node = (WorthQueryManagedIndexSourceSlot, Option<NodeId>);
    type Summary = ();
    type Denial = Infallible;

    const IDENTITY: &'static str = "sparse.dx.exact_title";
    const VERSION: u32 = 1;

    fn node_capacity(&self, source_count: usize, control: &Control<'_>) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(source_count)
    }

    fn build(
        &self,
        rows: &Rows<'_>,
        nodes: &mut Nodes<'_, Self::Node>,
        control: &Control<'_>,
    ) -> Result<Option<NodeId>, Stop> {
        let mut root = None;
        for slot in rows.slots() {
            control.checkpoint(1)?;
            root = Some(nodes.push((slot?, root))?);
        }
        Ok(root)
    }

    fn decompose(&self, node: &Self::Node, control: &Control<'_>) -> Result<Region, Stop> {
        control.checkpoint(1)?;
        Ok(Region::new(Some(node.0), [node.1, None]))
    }

    fn matches_source(&self, _: Row<'_>, _: &String, control: &Control<'_>) -> Result<bool, Stop> {
        control.checkpoint(1)?;
        Ok(true)
    }
}

impl WorthQueryManagedIndexRefinementProvider<SparseSchema, RefinedTitleIndex> for TitleProvider {
    const REFINEMENT_IDENTITY: &'static str = "sparse.dx.title_case";
    const REFINEMENT_VERSION: u32 = 1;

    fn validate_refinement_probe(&self, _: &String, control: &Control<'_>) -> Result<(), Stop> {
        control.checkpoint(1).map_err(Into::into)
    }

    fn refine_source(
        &self,
        row: Row<'_>,
        prefix: &String,
        control: &Control<'_>,
    ) -> Result<bool, Stop> {
        let title = TaskView::title().read_key::<Infallible>(&row, control)?;
        control.checkpoint(prefix.len() as u64)?;
        Ok(title.is_some_and(|title| title.starts_with(prefix)))
    }
}

#[test]
fn derived_registration_returns_exact_hits_and_rejects_coarse_hits() {
    let path = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("refined-index-{}", std::process::id()));
    let clock = WorthQueryAdmissionClock::system();
    let home: SparseHome = open(&path, &clock);
    super::ensure_ready(&home, &clock);
    let session = home.session(owner(&home, &clock));
    session.run(add(1, Some("Alpha"), (None, None))).unwrap();
    session.run(add(2, Some("alpha"), (None, None))).unwrap();
    assert_eq!(titled(&home, &clock, "Al"), [1, 2]);

    let probe = || FindExactTitle {
        inbox: INBOX.to_owned(),
        prefix: "Al".into(),
    };
    session.ensure_index_ready(probe()).unwrap();
    assert_eq!(ids(&session.query(probe()).unwrap()), [1]);

    home.close_clean().unwrap();
    std::fs::remove_dir_all(path).unwrap();
}

//! Shape 3 (r3 item 2b): same-rank `[P, U, X, Z]` batch-prefix split at `Recapture`:
//! `capture:x` recaptures and cuts the batch before its trailing sibling `capture:z`.
use super::*;

pub struct SplitParent;
impl ApplicationConnectionIdentity for SplitParent {
    const IDENTITY: &'static str = "captured-input.r3.split-parent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
    for SplitParent
{
}
type SplitParentConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Parent,
    hierarchy::ContinuationInput,
    SplitParent,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for SplitParent {
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
            "first:carriage:late" => vec![
                first::Demand("split:a".into()),
                first::Demand("split:b".into()),
            ],
            "first:split:a" => vec![first::Demand("carriage:d".into())],
            _ => Vec::new(),
        })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for SplitParent {
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
        Ok(query::members::Read("composed:none".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(first::Demand(row.key.clone()))
    }
}
pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (
        ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
        ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
        SplitParentConnection,
    );
}
pub type Tree =
    ApplicationOutputGraph<supplier::SupplierConnection, ApplicationOutputOccurrences<Component>>;
/// Second root into `program::Feature` gives Parent and Feature the SAME static rank (0).
pub type CapturedRootTree =
    ApplicationOutputGraph<program::Root, ApplicationOutputOccurrences<Component>>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Tree, CapturedRootTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.r3.batch-split");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<Tree>>()
    }
}

pub fn build_host() -> WorthQueryProgramApplicationRuntime<Schema, Program> {
    install_custom(|graph| {
        for (key, value) in [
            ("carriage:late", 5),
            ("carriage:d", 4),
            ("split:a", 2),
            ("split:b", 3),
            ("capture:p", 2),
            ("capture:x", 3),
            ("capture:z", 4),
            ("composed:none", 0),
        ] {
            bind_model(graph, key, value)?;
        }
        // split:a discovers P then U (`carriage:d`); split:b discovers X then Z
        // so X sorts BEFORE Z and cuts the same-rank batch mid-flight on Recapture.
        bind_member(graph, "capture:p", "split:p-a", "split:a", 1)?;
        bind_member(graph, "capture:x", "split:x-b", "split:b", 1)?;
        bind_member(graph, "capture:x", "split:x-d", "", handler::UNDECLARED)?;
        bind_member(graph, "capture:z", "split:z-b", "split:b", 1)
    })
}

pub fn run(trace: &mut Trace) {
    let host = build_host();
    trace.0 += "== r3 shape 3: batch-prefix split at Recapture\n";
    let born = edit_stage!(&host, Tree, "r3-split-born", "carriage:d", 4);
    assert_eq!(born.0, "committed", "shape 3 born must commit: {born:?}");
    trace.edit("born", born);
    let changed = edit_stage!(&host, Tree, "r3-split-changed", "carriage:d", 5);
    let cut = changed
        .1
        .iter()
        .position(|e| e == "break recapture at=6")
        .unwrap_or_else(|| {
            panic!(
                "shape 3 must cut mid-batch at task=6 before trailing sibling task=7: {:?}",
                changed.1
            )
        });
    assert!(
        changed.1[cut + 1..]
            .iter()
            .any(|e| e == "batch id=3 rank=0 ready=[6, 7]"),
        "trailing sibling task=7 must defer to the next batch with recaptured task=6: {:?}",
        changed.1
    );
    trace.edit("changed", changed);
    trace.rows(
        &host,
        &[
            "first:carriage:d",
            "captured:capture:p",
            "captured:capture:x",
            "captured:capture:z",
        ],
    );
}

//! Shape 2 (r3 item 2a): reverse-rank consumer across batches.
use super::*;

pub struct WaitW;
impl ApplicationConnectionIdentity for WaitW {
    const IDENTITY: &'static str = "captured-input.r3.wait-w";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::Feature, hierarchy::Parent> for WaitW {}
type WaitWConnection = ApplicationConnectionRef<
    Schema,
    program::Feature,
    program::Output,
    hierarchy::Parent,
    hierarchy::PredecessorInput,
    WaitW,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for WaitW {
    type RootDemand = Demand;
    type Demand = first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("captured:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "captured:capture:r" {
            vec![first::Demand("carriage:d".into())]
        } else {
            Vec::new()
        })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for WaitW {
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
        target: &first::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read(if target.0 == "carriage:d" {
            "waits:carriage:d".into()
        } else {
            "composed:none".into()
        }))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(row.key.trim_start_matches("wait:").to_owned()))
    }
}

/// Higher-rank consumer C (`from_first`, rank 2) waiting on P (`capture:p`, rank 0).
pub struct HighC;
impl ApplicationConnectionIdentity for HighC {
    const IDENTITY: &'static str = "captured-input.r3.high-c";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::Feature, hierarchy::Child> for HighC {}
type HighCConnection = ApplicationConnectionRef<
    Schema,
    program::Feature,
    program::Output,
    hierarchy::Child,
    hierarchy::ChildInput,
    HighC,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for HighC {
    type RootDemand = Demand;
    type Demand = from_first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("captured:{}", root.0)))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<from_first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.key == "captured:capture:p" {
            vec![from_first::Demand("first:carriage:d".into())]
        } else {
            Vec::new()
        })
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for HighC {
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
        _: &from_first::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read("waits:high-c".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(row.key.trim_start_matches("wait:").to_owned()))
    }
}

/// Static Parent -> Child edge giving `hierarchy::Child` numeric rank 2 > Parent (1) > Feature (0).
pub struct StaticRank;
impl ApplicationConnectionIdentity for StaticRank {
    const IDENTITY: &'static str = "captured-input.r3.static-rank";
}
impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Child>
    for StaticRank
{
}
type StaticRankConnection = ApplicationConnectionRef<
    Schema,
    hierarchy::Parent,
    hierarchy::Output,
    hierarchy::Child,
    hierarchy::SeedInput,
    StaticRank,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for StaticRank {
    type RootDemand = first::Demand;
    type Demand = from_first::Demand;
    type Discovery = query::first_output::Read;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        root: &first::Demand,
    ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read(format!("first:{}", root.0)))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<from_first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Vec::new())
    }
}

pub struct Component;
impl ApplicationOccurrenceComponent<Schema> for Component {
    type Connections = (
        ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
        ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
        ApplicationConnectionRef<
            Schema,
            hierarchy::Parent,
            hierarchy::Output,
            hierarchy::Parent,
            hierarchy::ContinuationInput,
            composed::Sibling,
        >,
        WaitWConnection,
        HighCConnection,
    );
}
pub type Tree = ApplicationOutputGraph<
    supplier::SupplierConnection,
    (
        ApplicationOutputOccurrences<Component>,
        ApplicationOutputEdge<StaticRankConnection, ApplicationOutputLeaf>,
    ),
>;
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.captured-input.r3.reverse-rank");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        program::feature_specs::<fixture::Delivery<Tree>>()
    }
}

pub fn run(trace: &mut Trace) {
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = install_custom(|graph| {
        for (key, value) in [
            ("carriage:late", 5),
            ("carriage:d", 4),
            ("capture:s", 6),
            ("capture:p", 2),
            ("capture:r", 3),
            ("composed:none", 0),
            ("waits:carriage:d", 0),
            ("waits:high-c", 0),
        ] {
            bind_model(graph, key, value)?;
        }
        // R reads only `capture:s` (no undeclared read of W); C reads W (`first:carriage:d`).
        bind_member(graph, "capture:p", "capture:p-late", "carriage:late", 1)?;
        bind_member(graph, "capture:r", "capture:r-s", "capture:s", 1)?;
        bind_member(graph, "waits:high-c", "wait:capture:p", "", 1)?;
        bind_member(graph, "waits:carriage:d", "wait:capture:r", "", 1)
    });
    trace.0 += "== r3 shape 2: reverse-rank consumer across batches\n";
    let seed_d = {
        let obs = WorthQueryExecutionObservation::install_on_this_thread();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut edit = request
            .edit_envelope()
            .idempotency("r3-rev-seed-d".into())
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<Tree, _>(fixture::member::Intent {
            key: "capture:p-late".into(),
            value: 1,
            key_derived_target: false,
        })
        .unwrap();
        summarize_outcome(
            edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap(),
            obs,
        )
    };
    assert_eq!(seed_d.0, "committed", "seed_d must commit: {seed_d:?}");
    trace.edit("seed-d", seed_d);
    let born = edit_stage!(&host, Tree, "r3-rev-born", "carriage:d", 4);
    assert_eq!(born.0, "committed", "born must commit: {born:?}");
    trace.edit("born", born);
    let changed = edit_stage!(&host, Tree, "r3-rev-changed", "carriage:d", 5);
    assert_eq!(changed.0, "committed", "changed must commit: {changed:?}");
    trace.edit("changed", changed);
    assert_eq!(child_value(&host, "from-first:first:carriage:d"), "5000");
    trace.rows(&host, &["first:carriage:d"]);
    trace.0 += &format!(
        "   from-first:first:carriage:d = {}\n",
        child_value(&host, "from-first:first:carriage:d")
    );
}

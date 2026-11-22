//! Named PR B base-order shapes from #256 r3:
//! 1. Late outgoing edge (r3 item 1): base refuses `CoupledReadSupportChanged` (hold None).
//! 2. Reverse-rank consumer across batches (r3 item 2a): base runs W before higher-rank C.
//! 3. Batch-prefix split at `Recapture` (r3 item 2b): `[P, U, X, R]` cuts at X before R.
use super::super::super::super::hierarchy::program as hierarchy;
use super::super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::super::*;
use super::{child_value, summarize_outcome, value, Trace};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

pub(super) mod batch_split;
mod reverse_rank;

fn install_custom<P>(
    seed_extra: impl FnOnce(
        &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    let schema = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, P>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (world::Configuration {
            envelope_records: true,
            ..Default::default()
        },),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 1, None)?;
            fixture::seed(graph)?;
            seed_extra(graph)
        },
    )
    .unwrap()
}

fn bind_model(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    key: &str,
    value: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Model::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(ModelKey::at("ModelKey"), key.to_owned())
        .field(ModelValue::at("ModelValue"), value),
    )
}
fn bind_member(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    owner: &str,
    key: &str,
    parent: &str,
    weight: u64,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Member::reference(),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
        )
        .field(MemberKey::at("MemberKey"), key.to_owned())
        .field(MemberValue::at("MemberValue"), weight),
    )?;
    graph.bind_relation(WorthQueryApplicationRelationSeed::new(
        ModelMember::reference(),
        format!("owns:{key}"),
        WorthQueryApplicationEntityKey::new(owner).unwrap(),
        WorthQueryApplicationEntityKey::new(key).unwrap(),
    ))?;
    if !parent.is_empty() {
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            UpstreamOf::reference(),
            format!("uses:{key}"),
            WorthQueryApplicationEntityKey::new(key).unwrap(),
            WorthQueryApplicationEntityKey::new(parent).unwrap(),
        ))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shape 1: Late outgoing edge (r3 item 1)
// ---------------------------------------------------------------------------
mod late_edge {
    use super::super::super::super::super::program as entry;
    use super::*;
    pub struct DirectFirst;
    impl ApplicationConnectionIdentity for DirectFirst {
        const IDENTITY: &'static str = "captured-input.r3.direct-first";
    }
    impl ApplicationOccurrenceConnectionBinding<Schema, entry::Entry, hierarchy::Parent>
        for DirectFirst
    {
    }
    type DirectFirstConnection = ApplicationConnectionRef<
        Schema,
        entry::Entry,
        entry::EntryOutput,
        hierarchy::Parent,
        hierarchy::FirstSeedInput,
        DirectFirst,
    >;
    impl WorthQueryApplicationRequiredOutputConnection<Schema> for DirectFirst {
        type Source = fixture::model::Binding;
        type Demand = first::Demand;
        const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
        fn demand_from_source(
            input: &fixture::model::Intent,
        ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
            Ok(first::Demand(input.key.clone()))
        }
    }
    impl WorthQueryApplicationRequiredOutputSource<Schema, DirectFirst> for fixture::model::Binding {
        fn demand_from_source(
            input: &fixture::model::Intent,
        ) -> Result<first::Demand, WorthQueryRequiredOutputConnectionDenial> {
            Ok(first::Demand(input.key.clone()))
        }
    }
    type SeedFirstTree = ApplicationOutputGraph<DirectFirstConnection, ApplicationOutputLeaf>;

    /// Two-hop sibling chain (`carriage:late -> capture:bridge -> capture:s`)
    /// so `capture:p` runs in batch 1 before `first:capture:s` updates in batch 2.
    pub struct BridgeSibling;
    impl ApplicationConnectionIdentity for BridgeSibling {
        const IDENTITY: &'static str = "captured-input.r3.bridge-sibling";
    }
    impl ApplicationOccurrenceConnectionBinding<Schema, hierarchy::Parent, hierarchy::Parent>
        for BridgeSibling
    {
    }
    type BridgeSiblingConnection = ApplicationConnectionRef<
        Schema,
        hierarchy::Parent,
        hierarchy::Output,
        hierarchy::Parent,
        hierarchy::ContinuationInput,
        BridgeSibling,
    >;
    impl WorthQueryApplicationDependentOutputConnection<Schema> for BridgeSibling {
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
                "first:carriage:late" => vec![first::Demand("capture:bridge".into())],
                "first:capture:bridge" => vec![first::Demand("capture:s".into())],
                _ => Vec::new(),
            })
        }
    }
    impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for BridgeSibling {
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

    pub struct LateOwed;
    impl ApplicationConnectionIdentity for LateOwed {
        const IDENTITY: &'static str = "captured-input.r3.late-owed";
    }
    impl ApplicationOccurrenceConnectionBinding<Schema, program::Feature, hierarchy::Parent>
        for LateOwed
    {
    }
    type LateOwedConnection = ApplicationConnectionRef<
        Schema,
        program::Feature,
        program::Output,
        hierarchy::Parent,
        hierarchy::PredecessorInput,
        LateOwed,
    >;
    impl WorthQueryApplicationDependentOutputConnection<Schema> for LateOwed {
        type RootDemand = Demand;
        type Demand = first::Demand;
        type Discovery = query::first_output::Read;
        const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
        fn discovery_from_root(
            root: &Demand,
        ) -> Result<query::first_output::Read, WorthQueryRequiredOutputConnectionDenial> {
            let key = if root.0 == "capture:p" {
                "first:capture:s"
            } else {
                "first:carriage:late"
            };
            Ok(query::first_output::Read(key.into()))
        }
        fn demands_from_discovery(
            row: &query::Row,
        ) -> Result<Vec<first::Demand>, WorthQueryRequiredOutputConnectionDenial> {
            // Links Q (`carriage:d`) only once S's branch writes 70.
            Ok(if row.key == "first:capture:s" && row.value == 70 {
                vec![first::Demand("carriage:d".into())]
            } else {
                Vec::new()
            })
        }
    }
    impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for LateOwed {
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
        ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
            Ok(Demand(row.key.clone()))
        }
    }
    pub struct Component;
    impl ApplicationOccurrenceComponent<Schema> for Component {
        type Connections = (
            ApplicationCapturedInputConnection<program::SeedConnection, Capture>,
            ApplicationCapturedInputConnection<program::AnchorConnection, Capture>,
            BridgeSiblingConnection,
            LateOwedConnection,
        );
    }
    pub type Tree = ApplicationOutputGraph<
        supplier::SupplierConnection,
        ApplicationOutputOccurrences<Component>,
    >;
    pub struct Program;
    impl ApplicationProgramDefinition<Schema> for Program {
        type Contributions = (Contribution,);
        type Outputs = ApplicationProgramOutputs<(Tree, SeedFirstTree)>;
        type Rules = ApplicationRuleLeaf;
        const IDENTITY: ApplicationProgramIdentity =
            ApplicationProgramIdentity::new("owned-outputs.captured-input.r3.late-edge");
        fn feature_specs() -> Vec<ApplicationFeatureSpec> {
            program::feature_specs::<fixture::Delivery<Tree>>()
        }
    }
    pub fn run(trace: &mut Trace) {
        let host: WorthQueryProgramApplicationRuntime<Schema, Program> = install_custom(|graph| {
            composed::seed(graph, 0, false)?;
            bind_model(graph, "capture:bridge", 1)
        });
        trace.0 += "== r3 shape 1: late outgoing edge\n";
        let seed_d = edit_stage!(&host, SeedFirstTree, "r3-late-seed-d", "carriage:d", 4);
        assert_eq!(seed_d.0, "committed", "seed_d must commit: {seed_d:?}");
        trace.edit("seed-d", seed_d);
        let seed_s = edit_stage!(&host, SeedFirstTree, "r3-late-seed-s", "capture:s", 6);
        assert_eq!(seed_s.0, "committed", "seed_s must commit: {seed_s:?}");
        trace.edit("seed-s", seed_s);
        let born = edit_stage!(&host, Tree, "r3-late-born", "carriage:late", 5);
        assert_eq!(born.0, "committed", "born must commit: {born:?}");
        trace.edit("born", born);
        let late = {
            let obs = WorthQueryExecutionObservation::install_on_this_thread();
            let scope = request_scope();
            let principal = authenticate_operator(host.installed_schema(), &scope);
            let request = host.request(&principal, &scope);
            let mut edit = request
                .edit_envelope()
                .idempotency("r3-late-change".into())
                .runtime_stages(&host)
                .ordered_authored()
                .unwrap();
            edit.push_stage::<Tree, _>(fixture::model::Intent {
                key: "carriage:d".into(),
                value: 5,
                key_derived_target: false,
            })
            .unwrap();
            edit.push_stage::<Tree, _>(fixture::model::Intent {
                key: "capture:s".into(),
                value: 7,
                key_derived_target: false,
            })
            .unwrap();
            let outcome = edit
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            summarize_outcome(outcome, obs)
        };
        // Qualified use reopens what read the old suppliers, and the edit
        // commits exactly the world one commit per stage commits.
        assert_eq!(late.0, "committed", "late outgoing edge: {late:?}");
        trace.edit("late-change", late);
        trace.rows(&host, &["first:carriage:d", "captured:capture:r"]);
        let sequential: WorthQueryProgramApplicationRuntime<Schema, Program> =
            install_custom(|graph| {
                composed::seed(graph, 0, false)?;
                bind_model(graph, "capture:bridge", 1)
            });
        for (key, model, value, tree_is_seed) in [
            ("r3-seq-seed-d", "carriage:d", 4, true),
            ("r3-seq-seed-s", "capture:s", 6, true),
            ("r3-seq-born", "carriage:late", 5, false),
            ("r3-seq-late-d", "carriage:d", 5, false),
            ("r3-seq-late-s", "capture:s", 7, false),
        ] {
            let stage = if tree_is_seed {
                edit_stage!(&sequential, SeedFirstTree, key, model, value)
            } else {
                edit_stage!(&sequential, Tree, key, model, value)
            };
            assert_eq!(stage.0, "committed", "sequential {key}: {stage:?}");
        }
        for key in ["first:carriage:d", "captured:capture:r"] {
            assert_eq!(value(&host, key), value(&sequential, key), "{key}");
        }
    }
}

pub(super) fn record_all(trace: &mut Trace) {
    late_edge::run(trace);
    reverse_rank::run(trace);
    batch_split::run(trace);
}

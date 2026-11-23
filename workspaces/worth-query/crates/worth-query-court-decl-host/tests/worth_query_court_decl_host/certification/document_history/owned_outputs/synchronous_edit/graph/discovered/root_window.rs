//! #238 (PR C C4): a root selection holds only within its own stage. One
//! commit per stage evaluates a root once, at its stage, so an authored
//! stage's append ends the window of every earlier root's selection support
//! there, and the root never selects again. (Re-running an earlier root at a
//! later state is not sequential: it can select what a later root selects
//! through another connection, `ExtraCoupledOwner`, derive a child
//! sequential never derives, or drop one it derived.)
use super::super::super::super::program as base;
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationDiscoveredActionDelivery, ApplicationDiscoveredDeliveryMode,
    ApplicationDiscoveredOutputGraph,
};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// Each edit stage's root reads model `b` and selects `first(a)` while it
/// is at least 7, nothing otherwise (a zero-output root selection).
pub struct Watch;
impl ApplicationConnectionIdentity for Watch {
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.watch-root";
}
impl ApplicationOccurrenceConnectionBinding<Schema, base::Entry, hierarchy::Parent> for Watch {}
pub type Connection = ApplicationConnectionRef<
    Schema,
    base::Entry,
    base::EntryOutput,
    hierarchy::Parent,
    hierarchy::Input,
    Watch,
>;
impl WorthQueryApplicationDiscoveredOutputConnection<Schema> for Watch {
    type Source = edit::Binding;
    type Discovery = query::model::Read;
    type Demand = first::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::model::Read("b".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(if row.value >= 7 {
            vec![first::Demand("a".into())]
        } else {
            Vec::new()
        })
    }
}
pub type Tree = ApplicationDiscoveredOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationDiscoveredDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.discovered-edit.watch-delivery";
}
impl ApplicationDiscoveredActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.discovered-edit.watch-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = base::entry_spec::<Delivery>();
        specs
    }
}

/// `first:a` after the stages, one envelope (`envelope`) or one commit per
/// stage, on a world with model `a` = 2 and `b` = `b`.
fn run(b: u64, stages: &[(&str, u64)], envelope: bool) -> (Option<u64>, Vec<String>) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(b),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let edits: Vec<&[(&str, u64)]> = if envelope {
        vec![stages]
    } else {
        stages.chunks(1).collect()
    };
    for (ordinal, stages) in edits.into_iter().enumerate() {
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("root-window:{envelope}:{ordinal}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        for (key, value) in stages {
            edit.push_stage::<Tree, _>(edit::Intent {
                key: (*key).into(),
                value: *value,
            })
            .unwrap();
        }
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(outcome, Outcome::Committed(_)),
            "{outcome:?}; {:#?}",
            _execution.wave_events_for_test()
        );
    }
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .ok()
        .and_then(|rows| rows.rows().first().map(|row| row.value));
    (first, _execution.wave_events_for_test())
}

/// (a) b = 3. Stage 0 writes a = 4; its root reads b = 3 and selects nothing
/// (a zero-output root selection). Stage 1 writes b = 8: that append changes
/// root 0's read after root 0's stage ended, so its window ends there; root
/// 1 reads b = 8 and selects first(a), derived once. The edit commits the
/// world one commit per stage builds (built first). Before C4 the changed
/// root support refused at final support (#238).
#[test]
fn a_zero_output_root_holds_within_its_stage_and_commits_the_sequential_world() {
    let stages = [("a", 4), ("b", 8)];
    let (sequential, _) = run(3, &stages, false);
    let (envelope, _) = run(3, &stages, true);
    assert!(sequential.is_some(), "stage 1's root derives first(a)");
    assert_eq!(envelope, sequential);
}

/// (c) b = 8. Stage 0 writes a = 4; its root reads b = 8 and selects first(a),
/// derived. Stage 1 writes b = 3, changing root 0's read after its stage:
/// its window ends at stage 1's append, so first(a) stands; root 1 selects
/// nothing. The edit commits the sequential world. Before C4 this refused
/// at final support (#238); a re-seeded root 0 would select nothing at the
/// later state and drop the first(a) it derived.
#[test]
fn a_root_whose_support_changes_after_its_stage_commits_the_sequential_world() {
    let stages = [("a", 4), ("b", 3)];
    let (sequential, _) = run(8, &stages, false);
    let (envelope, _) = run(8, &stages, true);
    assert!(sequential.is_some(), "stage 0's root derives first(a)");
    assert_eq!(envelope, sequential);
}

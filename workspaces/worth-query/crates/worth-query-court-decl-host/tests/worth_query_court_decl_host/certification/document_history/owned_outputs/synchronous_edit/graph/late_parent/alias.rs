//! A reopened canonical whose alias is queued for its static continuation
//! (PR C r2 blocker 2). The alias's static intent must run once the canonical
//! derived again, never be marked done while the canonical is not.
use super::selection::{seen, ObservedConnection};
use super::*;
use family::decision_input;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

/// Gives `decision-input(a)` an occurrence identity, so two roots of it
/// coalesce; it selects nothing and names no prerequisite.
pub struct Keyed;
impl ApplicationConnectionIdentity for Keyed {
    const IDENTITY: &'static str = "owned-outputs.late-parent.keyed";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, hierarchy::Child>
    for Keyed
{
}
type KeyedConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    hierarchy::Child,
    hierarchy::ChildInput,
    Keyed,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Keyed {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        _: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Vec::new())
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Keyed {
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
        _: &second::Demand,
    ) -> Result<query::members::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::members::Read("b".into()))
    }
    fn parent_from_prerequisite(
        row: &query::Row,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
pub struct KeyedComponent;
impl ApplicationOccurrenceComponent<Schema> for KeyedComponent {
    type Connections = (KeyedConnection,);
}
/// The canonical root: `decision-input(a)` with no static continuation.
pub type CanonicalTree =
    ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<KeyedComponent>>;
/// The alias root: the same route, plus a static edge that records the
/// `first:a` it discovers from.
pub type AliasTree = ApplicationOutputGraph<
    program::Connection,
    (
        ApplicationOutputOccurrences<KeyedComponent>,
        ApplicationOutputEdge<ObservedConnection, ApplicationOutputLeaf>,
    ),
>;
pub struct CanonicalDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for CanonicalDelivery {
    type Mode = ApplicationFixedDeliveryMode<CanonicalTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.canonical-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for CanonicalDelivery {
    type Root = CanonicalTree;
    type Projection = ApplicationNoProjection;
}
/// Canonical, alias and `decision-write(b)` as equal-rank roots.
pub struct AliasProgram;
impl ApplicationProgramDefinition<Schema> for AliasProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<((CanonicalTree, AliasTree), staged::WriteTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.alias");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<CanonicalDelivery>();
        specs
    }
}

/// The world with a third model `c`, so three stages enter three roots.
fn install() -> WorthQueryProgramApplicationRuntime<Schema, AliasProgram> {
    let schema = Schema::declaration().unwrap();
    in_memory_program(
        ApplicationProgramAuthoring::<Schema, AliasProgram>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        |_, graph, installed| {
            world::seed(graph, installed, 2, Some(7))?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Model::reference(),
                    WorthQueryApplicationEntityKey::new("c").unwrap(),
                )
                .field(ModelKey::at("ModelKey"), "c".to_owned())
                .field(ModelValue::at("ModelValue"), 3u64),
            )
        },
    )
    .unwrap()
}

/// The canonical `decision-input(a)` and `decision-write(b)` run in one
/// batch, the writer last; the alias waits on the canonical. The writer
/// stales the canonical after it completed and queued its alias. The alias
/// must discover only once the canonical derived again: at 11, once.
#[test]
fn a_reopened_canonicals_queued_alias_discovers_after_it_derives_again() {
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, CanonicalTree);
    seen();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-alias".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    for (key, value, stage) in [("a", 2, 0), ("c", 3, 1), ("b", 8, 2)] {
        let intent = edit::Intent {
            key: key.into(),
            value,
        };
        match stage {
            0 => edit.push_stage::<CanonicalTree, _>(intent),
            1 => edit.push_stage::<AliasTree, _>(intent),
            _ => edit.push_stage::<staged::WriteTree, _>(intent),
        }
        .unwrap();
    }
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        seen(),
        [11],
        "the alias's static continuation ran on the derived-again canonical"
    );
}

//! The late child consumes `first:a` through its admitted strict prerequisite
//! (#239, #246): `decision-write(b)` names `decision-input(a)` as its supplier,
//! so its derivation consumed `first:a`. Its change to `m` reopens `first:a`
//! and, with it, the consumer (R3): `first:a` re-derives from m = 9 (11), the
//! consumer runs again on it, writes m = 9 again and settles, and the edit
//! commits that fixed point, equal to the sequential world.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{decision_input, decision_write};

/// `decision-input(a)` to `decision-write(b)`, discovered from `first:a` as
/// `program::Discover` is, and also declaring `decision-input(a)` (the source
/// model of `first:a`) as the child's strict prerequisite supplier.
pub struct Supplied;
impl ApplicationConnectionIdentity for Supplied {
    const IDENTITY: &'static str = "owned-outputs.late-parent.supplied";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature>
    for Supplied
{
}
type SuppliedConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Supplied,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Supplied {
    type RootDemand = decision_input::Demand;
    type Discovery = query::first_output::Read;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn discovery_from_root(
        _: &Self::RootDemand,
    ) -> Result<Self::Discovery, WorthQueryRequiredOutputConnectionDenial> {
        Ok(query::first_output::Read("first:a".into()))
    }
    fn demands_from_discovery(
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        // The discovery sees the derivation it follows: first:a from m = 5
        // (2 + 5), or, once qualified use reopens it, from m = 9 (2 + 9).
        assert_eq!(row.key.as_str(), "first:a");
        assert!(matches!(row.value, 7 | 11), "{row:?}");
        Ok(vec![decision_write::Demand("b".into())])
    }
}
impl WorthQueryApplicationPrerequisiteOutputConnection<Schema> for Supplied {
    type Prerequisites = owner::Read;
    type PrerequisiteRow = owner::ParentBinding;
    type Selection = owner::Selection;
    fn prerequisite_rows() -> Self::Selection {
        owner::owner()
    }
    fn prerequisites_from_target(
        _: &decision_write::Demand,
    ) -> Result<owner::Read, WorthQueryRequiredOutputConnectionDenial> {
        Ok(owner::Read("first:a".into()))
    }
    fn parent_from_prerequisite(
        row: &owner::Parent,
    ) -> Result<decision_input::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_input::Demand(row.key.clone()))
    }
}
pub struct Supply;
impl ApplicationOccurrenceComponent<Schema> for Supply {
    type Connections = (SuppliedConnection,);
}
pub type Tree = ApplicationOutputGraph<program::Connection, ApplicationOutputOccurrences<Supply>>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.supplied-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
}
pub struct Program;
impl ApplicationProgramDefinition<Schema> for Program {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Tree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.supplied-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

fn install() -> WorthQueryProgramApplicationRuntime<Schema, Program> {
    world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    )
}

/// `first:a` is settled verified first. One authored stage (`b = 8`) enters
/// `decision-input(a)`, adopting verified `first:a`, and its child
/// `decision-write(b)`, whose strict prerequisite supplier is that verified
/// parent, changes `m`, which `first:a` read. The verified parent reopens as a
/// fresh part and its consumer reopens with it (#239); the edit commits the
/// fixed point.
#[test]
fn a_prerequisite_consumer_changing_its_verified_supplier_reaches_the_fixed_point() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, Tree);
    assert_eq!(
        super::derived::observe(&request).first,
        7,
        "settled from m = 5"
    );
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-supplied-verified".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    super::derived::committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let world = super::derived::observe(&request);
    assert_eq!(world.first, 11, "first:a re-derived from m = 9");
    assert_eq!(world, super::derived::sequential());
}

/// Nothing is settled first, so `decision-input(a)` derives `first:a` inside
/// the edit from model `a` and member `m`. Its child `decision-write(b)`,
/// whose strict prerequisite supplier is that derived parent, changes `m`.
/// The part reopens and its consumer reopens with it (#246); the edit commits
/// the fixed point.
#[test]
fn a_prerequisite_consumer_changing_its_derived_suppliers_decision_read_reaches_the_fixed_point() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-supplied-derived".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    super::derived::committed(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
    );
    let world = super::derived::observe(&request);
    assert_eq!(world.first, 11, "first:a re-derived from m = 9");
    assert_eq!(world, super::derived::sequential());
}

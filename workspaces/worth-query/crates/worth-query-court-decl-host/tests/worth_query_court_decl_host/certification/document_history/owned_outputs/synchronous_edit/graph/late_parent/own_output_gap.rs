//! A successor round that reaches a record its own chain created through a
//! relation read at the current image (PR C C4, #277 typed limit). The own-
//! output view is entity-only: the relation names the chain's output, whose
//! fields read at the chain head's pre-pin, where it does not exist. The
//! round stops typed (`Postponed { 119, PriorEffectRead }`) until #277's
//! relation overlay makes that view coherent.
use super::*;
use family::c3b::self_relation;
use family::decision_write;
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-start";
}
impl
    ApplicationOccurrenceConnectionBinding<
        Schema,
        super::super::super::super::program::Entry,
        self_relation::Feature,
    > for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    super::super::super::super::program::Entry,
    super::super::super::super::program::EntryOutput,
    self_relation::Feature,
    self_relation::InputPort,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = edit::Binding;
    type Demand = self_relation::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(self_relation::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<self_relation::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub struct Discover;
impl ApplicationConnectionIdentity for Discover {
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-discover";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, self_relation::Feature, decision_write::Feature>
    for Discover
{
}
pub type ChildConnection = ApplicationConnectionRef<
    Schema,
    self_relation::Feature,
    self_relation::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    Discover,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Discover {
    type RootDemand = self_relation::Demand;
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
        assert_eq!(row.key.as_str(), "first:a");
        Ok(vec![decision_write::Demand("b".into())])
    }
}
pub type Tree = ApplicationOutputGraph<
    Connection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.own-gap-delivery";
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
        ApplicationProgramIdentity::new("owned-outputs.late-parent.own-gap-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<Delivery>();
        specs
    }
}

/// One stage (`b = 8`) enters `self-relation(a)`, which derives `first:a`
/// (2 + 5) and reads every output model `a` owns through `FirstOf`: none
/// yet. Its child `decision-write(b)` sets m = 9, so `first:a` reopens as a
/// successor round of its own chain. That round's `FirstOf` read, at the
/// current image, reaches `first:a`, which the chain created; its value is
/// read through the own-output view at the chain head's pre-pin, where the
/// record does not exist, and the round fails. The failure is the chain's own
/// effect (#277), so the edit refuses typed
/// `CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }`
/// and publishes nothing. Red before the guard: an untyped producer failure.
#[test]
fn a_successor_reaching_its_own_created_record_by_relation_refuses_typed() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("own-output-gap".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains("CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"),
        "{detail}; {events:#?}"
    );
    assert_eq!(courts::native_head(&host), head);
}

/// The same edit with a traversal only (PR C C4 r7): the successor's
/// `FirstOf` read reaches `first:a` and never reads its value. The endpoint
/// is a record its own chain created, absent at the chain head's pre-pin, so
/// the traversal denies (`NativeRead::endpoint`) and the round stops typed
/// (`PriorEffectRead`). With a foreign linker, a relation another writer
/// made to the chain's record, nothing else would catch it (no field read,
/// and the chain's touch on the record does not intersect the anchor's
/// adjacency); no public fixture links to another producer's output, so
/// this court uses the chain's own link, which successor independence
/// would also catch after preparation.
#[test]
fn a_successor_traversing_to_its_own_created_record_refuses_typed() {
    family::c3b::self_relation::TRAVERSE_ONLY.store(true, std::sync::atomic::Ordering::Relaxed);
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<Program>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("own-output-traverse".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<Tree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let events = _execution.wave_events_for_test();
    let detail = format!("{outcome:?}");
    assert!(
        detail.contains("CoupledContinuationProvenancePostponed { issue: 119, reason: PriorEffectRead }"),
        "{detail}; {events:#?}"
    );
    assert_eq!(courts::native_head(&host), head);
}

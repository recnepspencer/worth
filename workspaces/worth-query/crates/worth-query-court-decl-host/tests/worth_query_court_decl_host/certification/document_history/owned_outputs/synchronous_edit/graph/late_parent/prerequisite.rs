//! The late child consumes `first:a` through its admitted strict prerequisite
//! (#239, #246): `decision-write(b)` names `decision-input(a)` as its supplier,
//! so its derivation consumed `first:a` and its change to `m` is held.
use super::super::super::interleaved::component_carriage::parent as owner;
use super::*;
use family::{decision_input, decision_write};
use worth_query_host::facade::primary_graph::WorthQueryCoupledSupportHold as Hold;

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
        assert_eq!((row.key.as_str(), row.value), ("first:a", 7));
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

/// The whole edit refuses on `first:a`'s support, changed in `support`,
/// attributed to the `decision-write(b)` writer (at authored stage `stage`
/// when the verified-support detail names one), with exactly `hold`.
pub(super) fn refuse_whole(
    request: &family::Request<'_, '_, '_>,
    outcome: worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome,
    hold: Hold,
    stage: Option<usize>,
    support: &str,
) {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let failure = cause
        .invariant_execution_failure()
        .unwrap_or_else(|| panic!("the final support owner refuses: {cause:?}"));
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert_eq!(failure.coupled_support_hold(), Some(hold), "{failure:?}");
    assert!(
        stage.is_none_or(|stage| failure
            .detail()
            .contains(&format!("authored stage {stage}")))
            && failure
                .detail()
                .contains("later writer producer owned-outputs.decision-write")
            && failure.detail().contains(support),
        "{failure:?}"
    );
    let model = |key: &str| {
        worth_relational::facade::transactions::RecordRef::Entity(
            request
                .query(query::model::Read(key.into()))
                .execute()
                .unwrap()
                .rows()[0]
                .native_id,
        )
    };
    // The reader's source, then the exact writer's source.
    assert_eq!(failure.affected_records(), [model("a"), model("b")]);
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
/// parent, changes `m`, which `first:a` read. The child consumed `first:a`, so
/// the edit refuses whole with CoupledReadSupportChanged held for
/// own-producer settlement (#239), attributed to the child, and nothing
/// commits.
#[test]
fn a_prerequisite_consumer_changing_its_verified_supplier_is_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, Tree);
    let before = super::super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
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
    refuse_whole(
        &request,
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Hold::ReaderProducerSupersession,
        Some(0),
        "verified output support",
    );
    assert_eq!(courts::native_head(&host), head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}

/// Nothing is settled first, so `decision-input(a)` derives `first:a` inside
/// the edit from model `a` and member `m`. Its child `decision-write(b)`,
/// whose strict prerequisite supplier is that derived parent, changes `m`.
/// The child consumed `first:a`, so the edit refuses whole with
/// CoupledReadSupportChanged held for consumer supersession (#246),
/// attributed to the child, and nothing commits.
#[test]
fn a_prerequisite_consumer_changing_its_derived_suppliers_decision_read_is_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let head = courts::native_head(&host);
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
    refuse_whole(
        &request,
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Hold::ConsumerSupersession,
        None,
        "changed decision support",
    );
    assert_eq!(courts::native_head(&host), head);
}

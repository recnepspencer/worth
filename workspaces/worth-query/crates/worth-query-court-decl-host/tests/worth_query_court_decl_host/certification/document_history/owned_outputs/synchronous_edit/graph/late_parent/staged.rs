//! Ordered stages that place the verified parent's read and the writer that
//! changes its support at chosen applied positions (#239, #244).
//!
//! Each member takes position `members so far + 1` when it appends; a verified
//! parent adopted now is observed at `members so far`. The edit's own reads are
//! Model fields no producer writes, so a stage's producer barrier is empty and
//! every root waits for the final drive, where equal-rank roots run in seed
//! order. Two stages may not write one entity, so stage 0 rewrites model `a`
//! with its current value 2, which leaves `first:a` verified.
use super::*;
use family::decision_write;
type Entry = super::super::super::super::program::Entry;
type EntryOutput = super::super::super::super::program::EntryOutput;

/// An independent root: `second(a)` reads only model `a` and creates
/// `second:a`; it neither reads nor writes `m`.
pub struct Independent;
impl ApplicationConnectionIdentity for Independent {
    const IDENTITY: &'static str = "owned-outputs.late-parent.independent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, hierarchy::Child> for Independent {}
pub type IndependentConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    EntryOutput,
    hierarchy::Child,
    hierarchy::SeedInput,
    Independent,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Independent {
    type Source = edit::Binding;
    type Demand = second::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        _: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(second::Demand("a".into()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Independent> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<second::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Independent as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(
            input,
        )
    }
}
pub type IndependentTree = ApplicationOutputGraph<IndependentConnection, ApplicationOutputLeaf>;
/// The prerequisite late-parent tree beside the independent root, both
/// entered at rank 1.
pub struct InterveningProgram;
impl ApplicationProgramDefinition<Schema> for InterveningProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(super::prerequisite::Tree, IndependentTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.intervening");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        super::prerequisite::Program::feature_specs()
    }
}

/// Only the verified parent `first:a`, with no consuming child.
pub type ParentTree = ApplicationOutputGraph<program::Connection, ApplicationOutputLeaf>;
/// `decision-write(b)` entered straight from the edit source, not discovered
/// from `first:a`.
pub struct Write;
impl ApplicationConnectionIdentity for Write {
    const IDENTITY: &'static str = "owned-outputs.late-parent.write";
}
impl ApplicationOccurrenceConnectionBinding<Schema, Entry, decision_write::Feature> for Write {}
pub type WriteConnection = ApplicationConnectionRef<
    Schema,
    Entry,
    EntryOutput,
    decision_write::Feature,
    decision_write::InputPort,
    Write,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Write {
    type Source = edit::Binding;
    type Demand = decision_write::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(decision_write::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Write> for edit::Binding {
    fn demand_from_source(
        input: &edit::Intent,
    ) -> Result<decision_write::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Write as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type WriteTree = ApplicationOutputGraph<WriteConnection, ApplicationOutputLeaf>;
pub struct ParentDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for ParentDelivery {
    type Mode = ApplicationFixedDeliveryMode<ParentTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.parent-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for ParentDelivery {
    type Root = ParentTree;
    type Projection = ApplicationNoProjection;
}
/// Both roots enter from the edit at rank 1; nothing connects them.
pub struct NonConsumingProgram;
impl ApplicationProgramDefinition<Schema> for NonConsumingProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(ParentTree, WriteTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.non-consuming");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<ParentDelivery>();
        specs
    }
}

/// Commits `b = 4` through `$tree` and settles: `first:a = 7` is then a
/// verified parent derived from model `a` and member `m`.
macro_rules! settle {
    ($host:expr, $request:expr, $program:ty, $tree:ty) => {{
        let WorthQueryApplicationPerformedMutationOutcome::Performed(done) = $request
            .advanced()
            .mutate(edit::Intent {
                key: "b".into(),
                value: 4,
            })
            .without_source()
            .idempotency(&"late-parent-staged-initial".to_owned())
            .execute_performed::<$program, $tree>(
                $host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
        else {
            panic!("the initial edit commits")
        };
        let (_, _, mut outputs) = done
            .start_required_outputs($host, $request, WorthQueryOutputDemandControls::default())
            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()))
            .into_parts();
        assert!((0..128).any(|_| matches!(
            outputs.advance($host, $request).unwrap(),
            WorthQueryApplicationProgramOutputProgress::Settled(_)
        )));
        drop(outputs);
        assert_eq!(
            $request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            7
        );
    }};
}

/// Commits `b = 4` through `$tree` as one ordered authored stage, whose
/// commit settles the tree's outputs: `first:a = 7` is then a verified parent
/// derived from model `a` and member `m`. An occurrence tree has no
/// mutation-path output continuation, so it settles through the envelope.
macro_rules! settle_staged {
    ($host:expr, $request:expr, $tree:ty) => {{
        use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
        let mut edit = $request
            .edit_envelope()
            .idempotency("late-parent-staged-settle".to_owned())
            .runtime_stages($host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<$tree, _>(edit::Intent {
            key: "b".into(),
            value: 4,
        })
        .unwrap();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
        assert_eq!(
            $request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            7
        );
    }};
}

fn stage(key: &str, value: u64) -> edit::Intent {
    edit::Intent {
        key: key.into(),
        value,
    }
}

/// The whole edit refuses on the final support owner, attributed to the
/// `decision-write(b)` writer, with exactly `hold`.
fn require_refusal(
    outcome: worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome,
    request: &family::Request<'_, '_, '_>,
    hold: Option<worth_query_host::facade::primary_graph::WorthQueryCoupledSupportHold>,
) {
    use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    let failure = cause
        .invariant_execution_failure()
        .unwrap_or_else(|| panic!("the final support owner refuses: {cause:?}"));
    assert_eq!(failure.kind(), worth_query_host::facade::primary_graph::WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged);
    assert_eq!(failure.coupled_support_hold(), hold, "{failure:?}");
    assert!(
        failure
            .detail()
            .contains("later writer producer owned-outputs.decision-write")
            && failure.detail().contains("verified output support")
            && failure.detail().contains("MemberValue"),
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

/// Stage 0 (edit `a = 2`, root `second(a)`) appends at 1 and stage 1 (edit
/// `b = 8`, root `decision-input(a)`) at 2. In the final drive `second(a)` runs first
/// and appends at 3, so `first:a` is adopted verified and observed at 3. Its
/// own child `decision-write(b)`, whose strict prerequisite supplier is
/// `first:a` and which is owed by stage 1 at 2, appends at 4 and changes `m`,
/// which `first:a` read. The child's owed position (2) is not the parent's
/// observed position (3), so a predicate keyed on owed positions would give no
/// hold. The child consumed `first:a`, so the edit refuses whole with CoupledReadSupportChanged held
/// for own-producer settlement (#239), attributed to the child, and nothing
/// commits.
#[test]
fn a_consuming_child_after_an_intervening_append_is_still_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<InterveningProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, super::prerequisite::Tree);
    let before = super::super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-intervening".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<IndependentTree, _>(stage("a", 2))
        .unwrap();
    edit.push_stage::<super::prerequisite::Tree, _>(stage("b", 8))
        .unwrap();
    require_refusal(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        &request,
        Some(worth_query_host::facade::primary_graph::WorthQueryCoupledSupportHold::ReaderProducerSupersession),
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "second(a) and the child derive; the verified parent stays unopened"
    );
    assert_eq!(courts::native_head(&host), head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}

/// Stage 0 (edit `a = 2`, root `decision-input(a)`) appends at 1 and stage 1
/// (edit `b = 8`, root `decision-write(b)` straight from the edit) at 2. In the
/// final drive `first:a` is adopted verified first, observed at 2; then
/// `decision-write(b)`, owed by stage 1 at 2, appends at 3 and changes `m`,
/// which `first:a` read. Its owed position equals the parent's observed
/// position, so a predicate keyed on owed positions would hold. It never
/// consumed `first:a`, so the edit refuses whole with a plain
/// CoupledReadSupportChanged, attributed to that writer, and nothing commits.
#[test]
fn a_non_consuming_producer_with_the_same_owed_position_is_not_held() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<NonConsumingProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle!(&host, &request, NonConsumingProgram, ParentTree);
    let before = super::super::super::failure::checkpoint(&host);
    let head = courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-non-consuming".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ParentTree, _>(stage("a", 2)).unwrap();
    edit.push_stage::<WriteTree, _>(stage("b", 8)).unwrap();
    require_refusal(
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        &request,
        None,
    );
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "only decision-write(b) derives; the verified parent stays unopened"
    );
    assert_eq!(courts::native_head(&host), head);
    super::super::super::failure::assert_model_unchanged(&host, &before);
}

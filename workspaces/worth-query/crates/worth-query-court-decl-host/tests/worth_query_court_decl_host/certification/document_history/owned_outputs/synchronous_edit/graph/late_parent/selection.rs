//! Discovery from a reopened verified parent (PR C r1 blockers 1 and 3).
//!
//! `first:a` is verified at 7 (model a 2 + member m 5). A writer of `m` stales
//! it, qualified use reopens it, and it derives again at 11. Its discovery
//! must never select from the stale 7 once that writer appended (R2), and a
//! re-discovery that drops a child it selected before must not commit (R4,
//! `SupersededSelectionDropped`, typed in C4).
use super::*;
use family::{decision_input, decision_write};
use std::cell::RefCell;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;

thread_local! {
    /// Every `first:a` value a discovery from it observed on this thread.
    static SEEN: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// `decision-write(b)` while `first:a` is 7, nothing at 11.
pub struct WhileSeven;
impl ApplicationConnectionIdentity for WhileSeven {
    const IDENTITY: &'static str = "owned-outputs.late-parent.while-seven";
}
impl
    ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, decision_write::Feature>
    for WhileSeven
{
}
pub type WhileSevenConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    decision_write::Feature,
    decision_write::InputPort,
    WhileSeven,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for WhileSeven {
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
        SEEN.with(|seen| seen.borrow_mut().push(row.value));
        Ok(if row.value == 7 {
            vec![decision_write::Demand("b".into())]
        } else {
            Vec::new()
        })
    }
}

/// Selects nothing; only records the `first:a` it was discovered from.
pub struct Observed;
impl ApplicationConnectionIdentity for Observed {
    const IDENTITY: &'static str = "owned-outputs.late-parent.observed";
}
impl ApplicationOccurrenceConnectionBinding<Schema, decision_input::Feature, hierarchy::Child>
    for Observed
{
}
pub type ObservedConnection = ApplicationConnectionRef<
    Schema,
    decision_input::Feature,
    decision_input::OutputPort,
    hierarchy::Child,
    hierarchy::SeedInput,
    Observed,
>;
impl WorthQueryApplicationDependentOutputConnection<Schema> for Observed {
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
        row: &query::Row,
    ) -> Result<Vec<Self::Demand>, WorthQueryRequiredOutputConnectionDenial> {
        SEEN.with(|seen| seen.borrow_mut().push(row.value));
        Ok(Vec::new())
    }
}

pub type SelectTree = ApplicationOutputGraph<
    program::Connection,
    ApplicationOutputEdge<WhileSevenConnection, ApplicationOutputLeaf>,
>;
pub type ObservedTree = ApplicationOutputGraph<
    program::Connection,
    ApplicationOutputEdge<ObservedConnection, ApplicationOutputLeaf>,
>;
pub struct SelectDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for SelectDelivery {
    type Mode = ApplicationFixedDeliveryMode<SelectTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.select-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for SelectDelivery {
    type Root = SelectTree;
    type Projection = ApplicationNoProjection;
}
pub struct ObservedDelivery;
impl ApplicationActionDelivery<Schema, edit::Binding> for ObservedDelivery {
    type Mode = ApplicationFixedDeliveryMode<ObservedTree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "owned-outputs.late-parent.observed-delivery";
}
impl ApplicationFixedActionDelivery<Schema, edit::Binding> for ObservedDelivery {
    type Root = ObservedTree;
    type Projection = ApplicationNoProjection;
}
/// `first:a` selects `decision-write(b)` only while it is 7.
pub struct SelectProgram;
impl ApplicationProgramDefinition<Schema> for SelectProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<SelectTree>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.select-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<SelectDelivery>();
        specs
    }
}
/// The observed parent and `decision-write(b)` as equal-rank roots.
pub struct SameBatchProgram;
impl ApplicationProgramDefinition<Schema> for SameBatchProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(ObservedTree, staged::WriteTree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("owned-outputs.late-parent.same-batch");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = super::super::hierarchy::Program::feature_specs();
        specs[0] = super::super::super::super::program::entry_spec::<ObservedDelivery>();
        specs
    }
}

pub(super) fn seen() -> Vec<u64> {
    SEEN.with(|seen| std::mem::take(&mut *seen.borrow_mut()))
}

/// The reopened parent derives at 11 and no longer selects `decision-write(b)`,
/// which already wrote `m`. The edit must not commit that dropped selection:
/// the wave refuses it once the re-discovery is identified.
#[test]
fn a_reopened_parent_that_drops_its_selection_refuses() {
    let host = world::install_program_with_other_model::<SelectProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, SelectTree);
    let head = courts::native_head(&host);
    seen();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-dropped-selection".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<SelectTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(seen(), [7, 11], "selected at 7, then dropped at 11");
    require_dropped(outcome);
    assert_eq!(courts::native_head(&host), head);
}

/// The edit refused in the wave because a reopened derivation dropped a
/// selection it had derived (typed `SupersededSelectionDropped` in C4).
pub(super) fn require_dropped(outcome: Outcome) {
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("a dropped selection must not commit: {outcome:?}")
    };
    let denial = cause
        .coupled_producer_denial()
        .unwrap_or_else(|| panic!("the wave refuses the dropped selection: {cause:?}"));
    assert!(
        format!("{denial:?}").contains("dropped a selection it derived"),
        "{denial:?}"
    );
}

/// `first:a` and `decision-write(b)` run in one batch, the writer last. The
/// writer stales `first:a`, so its discovery runs only after it derived again:
/// it never observes the stale 7, and the edit commits it at 11.
#[test]
fn a_same_batch_writer_stales_the_parent_before_its_discovery() {
    let host = world::install_program_with_other_model::<SameBatchProgram>(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        2,
        Some(7),
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle_staged!(&host, &request, ObservedTree);
    seen();
    let mut edit = request
        .edit_envelope()
        .idempotency("late-parent-same-batch".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ObservedTree, _>(edit::Intent {
        key: "a".into(),
        value: 2,
    })
    .unwrap();
    edit.push_stage::<staged::WriteTree, _>(edit::Intent {
        key: "b".into(),
        value: 8,
    })
    .unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    assert_eq!(
        seen(),
        [11],
        "discovery ran only on the derived-again parent"
    );
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2 + 9
    );
}

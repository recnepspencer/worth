//! Two real writers share a stage and settle through declared program connections.
use super::super::{
    correction_courts::install_definition,
    lifecycle::*,
    program::{Demand, Start},
    *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;

pub struct SecondStart;
impl ApplicationConnectionIdentity for SecondStart {
    const IDENTITY: &'static str = "ownership.second-start.v1";
}
impl<S: Scenario> ApplicationOccurrenceConnectionBinding<S, program::Entry, Parent>
    for SecondStart
{
}
impl<S: Scenario> WorthQueryApplicationRequiredOutputConnection<S> for SecondStart {
    type Source = super::super::producer::Binding<S>;
    type Demand = Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &query::Input,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand(input.id))
    }
}
impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, SecondStart> for RetireBinding<S> {
    fn demand_from_source(_: &Retire) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Err(WorthQueryRequiredOutputConnectionDenial::new(
            "stage pre-image is required",
        ))
    }
    fn demand_from_preimage(
        input: &Retire,
        preimage: &dyn ApplicationCorrectionPreImage<S>,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        if S::MODE == 24 {
            return Ok(Demand(input.id));
        }
        let Demand(first) =
            <Self as WorthQueryApplicationRequiredOutputSource<S, Start>>::demand_from_preimage(
                input, preimage,
            )?;
        if S::MODE == 16 {
            Ok(Demand(first))
        } else {
            second(first)
        }
    }
}
impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, SecondStart> for RestoreBinding<S> {
    fn demand_from_source(
        input: &Restore,
    ) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        second(input.value)
    }
}
fn second(first: u64) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
    first
        .checked_add(1)
        .map(Demand)
        .ok_or_else(|| WorthQueryRequiredOutputConnectionDenial::new("second BodyKey exhausted"))
}
pub struct Parent;
pub struct FirstInput;
pub struct SecondInput;
impl<S: Scenario> ApplicationFeature<S> for Parent {
    type Inputs = ApplicationFeatureInputList<
        FirstInput,
        ApplicationFeatureInputList<SecondInput, ApplicationFeatureInputLeaf>,
    >;
    const IDENTITY: &'static str = "ownership.two-body-parent.v1";
}
impl<S: Scenario> ApplicationInputPort<S, Parent> for FirstInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.first-body-input.v1";
    const REQUIRED: bool = false;
}
impl<S: Scenario> ApplicationInputPort<S, Parent> for SecondInput {
    type Value = query::RowBinding;
    const IDENTITY: &'static str = "ownership.second-body-input.v1";
    const REQUIRED: bool = false;
}
impl<S: Scenario> ApplicationOccurrenceConnectionBinding<S, program::Entry, Parent> for Start {}
type FirstConnection<S> =
    ApplicationConnectionRef<S, program::Entry, program::Output, Parent, FirstInput, Start>;
type SecondConnection<S> =
    ApplicationConnectionRef<S, program::Entry, program::Output, Parent, SecondInput, SecondStart>;
pub(in super::super) type Graph<S> =
    ApplicationDemandOutputGraph<FirstConnection<S>, ApplicationOutputLeaf>;
type SecondGraph<S> = ApplicationDemandOutputGraph<SecondConnection<S>, ApplicationOutputLeaf>;
pub struct TwoProgram;
impl<S: Scenario> ApplicationProgramDefinition<S> for TwoProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Graph<S>, SecondGraph<S>)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.two-bodies.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![program::entry_spec::<S>(), {
            let spec = ApplicationFeatureSpec::root::<S, Parent>()
                .conditional_operation::<super::super::producer::Operation>();
            if S::COUPLED {
                spec.conditional_operation::<super::producer::Operation>()
                    .finish()
            } else {
                spec.finish()
            }
        }]
    }
}
type Host<S> = WorthQueryProgramApplicationRuntime<S, TwoProgram>;
pub(in super::super) fn create<S: Scenario>(
    host: &Host<S>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
    co_anchors: Vec<u64>,
) -> WorthQueryApplicationCommitReceipt {
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: true,
            co_anchors,
            scope: 1,
            id,
            depth: 1,
        })
        .without_source()
        .idempotency(&format!("body-and-stage-{id}"))
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
}
pub(in super::super) fn settle<S: Scenario>(
    host: &Host<S>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
) {
    let mut pending = request
        .demand(Demand(id))
        .retain_program_outputs::<TwoProgram, Graph<S>>(host)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    for _ in 0..32 {
        match pending.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled
                    .close(request)
                    .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("body output did not settle");
}
fn two_bodies<const MODE: u8>() {
    type S<const MODE: u8> = Model<MODE>;
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let first = create(
        &host,
        &request,
        10,
        if MODE == 27 { vec![11, 20] } else { vec![20] },
    );
    let other = if MODE == 27 {
        None
    } else {
        Some(create(
            &host,
            &request,
            11,
            if MODE == 17 { vec![30] } else { vec![] },
        ))
    };
    if MODE == 27 {
        settle(&host, &request, 11);
    }
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    super::producer::handler::reset_contacts();
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 20,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"retire-shared-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(
        super::producer::handler::contacts(),
        if matches!(MODE, 16..=18) { 1 } else { 2 },
        "actual Query contacts before Native closure judgment"
    );
    if matches!(MODE, 16..=18) {
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("malformed roster must refuse")
        };
        let error = denial
            .authored_retirement_error()
            .expect("Native's typed exact-roster refusal");
        use worth_relational::facade::publication::AuthoredRetirementRefusal as Cause;
        assert!(
            matches!(
                (MODE, error.authored_retirement_refusal()),
                (16, Some(Cause::RepeatedCoupledOwner))
                    | (17, Some(Cause::ExtraCoupledOwner))
                    | (18, Some(Cause::MissingCoupledOwner))
            ),
            "{error:?}"
        );
        let subject = if MODE == 16 {
            &first
        } else {
            other.as_ref().expect("second source")
        };
        let native = subject
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S<MODE>>>()
            .unwrap()
            .entity_id();
        assert_eq!(
            error.context.affected_records,
            [worth_relational::facade::transactions::RecordRef::Entity(
                native
            )]
        );
        for id in [20, 101, 102, 111, 112] {
            assert!(
                request.query(query::Input { id }).execute().is_ok(),
                "roster refusal changed {id}"
            );
        }
        for id in [103, 113] {
            assert!(
                request.query(query::Input { id }).execute().is_err(),
                "roster refusal created {id}"
            );
        }
        return;
    }
    if matches!(MODE, 7 | 27 | 28) {
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = outcome
        else {
            panic!("{outcome:?}")
        };
        let failure = denial
            .invariant_execution_failure()
            .expect("typed complete-candidate support refusal");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
        );
        assert_eq!(
            failure.affected_records(),
            [worth_relational::facade::transactions::RecordRef::Entity(
                first
                    .outputs_of::<CreatedOutputs>()
                    .unwrap()
                    .entity::<Created<S<MODE>>>()
                    .unwrap()
                    .entity_id(),
            )]
        );
        for id in [20, 101, 102, 111, 112] {
            assert!(
                request.query(query::Input { id }).execute().is_ok(),
                "refusal changed {id}"
            );
        }
        for id in [103, 113] {
            assert!(
                request.query(query::Input { id }).execute().is_err(),
                "refusal created {id}"
            );
        }
    } else {
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
        assert!(request.query(query::Input { id: 20 }).execute().is_err());
        for id in if MODE == 9 {
            [101, 103, 111, 102]
        } else {
            [101, 103, 111, 113]
        } {
            assert!(
                request.query(query::Input { id }).execute().is_ok(),
                "missing committed body member {id}"
            );
        }
        assert!(request.query(query::Input { id: 112 }).execute().is_err());
        settle(&host, &request, 10);
        settle(&host, &request, 11);
        assert_eq!(
            super::producer::handler::contacts(),
            2,
            "both actual successors were settled atomically"
        );
        if MODE == 15 {
            create(&host, &request, 40, vec![]);
            super::history::require_unique_edge_ownership(
                &host,
                match &outcome {
                    WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
                    _ => unreachable!(),
                },
                first
                    .outputs_of::<CreatedOutputs>()
                    .unwrap()
                    .entity::<Created<S<MODE>>>()
                    .unwrap()
                    .entity_id(),
            );
        }
    }
}
#[test]
fn a_later_writer_cannot_change_an_earlier_writers_decision_support() {
    two_bodies::<7>();
}
#[test]
fn independent_writers_can_share_the_same_local_create_key() {
    two_bodies::<8>();
}
#[test]
fn later_unique_writes_read_the_actual_candidate_after_an_earlier_writer() {
    two_bodies::<9>();
}

#[test]
fn each_coupled_writer_owns_only_its_own_business_edges_even_with_a_shared_endpoint() {
    two_bodies::<15>();
}

#[test]
fn repeating_a_declared_body_refuses_with_that_body_and_no_effect() {
    two_bodies::<16>();
}
#[test]
fn a_declared_body_outside_the_required_owner_set_refuses_with_its_source() {
    two_bodies::<17>();
}
#[test]
fn omitting_one_required_body_refuses_with_that_missing_source_and_no_effect() {
    two_bodies::<18>();
}

#[test]
fn a_later_writer_cannot_change_a_foreign_field_in_an_earlier_component_guard() {
    two_bodies::<27>();
}

#[test]
fn a_later_writer_cannot_add_an_edge_to_an_earlier_component_visibility_walk() {
    two_bodies::<28>();
}

#[test]
fn each_coupled_writer_rebases_only_its_own_predecessor_walk() {
    two_bodies::<42>();
}

//! A read projecting `#[worth_query(group_name)]` is a source a mutation can
//! expect: regrouping the read member changes no record, yet stales the
//! source (`SourceChanged`), while regrouping another member leaves it
//! current.
use super::super::named_read::grouped_member;
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryObservedSource;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use GroupRole::{Host, Member as Plain};

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct ObserveGrouped {
    pub key: String,
}
worth_query_structured_value_binding!(pub Input for ObserveGrouped { identity: "persistent-naming.observe-grouped.v1" });
worth_query_operation!(pub ObserveGroupedOp for Schema, input Input; access {
    read_fields: [MemberKey::at("MemberKey")];
});

pub struct Binding;
impl ApplicationMutationAdmissionBinding<Schema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = ObserveGrouped;
    type InputBinding = Input;
    type Result = edit::Edited;
    type ResultBinding = edit::ResultBinding;
    type IdempotencyKey = String;
    type Operation = ObserveGroupedOp;
    type Decision = ();
    type Denial = edit::Denial;
    type DenialBinding = edit::DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<MemberKey>;
    type PrincipalBinding = Actor;
    type Mapping = Mapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<grouped_member::Query>;
    const IDENTITY: &'static str = "persistent-naming.observe-grouped.v1";
    const HANDLER_IDENTITY: &'static str = "persistent-naming.observe-grouped-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "persistent-naming.observe-grouped-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> MemberKey {
        MemberKey::at("MemberKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Actor,
        Mapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        Actor::reference()
    }
    fn expected_source_parameters(
        _: &ObserveGrouped,
    ) -> Result<
        Option<ApplicationQueryParameterSet<grouped_member::Query>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(ApplicationQueryParameterSet::new()))
    }
}
impl ApplicationMutationIntent<Schema> for ObserveGrouped {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> ApplicationMutationFieldScope<MemberKey> {
        ApplicationMutationFieldScope::new(MemberKey::at("MemberKey"), self.key.clone())
    }
}

pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        _: &ObserveGrouped,
        _: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<(), edit::Denial> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &ObserveGrouped,
        _: (),
        _: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<edit::Edited, edit::Denial> {
        HandlerResult::Completed(edit::Edited)
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let op = ObserveGroupedOp::reference();
    schema
        .declared_operation::<ObserveGroupedOp>(
            op.definition().no_external_effect().no_aftermath().finish(),
        )
        .application_mutation_binding::<Binding>()
}

/// The row source of `key`'s grouped read.
fn grouped_source(world: &World, key: &str) -> WorthQueryObservedSource<grouped_member::Query> {
    let request = world.request();
    let on_world = request.on_branch(world.host.current_world());
    let read = on_world
        .query(grouped_member::Read(key.into()))
        .execute()
        .unwrap_or_else(|cause| panic!("{key}: {cause:?}"));
    read.observed_sources()[0].clone()
}

/// `true` when a mutation expecting `source` commits, `false` when it
/// refuses `SourceChanged`; any other outcome fails the court.
fn current(
    world: &World,
    key: &str,
    source: &WorthQueryObservedSource<grouped_member::Query>,
    run: &str,
) -> bool {
    let outcome = world
        .request()
        .advanced()
        .mutate(ObserveGrouped { key: key.into() })
        .expect_source(source.clone())
        .idempotency(&run.to_owned())
        .execute_in_program(&world.host, ExecutionAllocationPolicy::SystemAllocation);
    let rendered = format!("{outcome:?}");
    match outcome {
        Ok(WorthQueryApplicationMutationOutcome::Committed { .. }) => true,
        _ if rendered.contains("SourceChanged") => false,
        other => panic!("expected Committed or SourceChanged: {other:?}"),
    }
}

/// Loops `{h1, x}` and `{h2, y}`.
fn two_loops(token: &str) -> (World, Vec<WorthQueryName<Member>>) {
    let (world, n) = seeded(token, &["h1", "x", "h2", "y"]);
    let decl = loops(
        vec![
            group(&[(Host, n[0]), (Plain, n[1])]),
            group(&[(Host, n[2]), (Plain, n[3])]),
        ],
        &[],
    );
    declare_groups(&world, &format!("{token}-loops"), decl);
    (world, n)
}

fn declare_groups(world: &World, token: &str, decl: GroupDecl) {
    super::declare(world, token, decl);
}

#[test]
fn regrouping_the_read_member_stales_a_source_projecting_its_group_name() {
    let (world, n) = two_loops("source-regroup");
    let source = grouped_source(&world, "x");
    assert!(
        current(&world, "x", &source, "source-regroup-before"),
        "a fresh source is current"
    );
    // `x` leaves `{h1, x}` for its own loop: no record changes.
    let detach = loops(vec![group(&[(Host, n[1])])], &[]);
    declare_groups(&world, "source-regroup-detach", detach);
    assert!(
        !current(&world, "x", &source, "source-regroup-after"),
        "regrouping the read member stales the source"
    );
}

#[test]
fn regrouping_another_member_leaves_the_grouped_source_current() {
    let (world, n) = two_loops("source-other");
    let source = grouped_source(&world, "x");
    let detach = loops(vec![group(&[(Host, n[3])])], &[]);
    declare_groups(&world, "source-other-detach", detach);
    assert!(
        current(&world, "x", &source, "source-other-after"),
        "regrouping `y` in another loop leaves `x`'s source current"
    );
}

/// One edit envelope: a stage regrouping per `decl`, then a stage expecting
/// `source`. The later stage reads the earlier one's unpublished candidate.
fn envelope_after_regroup(
    world: &World,
    token: &str,
    decl: GroupDecl,
    source: &WorthQueryObservedSource<grouped_member::Query>,
) -> String {
    let mut envelope = world
        .request()
        .edit_envelope()
        .idempotency(token.to_owned())
        .runtime_stages(&world.host)
        .ordered_authored()
        .unwrap();
    super::super::outputs::sculpt::author(
        &format!("{token}-regroup"),
        Plan {
            steps: vec![Step::Groups(decl)],
            ..Plan::default()
        },
    );
    let regroup = super::super::outputs::sculpt::Intent {
        token: format!("{token}-regroup"),
    };
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(regroup)
        .unwrap();
    envelope
        .push_stage_with_source::<ApplicationNoOutputGraph, _, _>(
            ObserveGrouped { key: "x".into() },
            source.clone(),
        )
        .unwrap();
    format!(
        "{:?}",
        envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
    )
}

#[test]
fn a_later_stage_never_trusts_a_grouped_source_over_an_unpublished_regrouping() {
    let (world, n) = two_loops("source-envelope");
    let source = grouped_source(&world, "x");
    let detach = loops(vec![group(&[(Host, n[1])])], &[]);
    let ended = envelope_after_regroup(&world, "source-envelope-run", detach, &source);
    assert!(
        !ended.contains("Committed"),
        "the candidate regrouped `x`: its grouped source cannot be trusted there: {ended}"
    );
}

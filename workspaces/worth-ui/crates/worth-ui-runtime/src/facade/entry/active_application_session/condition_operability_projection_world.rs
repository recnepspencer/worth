//! The world of the projection condition tests: a consumer whose mutability
//! condition reads a Query-scalar projection, beside a component whose
//! binding a later source recovers, so that recovery publishes an authored
//! content successor in the same turn that moves the projection.

use super::*;
use crate::facade::expression::UiExpressionOutcome;
use crate::runtime::rebind::{UiRebindOutcome, UiRebindPlan, UiRebindSemanticProof};
use worth_ui_query_binding::{
    UiProjectionObservation, WorthUiScalarProjectionSourceRecord, WorthUiStatusSourceOwner,
};

const COMPONENT: &str = "platform.pulse.component.projected_status";
const PROJECTION: &str = "platform.pulse.status";

/// What the consumer's mutability condition reads.
#[derive(Clone, Copy)]
pub(super) enum Mutability {
    /// `p == "ONLINE"` over the projection.
    Projected,
    /// The fixture's mutability fact, which never moves here.
    Fact,
}

/// The source a recovery turn admits: what its mutability condition reads,
/// and whether it also authors `AHEAD_WHEN`, whose identity sorts before the
/// consumer's conditions and so moves each of them to the next slot.
#[derive(Clone, Copy)]
pub(super) struct Recovery {
    pub(super) mutability: Mutability,
    pub(super) ahead: bool,
}

/// A condition no declaration reads, installed ahead of the consumer's.
const AHEAD_WHEN: &str = "test.appearance.ahead_when";

const LAUNCH: Recovery = Recovery {
    mutability: Mutability::Projected,
    ahead: false,
};

/// A consumer whose mutability reads `recovery.mutability` and whose policy
/// reads `POLICY_WHEN`, beside a component that binds the projection's
/// `field`. Only `"status"` is a field of the projection: any other field
/// leaves the binding to be recovered to the installed requirement.
fn source(field: &str, recovery: Recovery) -> WorthUiRustAuthoredArtifactInput {
    let operability = WorthUiIntentOperabilityContractSpec::new(
        "test.appearance.operability",
        WorthUiIntentMutabilitySourceSpec::condition(MUTABLE_WHEN),
        WorthUiIntentReadinessSourceSpec::application_boolean(fixture::READY),
        WorthUiIntentPolicySourceSpec::condition(POLICY_WHEN),
    );
    let (operand, body) = match recovery.mutability {
        Mutability::Projected => (
            WorthUiExpressionOperand::new(
                "p",
                WorthUiExpressionOperandSource::QueryScalar {
                    projection: PROJECTION.to_owned(),
                },
            ),
            "p == \"ONLINE\"",
        ),
        Mutability::Fact => (fact_operand(fixture::MUTABLE), "f"),
    };
    let module = fixture::consumer_module_with(Some(&fixture::role()), 1, &operability)
        .with_component_body_atoms(
            COMPONENT,
            ["content", "projection", PROJECTION]
                .map(|atom| WorthUiArtifactInputBodyAtom::Identifier(atom.to_owned())),
        )
        .try_with_query_scalar_text(
            PROJECTION,
            PROJECTION,
            field,
            WorthUiProjectionLifecycle::Live,
        )
        .unwrap()
        .try_with_condition(MUTABLE_WHEN, [operand], body)
        .unwrap()
        .try_with_condition(POLICY_WHEN, [fact_operand(fixture::POLICY)], "f")
        .unwrap();
    let module = if recovery.ahead {
        module
            .try_with_condition(AHEAD_WHEN, [fact_operand(fixture::POLICY)], "f")
            .unwrap()
    } else {
        module
    };
    WorthUiRustAuthoredArtifactInput::from_modules([module])
}

fn fact_operand(fact: &str) -> WorthUiExpressionOperand {
    WorthUiExpressionOperand::new(
        "f",
        WorthUiExpressionOperandSource::ApplicationBoolean {
            fact: fact.to_owned(),
        },
    )
}

pub(super) fn submission(
    capabilities: &crate::capability::CapabilitySnapshot,
    field: &str,
    recovery: Recovery,
    name: &str,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    crate::runtime::tests::source_ingress_boundary_test_support::lower_rust_submission(
        crate::runtime::WorthUiSourceProvider::rust_authored(name)
            .with_rust_authored_input(source(field, recovery)),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(name)],
        capabilities,
    )
}

/// A published world under `profile` whose consumer reads `ONLINE`, while
/// its component still binds the unrecovered `revision`. No pointer hovers.
pub(super) fn online(
    profile: crate::runtime::rebind::UiChangeProfile,
) -> (World, WorthUiStatusSourceOwner) {
    let owner = WorthUiStatusSourceOwner::install().unwrap();
    let (registration, pending) = owner.initial_projection().unwrap();
    let builder = || {
        fixture::builder_with_component(&fixture::role(), fixture::component(), profile)
            .register_component(crate::capability::ComponentDescriptor::new(
                crate::capability::ComponentId::new(COMPONENT).unwrap(),
                crate::capability::ComponentPropSchema::named(
                    "platform.pulse.projected_status.props",
                ),
                crate::capability::ComponentChildPolicy::no_children(),
                crate::capability::ComponentStateOwnership::runtime_owned(),
            ))
            .register_application_scalar_projection(registration.clone())
            .unwrap()
    };
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let capabilities = builder().freeze().unwrap();
    let launch = submission(capabilities.capabilities(), "revision", LAUNCH, "launch");
    let app = builder()
        .with_candidate_submission(launch)
        .freeze()
        .unwrap();
    let mut session =
        crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
            app,
            host.clone(),
        )
        .launch()
        .unwrap();
    let (surface, graph) =
        super::super::super::mounting_fixture::mount_only_appearance_consumer(&mut session);
    close(&mut session, "initial");
    session.advance_mounted_identity_frame().unwrap();
    let frame = prepare(&mut session);
    publish(&mut session, &host, frame, 1);
    let mut world = World {
        session,
        host,
        surface,
        graph,
    };
    let plan = observe(&mut world, pending, None, 2);
    execute(&mut world, plan, 2);
    let plan = observe(&mut world, status(&owner, "ONLINE", 1), None, 3);
    execute(&mut world, plan, 3);
    (world, owner)
}

/// The `online` world with its consumer hovered, painted operable.
pub(super) fn hovered_online(
    profile: crate::runtime::rebind::UiChangeProfile,
) -> (World, WorthUiStatusSourceOwner, UiMountedInstanceIdentity) {
    let (mut world, owner) = online(profile);
    let (target, _) = activate(&mut world.session, world.surface, 1);
    close(&mut world.session, "hovered");
    let frame = project(
        &mut world.session,
        &[(target, 10)],
        &[(
            world.surface,
            Some((target, UiPointerAffordanceFamily::Activation)),
        )],
    );
    publish(&mut world.session, &world.host, frame, 5);
    (world, owner, target)
}

/// Closes an observation turn over the unchanged launch source.
pub(super) fn close(session: &mut crate::facade::WorthUiActiveApplicationSession, name: &str) {
    let source = submission(session.capabilities(), "revision", LAUNCH, name);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(source).unwrap();
    let admitted = turn.seal().unwrap();
    session.classify_observations(admitted).unwrap();
}

/// Plans the rebind of one turn that observes `observation` and, when
/// `recovery` names one, a source that recovers the binding to `status`.
pub(super) fn observe(
    world: &mut World,
    observation: UiProjectionObservation,
    recovery: Option<Recovery>,
    request: u64,
) -> UiRebindPlan {
    let source = recovery.map(|recovery| {
        let name = format!("recovery-{request}");
        submission(world.session.capabilities(), "status", recovery, &name)
    });
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_projection_query(observation).unwrap();
    if let Some(source) = source {
        turn.admit_source(source).unwrap();
    }
    let admitted = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        world.session.classify_observations(admitted).unwrap()
    else {
        panic!("a projection value changes mounted content");
    };
    let lifecycle = world
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = world
        .session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    assert_eq!(
        matches!(
            plan.semantic_proof(),
            UiRebindSemanticProof::AuthoredContent(_)
        ),
        recovery.is_some(),
        "a recovered binding publishes an authored successor; a value alone does not"
    );
    plan
}

fn prepare_plan(
    world: &mut World,
    plan: UiRebindPlan,
    request: u64,
) -> crate::runtime::rebind::UiPreparedRebind<'_> {
    world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(request),
        )
        .unwrap()
}

/// Executes `plan` with its presentation left in flight, detached from the
/// session as the native loop holds it.
pub(super) fn in_flight(
    world: &mut World,
    plan: UiRebindPlan,
    request: u64,
) -> crate::runtime::rebind::UiDetachedRebindCompletion {
    world.host.push_in_flight(
        vec![
            crate::certification_support::ScriptedSurfaceCompletion::Presented(
                crate::certification_support::ScriptedPresentationAcknowledgement::new(
                    crate::facade::mounted::UiHostSurfacePresentationMode::NativeDisplay,
                    crate::certification_support::scripted_presentation_epoch(),
                    UiMountedCompletedEffects::new(Vec::new()),
                    UiHostPresentationCostReport::default(),
                ),
            ),
        ],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let UiRebindOutcome::InFlight(pending) = prepare_plan(world, plan, request).execute(request)
    else {
        panic!("the successor's presentation stays in flight");
    };
    pending.detach_for_native()
}

pub(super) fn execute(world: &mut World, plan: UiRebindPlan, request: u64) {
    let host = world.host.clone();
    let prepared = prepare_plan(world, plan, request);
    for _ in prepared
        .prepared_frame()
        .into_iter()
        .flat_map(|frame| frame.surfaces())
    {
        host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        prepared.execute(request),
        UiRebindOutcome::Published(_)
    ));
}

pub(super) fn status(
    owner: &WorthUiStatusSourceOwner,
    status: &str,
    revision: u64,
) -> UiProjectionObservation {
    owner
        .publish_source(WorthUiScalarProjectionSourceRecord::new(status, revision).unwrap())
        .unwrap()
        .into_projection_observation()
        .unwrap()
}

pub(super) fn outcome(world: &World, identity: &str) -> UiExpressionOutcome {
    let record = world.session.expression_record(identity).unwrap();
    assert_eq!(
        record.generation(),
        &world.session.active_generation_identity()
    );
    record.outcome().clone()
}

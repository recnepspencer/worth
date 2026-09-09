use worth_ui::facade::{
    app::{WorthUiNativeIntentTransition, WorthUiNativeManagedIntentConsequencePublicationOutcome},
    declaration::{
        CommandDescriptor, CommandId, UiCommandKeyCode, UiCommandModifierSet,
        UiCommandShortcutSequence, UiCommandShortcutStroke,
    },
    intent::{
        UiIntentDefinition, UiIntentExecutionAdvanceOutcome, UiIntentExecutionTransitionPosture,
        UiIntentRuntimeServiceDestination,
    },
    interaction::{
        UiCommandRoutingOutcome, UiCommandRoutingSuppression, UiHostInteractionIngressOutcome,
    },
    observation_report::{
        UiHostImeCompositionPhase, UiHostImePreedit, UiHostKey, UiHostKeyTransition,
        UiHostKeyboardModifiers, UiHostObservationBatch, UiHostObservationBatchInput,
        UiHostObservationDrain, UiHostObservationLoss, UiHostObservationPayload,
        UiHostObservationPresentationBasis, UiHostObservationReport, UiHostObservationSequence,
        UiHostObservationSequenceRange, UiHostObservationTimeBasis, UiHostProtocolContract,
        UiHostProtocolNegotiation,
    },
};
use worth_ui_runtime::facade::mounted::{
    UiMountedFrameOutcome, UiMountedInspectionReceipt, UiMountedInspectionRequest,
};
use worth_ui_test_support::WorthUiRuntimeServiceInstallationCertificationExt;

use super::{
    declaration::support::{routed_command_input, routed_command_replacement_input, AdvanceStatus},
    interaction_world::InteractionWorld,
};
use crate::filesystem_mounted_world::{
    launch_rust_command_intent_world, prepare_rust_command_intent_application,
    prepare_rust_command_intent_application_with_policy,
};

#[path = "command_routing/rebind.rs"]
mod rebind;

#[test]
fn validated_host_shortcut_reaches_the_existing_managed_intent_lifecycle() {
    let host = native_command_host();
    host.push_native_display_presented();
    let application = prepare_rust_command_intent_application::<AdvanceStatus>(
        routed_command_input(),
        command_descriptor(),
        host,
    );
    let mut shell = application
        .launch_native_surface()
        .expect("the native command world launches through the production composition root");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    let frame = shell
        .present_frame(10, 1)
        .unwrap_or_else(|_| panic!("native frame executes"));
    assert!(matches!(frame, UiMountedFrameOutcome::Published(_)));
    let presentation = current_presentation(&shell);
    let definition = UiIntentDefinition::<AdvanceStatus>::runtime_service(
        UiIntentRuntimeServiceDestination::InvokeCommand,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        shortcut_drain(shell.host_session_identity().as_u64(), presentation, false),
        crate::intent::execution::execution_deadline(20),
    );
    let reference = match ingress.transitions() {
        [WorthUiNativeIntentTransition::AttemptPrepared(prepared)] => prepared
            .dispatch()
            .evidence_reference()
            .expect("a command route retains its own causal evidence reference"),
        transitions => panic!(
            "expected one prepared command attempt, got {} transition(s)",
            transitions.len()
        ),
    };
    assert!(matches!(
        ingress.transitions(),
        [WorthUiNativeIntentTransition::AttemptPrepared(_)]
    ));
    let trace = match shell.lookup_intent_causal_trace(reference) {
        worth_ui::facade::inspection::UiIntentEvidenceLookup::Found(trace) => trace,
        lookup => panic!("command causal evidence must remain inspectable: {lookup:?}"),
    };
    assert_eq!(
        trace.interaction().family(),
        worth_ui::facade::inspection::UiIntentInteractionEvidenceFamily::CommandRoute
    );
    let report = match shell
        .advance_native_intent_executions(crate::intent::execution::execution_reading(1))
    {
        UiIntentExecutionAdvanceOutcome::Advanced(report) => report,
        UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
            panic!("command execution advance stopped: {stop:?}")
        }
    };
    let mut transitions = report.into_transitions().into_vec();
    assert_eq!(transitions.len(), 1);
    let transition = transitions.pop().expect("one command transition");
    assert!(matches!(
        transition.posture(),
        UiIntentExecutionTransitionPosture::Completed { .. }
    ));
    let consequence = transition
        .into_consequence()
        .expect("completed command execution retains consequence authority");
    match shell
        .begin_managed_native_intent_consequence_publication(consequence, 20)
        .expect("the routed consequence belongs to this native session")
    {
        WorthUiNativeManagedIntentConsequencePublicationOutcome::NoConsequences(_) => {}
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(_) => {
            panic!("command with no product consequences cannot publish mounted work")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending => {
            panic!("command with no product consequences cannot remain pending")
        }
        WorthUiNativeManagedIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("routed command consequence stopped: {stop:?}")
        }
    }
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.command_routes_released(), 1);
}

#[test]
fn command_only_application_allocates_only_the_command_owner() {
    let session = launch_rust_command_intent_world::<AdvanceStatus>(
        routed_command_input(),
        command_descriptor(),
    );
    let installed = session.inspect_runtime_service_installation_for_certification();

    assert_eq!(installed.installed_family_count(), 1);
    assert!(installed.command_routing());
    assert!(!installed.portal());
    assert!(!installed.focus());
    assert!(!installed.motion());
    assert!(!installed.scroll());
    assert!(!installed.selection());
    drop(session.shutdown());
}

#[test]
fn ime_composition_suppresses_shortcuts_until_composition_ends() {
    let mut world = command_world();
    let preedit =
        UiHostImePreedit::from_unicode_scalar_range("p", None).expect("nonempty preedit is valid");
    assert!(matches!(
        world.payload_at(
            1,
            10,
            UiHostObservationPayload::ImeComposition {
                revision: 1,
                phase: UiHostImeCompositionPhase::Preedit(preedit),
            },
        ),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let UiHostInteractionIngressOutcome::Applied(suppressed) =
        world.payload_at(2, 11, shortcut_payload())
    else {
        panic!("the composed keyboard observation remains structurally valid")
    };
    assert!(matches!(
        suppressed.command_routes(),
        [UiCommandRoutingOutcome::Suppressed(
            UiCommandRoutingSuppression::ImeComposition
        )]
    ));
    assert!(matches!(
        world.payload_at(
            3,
            12,
            UiHostObservationPayload::ImeComposition {
                revision: 2,
                phase: UiHostImeCompositionPhase::Commit("p".into()),
            },
        ),
        UiHostInteractionIngressOutcome::Applied(_)
    ));
    let UiHostInteractionIngressOutcome::Applied(routed) =
        world.payload_at(4, 13, shortcut_payload())
    else {
        panic!("the post-composition keyboard observation remains valid")
    };
    assert!(matches!(
        routed.command_routes(),
        [UiCommandRoutingOutcome::Routed(_)]
    ));
    let _ = world.session.shutdown();
}

#[test]
fn native_repeat_policy_changes_attempt_admission_in_the_production_composition_root() {
    let default =
        launch_native_command_shell(worth_ui::facade::service::UiCommandRoutingPolicy::desktop());
    assert_native_repeat_attempt_count(default, 0);

    let permissive = launch_native_command_shell(
        worth_ui::facade::service::UiCommandRoutingPolicy::desktop().with_repeat_suppression(false),
    );
    assert_native_repeat_attempt_count(permissive, 1);
}

fn launch_native_command_shell(
    policy: worth_ui::facade::service::UiCommandRoutingPolicy,
) -> worth_ui::facade::app::WorthUiNativeApplicationShell {
    let host = native_command_host();
    host.push_native_display_presented();
    let application = prepare_rust_command_intent_application_with_policy::<AdvanceStatus>(
        routed_command_input(),
        command_descriptor(),
        host,
        policy,
    );
    let mut shell = application
        .launch_native_surface()
        .expect("native command policy fixture launches");
    crate::mounted_geometry_fixture::install_native_occurrence_geometry(&mut shell);
    assert!(matches!(
        shell
            .present_frame(10, 1)
            .unwrap_or_else(|_| panic!("native frame executes")),
        UiMountedFrameOutcome::Published(_)
    ));
    shell
}

fn assert_native_repeat_attempt_count(
    mut shell: worth_ui::facade::app::WorthUiNativeApplicationShell,
    expected: usize,
) {
    let presentation = current_presentation(&shell);
    let definition = UiIntentDefinition::<AdvanceStatus>::runtime_service(
        UiIntentRuntimeServiceDestination::InvokeCommand,
    );
    let ingress = shell.admit_native_intent_observations(
        definition,
        shortcut_drain(shell.host_session_identity().as_u64(), presentation, true),
        crate::intent::execution::execution_deadline(20),
    );
    assert_eq!(ingress.transitions().len(), expected);
    let shutdown = shell.shutdown();
    assert!(shutdown.intent_resources_empty());
    assert_eq!(shutdown.command_routes_released(), 1);
}

fn command_world() -> InteractionWorld {
    let command = command_descriptor();
    let session =
        launch_rust_command_intent_world::<AdvanceStatus>(routed_command_input(), command);
    InteractionWorld::from_session(session)
}

fn command_descriptor() -> CommandDescriptor {
    CommandDescriptor::new(
        CommandId::new("platform.pulse.advance").expect("valid command identity"),
        "Advance platform pulse",
    )
    .with_default_shortcut(UiCommandShortcutSequence::single(
        UiCommandShortcutStroke::logical(
            UiCommandKeyCode::P,
            UiCommandModifierSet::none().with_primary().with_shift(),
        ),
    ))
    .with_intent_destination::<AdvanceStatus>()
}

fn shortcut_payload() -> UiHostObservationPayload {
    key_payload(UiHostKey::P)
}

fn key_payload(logical_key: UiHostKey) -> UiHostObservationPayload {
    UiHostObservationPayload::Keyboard {
        logical_key,
        physical_key: Some(logical_key),
        modifiers: UiHostKeyboardModifiers::new(false, true, true, false, false),
        transition: UiHostKeyTransition::Pressed { repeat: false },
    }
}

fn native_command_host() -> worth_ui_runtime::certification_support::ScriptedPresentationHost {
    let host = worth_ui_runtime::certification_support::ScriptedPresentationHost::default();
    host.set_capabilities(
        worth_ui_host_contract::WorthUiHostCapabilityReport::available(vec![
            worth_ui_host_contract::WorthUiHostCapability::NativePaint,
            worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
            worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            worth_ui_host_contract::WorthUiHostCapability::FontMetrics,
            worth_ui_host_contract::WorthUiHostCapability::TextIntrinsicMeasurement,
            worth_ui_host_contract::WorthUiHostCapability::TextBaselineMeasurement,
        ]),
    );
    host
}

fn current_presentation(
    shell: &worth_ui::facade::app::WorthUiNativeApplicationShell,
) -> UiHostObservationPresentationBasis {
    let inspected = match shell.inspect_mounted_frame(UiMountedInspectionRequest::current()) {
        UiMountedInspectionReceipt::Available(frame) => frame,
        UiMountedInspectionReceipt::Omitted(denial) => {
            panic!("the command frame must remain inspectable: {denial:?}")
        }
    };
    let surface = &inspected.presentation().surfaces()[0];
    UiHostObservationPresentationBasis::new(
        surface.host_surface(),
        inspected.frame(),
        surface.binding(),
        surface.epoch(),
    )
}

fn shortcut_drain(
    host_session: u64,
    presentation: UiHostObservationPresentationBasis,
    repeat: bool,
) -> UiHostObservationDrain {
    let sequence = UiHostObservationSequence::new(1);
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol negotiates"),
    };
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session,
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(10),
            match shortcut_payload() {
                UiHostObservationPayload::Keyboard {
                    logical_key,
                    physical_key,
                    modifiers,
                    ..
                } => UiHostObservationPayload::Keyboard {
                    logical_key,
                    physical_key,
                    modifiers,
                    transition: UiHostKeyTransition::Pressed { repeat },
                },
                _ => unreachable!("shortcut payload is keyboard input"),
            },
        )],
    })
    .expect("shortcut batch satisfies the host protocol");
    UiHostObservationDrain::bounded(vec![batch]).expect("one shortcut is bounded")
}

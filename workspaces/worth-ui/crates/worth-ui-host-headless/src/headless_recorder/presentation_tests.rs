use worth_ui_host_contract::{
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfaceIdentity,
    UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode, UiMountedFrameConsumptionInput,
    UiMountedFrameIdentity, UiMountedLogicalDamage, UiMountedPaintCommand,
    UiMountedPaintCommandChange, UiMountedPaintOrderEdit, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPresentationAttemptIdentity, UiMountedPresentationDelta,
    UiMountedPresentationDeltaInput, UiMountedPresentationInitial,
    UiMountedPresentationInitialInput, UiMountedPresentationOpacity, UiMountedPresentationSample,
    UiMountedPresentationSampleChange, UiMountedPresentationSampleInput,
    UiMountedPresentationTransform, UiMountedPresentationWorkView,
    UiMountedSurfaceBindingRequirement, WorthUiHostCapabilityObservationGeneration,
};

use super::{apply_work, validated_initial_commands};

#[test]
fn initial_rejects_duplicate_command_identity_before_retention() {
    let valid = valid_initial();
    let mut commands = valid.commands().to_vec();
    commands.push(commands[0].clone());
    let malformed = rebuild_initial(
        &valid,
        commands,
        valid.order().to_vec(),
        valid.order_integrity(),
    );

    assert!(matches!(
        validated_initial_commands(&malformed),
        Err(UiHostSurfacePresentationDenial::MalformedProjection)
    ));
}

#[test]
fn initial_rejects_order_that_omits_a_command() {
    let valid = valid_initial();
    let mut order = valid.order().to_vec();
    order.pop().expect("fixture supplies paint order");
    let malformed = rebuild_initial(
        &valid,
        valid.commands().to_vec(),
        order,
        valid.order_integrity(),
    );

    assert!(matches!(
        validated_initial_commands(&malformed),
        Err(UiHostSurfacePresentationDenial::MalformedProjection)
    ));
}

#[test]
fn initial_rejects_stale_order_integrity_for_exact_membership() {
    let valid = valid_initial();
    let malformed = rebuild_initial(
        &valid,
        valid.commands().to_vec(),
        valid.order().to_vec(),
        UiMountedPaintOrderIntegrity::for_order(&[]),
    );

    assert!(matches!(
        validated_initial_commands(&malformed),
        Err(UiHostSurfacePresentationDenial::MalformedProjection)
    ));
}

#[test]
fn initial_rejects_command_identity_that_disagrees_with_its_mechanic() {
    let valid = valid_initial();
    let foreign = valid_initial();
    let mut commands = valid.commands().to_vec();
    commands[0] = command_with_identity(commands[0].clone(), foreign.commands()[0].identity());
    assert_malformed_commands(&valid, commands);
}

#[test]
fn initial_rejects_command_payload_that_disagrees_with_projection_row() {
    let valid = valid_initial();
    let foreign = valid_initial();
    let mut commands = valid.commands().to_vec();
    commands[0] = command_with_payload(commands[0].clone(), foreign.commands()[0].clone());
    assert_malformed_commands(&valid, commands);
}

#[test]
fn ordinary_delta_returns_one_delta_record_without_parallel_retained_history() {
    let (initial, requirement) = valid_initial_with_requirement();
    let capacity = crate::UiHeadlessRecorderCapacity::production_default();
    let mut retained = None;
    let text = worth_ui_test_support::semantic_text_layout_resolver_for_certification();
    let initial_view = view(
        &text,
        requirement,
        UiMountedPresentationWorkView::Initial(&initial),
    );
    let (recorded, _) = apply_work(&initial_view, capacity, &mut retained).unwrap();
    assert!(matches!(
        recorded,
        Some(super::super::recorded_frame::UiHeadlessRecordedFrame::Complete(_))
    ));
    let removed = initial.commands()[0].identity();
    let remaining = &initial.order()[1..];
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: initial.affinity().successor(),
        successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
        surface: initial.affinity().surface(),
        binding: initial.affinity().binding(),
        content: initial.affinity().content(),
        baseline: initial.affinity().baseline(),
        changes: vec![UiMountedPaintCommandChange::Remove(removed)],
        nodes: Vec::new(),
        order: vec![UiMountedPaintOrderEdit::remove(
            UiMountedPaintOrderIdentity::for_command(removed),
        )],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(remaining),
        damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
            initial.commands()[0].bounds(),
        )],
        auxiliary: None,
        production_cost: Default::default(),
    });
    let delta_view = view(
        &text,
        requirement,
        UiMountedPresentationWorkView::Delta(&delta),
    );
    let (recorded, _) = apply_work(&delta_view, capacity, &mut retained).unwrap();
    assert!(matches!(
        recorded,
        Some(super::super::recorded_frame::UiHeadlessRecordedFrame::Delta(_))
    ));
    let retained = retained.as_ref().unwrap();
    assert_eq!(retained.frame, delta.affinity().successor());
    assert_eq!(retained.commands.len(), initial.commands().len() - 1);
}

#[test]
fn sample_rejects_stale_frame_and_unknown_command_without_mutating_overrides() {
    let (initial, requirement) = valid_initial_with_requirement();
    let capacity = crate::UiHeadlessRecorderCapacity::production_default();
    let text = worth_ui_test_support::semantic_text_layout_resolver_for_certification();
    let mut retained = None;
    apply_work(
        &view(
            &text,
            requirement,
            UiMountedPresentationWorkView::Initial(&initial),
        ),
        capacity,
        &mut retained,
    )
    .unwrap();

    let stale = sample_for(
        &initial,
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        initial.commands()[0].identity(),
    );
    assert!(apply_work(
        &view(
            &text,
            requirement,
            UiMountedPresentationWorkView::Sample(&stale),
        ),
        capacity,
        &mut retained,
    )
    .is_err());
    assert_eq!(retained.as_ref().unwrap().sample_overrides().count(), 0);

    let foreign = valid_initial();
    let unknown = sample_for(
        &initial,
        initial.affinity().successor(),
        foreign.commands()[0].identity(),
    );
    assert!(apply_work(
        &view(
            &text,
            requirement,
            UiMountedPresentationWorkView::Sample(&unknown),
        ),
        capacity,
        &mut retained,
    )
    .is_err());
    assert_eq!(retained.as_ref().unwrap().sample_overrides().count(), 0);
}

fn sample_for(
    initial: &UiMountedPresentationInitial,
    frame: UiMountedFrameIdentity,
    command: worth_ui_host_contract::UiMountedPaintCommandIdentity,
) -> UiMountedPresentationSample {
    let bounds = initial.commands()[0].bounds();
    UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
        frame,
        surface: initial.affinity().surface(),
        binding: initial.affinity().binding(),
        content: initial.affinity().content(),
        baseline: initial.affinity().baseline(),
        changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
            command,
            Some(UiMountedPresentationTransform::from_runtime_sampling(bounds, bounds).unwrap()),
            UiMountedPresentationOpacity::from_runtime_composition(32_768),
        )],
        damage: vec![UiMountedLogicalDamage::from_runtime_mounting(bounds)],
        production_cost: Default::default(),
    })
    .unwrap()
}

fn view<'work>(
    text: &'work dyn worth_ui_host_contract::UiMountedQualifiedTextResolver,
    requirement: UiMountedSurfaceBindingRequirement,
    presentation_work: UiMountedPresentationWorkView<'work>,
) -> worth_ui_host_contract::UiMountedFrameConsumptionView<'work> {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol must negotiate"),
    };
    worth_ui_host_contract::UiMountedFrameConsumptionView::from_inert_mechanics(
        UiMountedFrameConsumptionInput {
            qualified_text: text,
            text_raster_work: None,
            authority: std::rc::Rc::new(()),
            host_session_identity: 13,
            protocol,
            capability_generation: requirement.capability_generation(),
            capability_profile_digest: requirement.capability_profile_digest(),
            attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            deadline: worth_ui_host_contract::UiPresentationDeadline::at_tick(20),
            requirement,
            presentation_work,
        },
    )
}

fn assert_malformed_commands(
    valid: &UiMountedPresentationInitial,
    commands: Vec<UiMountedPaintCommand>,
) {
    let malformed = rebuild_initial(
        valid,
        commands,
        valid.order().to_vec(),
        valid.order_integrity(),
    );
    assert!(matches!(
        validated_initial_commands(&malformed),
        Err(UiHostSurfacePresentationDenial::MalformedProjection)
    ));
}

fn command_with_identity(
    command: UiMountedPaintCommand,
    identity: worth_ui_host_contract::UiMountedPaintCommandIdentity,
) -> UiMountedPaintCommand {
    match command {
        UiMountedPaintCommand::FilledRect { mechanic, .. } => {
            UiMountedPaintCommand::FilledRect { identity, mechanic }
        }
        UiMountedPaintCommand::SemanticText { mechanic, .. } => {
            UiMountedPaintCommand::SemanticText { identity, mechanic }
        }
        UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
            UiMountedPaintCommand::PortalOverlay { identity, mechanic }
        }
    }
}

fn command_with_payload(
    command: UiMountedPaintCommand,
    donor: UiMountedPaintCommand,
) -> UiMountedPaintCommand {
    match (command, donor) {
        (
            UiMountedPaintCommand::FilledRect { identity, .. },
            UiMountedPaintCommand::FilledRect { mechanic, .. },
        ) => UiMountedPaintCommand::FilledRect { identity, mechanic },
        (
            UiMountedPaintCommand::SemanticText { identity, .. },
            UiMountedPaintCommand::SemanticText { mechanic, .. },
        ) => UiMountedPaintCommand::SemanticText { identity, mechanic },
        (
            UiMountedPaintCommand::PortalOverlay { identity, .. },
            UiMountedPaintCommand::PortalOverlay { mechanic, .. },
        ) => UiMountedPaintCommand::PortalOverlay { identity, mechanic },
        _ => panic!("fixture donor must use the same drawable family"),
    }
}

fn valid_initial() -> UiMountedPresentationInitial {
    valid_initial_with_requirement().0
}

fn valid_initial_with_requirement() -> (
    UiMountedPresentationInitial,
    UiMountedSurfaceBindingRequirement,
) {
    let projection = worth_ui_test_support::semantic_text_projection_for_certification(
        worth_ui_test_support::UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().expect("host surface identity"),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::RecordOnly,
    );
    (
        worth_ui_test_support::initial_presentation_mechanics_for_certification(
            &projection,
            requirement,
        ),
        requirement,
    )
}

fn rebuild_initial(
    valid: &UiMountedPresentationInitial,
    commands: Vec<UiMountedPaintCommand>,
    order: Vec<UiMountedPaintOrderIdentity>,
    order_integrity: UiMountedPaintOrderIntegrity,
) -> UiMountedPresentationInitial {
    let affinity = valid.affinity();
    UiMountedPresentationInitial::from_inert_mechanics(UiMountedPresentationInitialInput {
        successor: affinity.successor(),
        surface: affinity.surface(),
        binding: affinity.binding(),
        content: affinity.content(),
        baseline: affinity.baseline(),
        projection: valid.projection().clone(),
        commands,
        order,
        order_integrity,
        damage: valid.damage().to_vec(),
        production_cost: valid.production_cost(),
    })
}

use worth_ui_host_contract::{
    UiHostProtocolContract, UiHostProtocolNegotiation, UiHostSurfaceIdentity,
    UiHostSurfacePresentationMode, UiMountedFrameConsumptionInput, UiMountedFrameIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationOpacity,
    UiMountedPresentationReconstruction, UiMountedPresentationReconstructionInput,
    UiMountedPresentationSampleChange, UiMountedPresentationWorkView,
    UiMountedSurfaceBindingRequirement, WorthUiHostCapabilityObservationGeneration,
};

#[test]
fn reconstruction_retains_the_exact_accepted_motion_override() {
    let (reconstruction, requirement, expected) = fixture(false);
    let text = worth_ui_test_support::semantic_text_layout_resolver_for_certification();
    let mut retained = None;
    super::apply_work(
        &view(
            &text,
            requirement,
            UiMountedPresentationWorkView::Reconstruction(&reconstruction),
        ),
        crate::UiHeadlessRecorderCapacity::production_default(),
        &mut retained,
    )
    .unwrap();

    let mut retained = retained.unwrap();
    let epoch = worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(19);
    retained.epoch = Some(epoch);
    let observation =
        super::super::sample_observation::UiHeadlessPresentationSampleObservation::from_retained(
            &retained,
        )
        .unwrap();
    assert_eq!(observation.frame(), reconstruction.affinity().successor());
    assert_eq!(observation.epoch(), epoch);
    assert_eq!(observation.changes(), &[expected]);
    assert_eq!(observation.damage(), reconstruction.damage());
}

#[test]
fn reconstruction_rejects_duplicate_motion_overrides_before_retention() {
    let (reconstruction, requirement, _) = fixture(true);
    let text = worth_ui_test_support::semantic_text_layout_resolver_for_certification();
    let mut retained = None;
    assert!(super::apply_work(
        &view(
            &text,
            requirement,
            UiMountedPresentationWorkView::Reconstruction(&reconstruction),
        ),
        crate::UiHeadlessRecorderCapacity::production_default(),
        &mut retained,
    )
    .is_err());
    assert!(retained.is_none());
}

fn fixture(
    duplicate: bool,
) -> (
    UiMountedPresentationReconstruction,
    UiMountedSurfaceBindingRequirement,
    UiMountedPresentationSampleChange,
) {
    let projection = worth_ui_test_support::semantic_text_projection_for_certification(
        worth_ui_test_support::UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let requirement = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::RecordOnly,
    );
    let initial = worth_ui_test_support::initial_presentation_mechanics_for_certification(
        &projection,
        requirement,
    );
    let expected = UiMountedPresentationSampleChange::from_runtime_sampling(
        initial.commands()[0].identity(),
        None,
        UiMountedPresentationOpacity::from_runtime_composition(32_768),
    );
    let mut sample_overrides = vec![expected];
    if duplicate {
        sample_overrides.push(expected);
    }
    let affinity = initial.affinity();
    (
        UiMountedPresentationReconstruction::from_inert_mechanics(
            UiMountedPresentationReconstructionInput {
                predecessor: UiMountedFrameIdentity::mint_unbound().unwrap(),
                successor: affinity.successor(),
                surface: affinity.surface(),
                binding: affinity.binding(),
                content: affinity.content(),
                baseline: affinity.baseline(),
                projection: initial.projection().clone(),
                commands: initial.commands().to_vec(),
                sample_overrides,
                order: initial.order().to_vec(),
                order_integrity: initial.order_integrity(),
                damage: initial.damage().to_vec(),
                production_cost: Default::default(),
            },
        ),
        requirement,
        expected,
    )
}

fn view<'work>(
    text: &'work dyn worth_ui_host_contract::UiMountedQualifiedTextResolver,
    requirement: UiMountedSurfaceBindingRequirement,
    presentation_work: UiMountedPresentationWorkView<'work>,
) -> worth_ui_host_contract::UiMountedFrameConsumptionView<'work> {
    let protocol = match UiHostProtocolContract::current().negotiate() {
        UiHostProtocolNegotiation::Compatible(protocol) => protocol,
        UiHostProtocolNegotiation::Incompatible(_) => panic!("current protocol negotiates"),
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

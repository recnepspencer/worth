use super::UiMountedAppearanceFrameState;
use crate::mounting::projection::frame_storage::UiMountedAppearanceNodeInputContext;

#[path = "appearance_state_capacity_tests.rs"]
mod capacity_tests;

#[derive(Clone, Copy)]
struct ContextIdentities {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    incarnation: worth_ui_host_contract::UiMountIncarnation,
    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
}

fn context_identities() -> ContextIdentities {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    ContextIdentities {
        frame,
        issuer,
        instance,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        incarnation: worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap(),
        node_receipt: issuer.receipt_for(instance),
    }
}

fn context(
    identities: ContextIdentities,
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ordinal: u64,
) -> crate::runtime::appearance::UiAppearanceAttemptContext {
    let target = crate::runtime::appearance::UiAppearanceTarget::new(
        session,
        identities.surface,
        crate::graph::UiGraphNodeIdentity::new(ordinal),
        identities.instance,
        identities.incarnation,
        identities.node_receipt,
    )
    .unwrap();
    crate::runtime::appearance::UiAppearanceAttemptContext::new(
        target,
        identities.frame,
        identities.issuer,
        ordinal,
        worth_ui_host_contract::UiMountedAllocationProjection::Omitted(
            worth_ui_host_contract::UiMountedOmissionReason::NoCommittedAllocation,
        ),
        generation.clone(),
        0,
        0,
    )
}

fn node_for(
    context: &crate::runtime::appearance::UiAppearanceAttemptContext,
) -> UiMountedAppearanceNodeInputContext {
    UiMountedAppearanceNodeInputContext {
        frame: context.frame(),
        semantic_surface: context.semantic_surface(),
        mounted_instance: context.mounted_instance(),
        graph_node: context.graph_node(),
        incarnation: context.incarnation(),
        node_receipt: context.node_receipt(),
        issuer: context.issuer(),
        plan_digest: 0,
        allocation: worth_ui_host_contract::UiMountedAllocationProjection::Omitted(
            worth_ui_host_contract::UiMountedOmissionReason::NoCommittedAllocation,
        ),
    }
}

#[test]
fn appearance_state_prunes_stale_foreign_and_retired_membership_before_replacement_reservation() {
    let (session, binding, target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let identities = context_identities();
    let live = context(
        identities,
        session_identity,
        &generation,
        target.graph_node().digest(),
    );
    let stale_source = crate::runtime::tests::appearance_component_session_test_support::
        source_backed_static_paint_consumer_session();
    let stale_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        session_identity,
        stale_source.generation_identity(),
    );
    let stale_generation_entry = context(
        identities,
        session_identity,
        &stale_generation,
        live.graph_node().digest(),
    );
    let changed_incarnation = worth_ui_host_contract::UiMountIncarnation::mint_unbound().unwrap();
    let changed_incarnation_entry = context(
        ContextIdentities {
            incarnation: changed_incarnation,
            ..identities
        },
        session_identity,
        &generation,
        live.graph_node().digest(),
    );
    let retired = context(
        identities,
        session_identity,
        &generation,
        live.graph_node().digest() + 1,
    );
    let foreign_session =
        crate::lifecycle::WorthUiActiveApplicationSessionIdentity::from_host_session_value(99_001);
    let foreign_generation = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
        foreign_session,
        session.generation_identity(),
    );
    let foreign = context(
        identities,
        foreign_session,
        &foreign_generation,
        live.graph_node().digest() + 2,
    );
    let mut state = UiMountedAppearanceFrameState::default();
    state.retain_projection_for_test(&live, projection.clone());
    state.retain_projection_for_test(&stale_generation_entry, projection.clone());
    state.retain_projection_for_test(&changed_incarnation_entry, projection.clone());
    state.retain_projection_for_test(&retired, projection.clone());
    state.retain_projection_for_test(&foreign, projection);

    state.prune_to_current_nodes(session_identity, &generation, &[node_for(&live)]);

    assert_eq!(state.membership_counts(), (1, 0, 0));
    assert_eq!(
        state
            .retained_entry_for_test(&live)
            .expect("live entry remains retained")
            .key,
        super::state_key(&live)
    );
    assert!(state.retained_entry_for_test(&retired).is_none());
    assert!(state.retained_entry_for_test(&foreign).is_none());
    assert!(state
        .retained_entry_for_test(&stale_generation_entry)
        .is_none());
    assert!(state
        .retained_entry_for_test(&changed_incarnation_entry)
        .is_none());
    state.reserve(&retired).unwrap();
    assert_eq!(state.membership_counts(), (1, 1, 0));

    let _ = stale_source.shutdown();
    let _ = session.shutdown();
}

#[test]
fn appearance_state_duplicate_stage_and_lower_release_preserve_replacement_capacity() {
    let (session, binding, _target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let identities = context_identities();
    let first = context(identities, session_identity, &generation, 1);
    let second = context(identities, session_identity, &generation, 2);

    let mut state = UiMountedAppearanceFrameState::default();
    state.reserve(&first).unwrap();
    state.reserve(&first).unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                first.clone(),
                projection.clone(),
            ),
        )
        .unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                first.clone(),
                projection.clone(),
            ),
        )
        .unwrap();
    assert_eq!(state.membership_counts(), (0, 0, 1));

    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state.lower(presentation);
    assert_eq!(records.len(), 1);
    let crate::runtime::appearance::UiAppearanceInspectionRecord::Denial {
        context,
        denial: crate::runtime::appearance::UiAppearanceInspectionDenial::MountLowering,
        ..
    } = &records[0]
    else {
        panic!("an already-resolved attempt with omitted allocation must deny at lowering");
    };
    assert!(context.theme_slots_compared() > 0);
    assert_eq!(state.membership_counts(), (0, 0, 0));

    state.retain_projection_for_test(&first, projection.clone());
    state.reserve(&first).unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                first.clone(),
                projection.clone(),
            ),
        )
        .unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                first.clone(),
                projection,
            ),
        )
        .unwrap();
    assert_eq!(state.membership_counts(), (0, 0, 1));
    state.lower(presentation);
    assert_eq!(state.membership_counts(), (1, 0, 0));
    assert!(state.retained_entry_for_test(&first).is_some());

    state.reserve(&second).unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                second.clone(),
                crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ),
        )
        .unwrap();
    state.lower(presentation);
    assert_eq!(state.membership_counts(), (1, 0, 0));
    state.reserve(&second).unwrap();
    assert_eq!(state.membership_counts(), (1, 1, 0));

    let _ = session.shutdown();
}

#[test]
fn duplicate_denial_on_retained_entry_emits_latest_denial_and_preserves_predecessor_facts() {
    let (session, binding, _target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let identities = context_identities();
    let retained = context(identities, session_identity, &generation, 77);
    let mut retained_fixture =
        crate::mounting::projection::appearance::mounted_sidecar_with_retained_facts_for_test();
    let predecessor_facts = retained_fixture
        .sidecar
        .current()
        .cloned()
        .expect("retained fixture has mounted predecessor facts");
    let predecessor_receipts = retained_fixture.sidecar.current_node_receipts();

    let mut state = UiMountedAppearanceFrameState::default();
    state.retain_projection_for_test(&retained, projection.clone());
    state.replace_sidecar_for_test(&retained, std::mem::take(&mut retained_fixture.sidecar));
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::resolved(
                retained.clone(),
                projection,
            ),
        )
        .unwrap();
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                retained.clone(),
                crate::runtime::appearance::UiAppearanceInspectionDenial::MountLowering,
            ),
        )
        .unwrap();

    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state.lower(presentation);
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records.as_slice(),
        [
            crate::runtime::appearance::UiAppearanceInspectionRecord::Denial {
                denial: crate::runtime::appearance::UiAppearanceInspectionDenial::MountLowering,
                ..
            }
        ]
    ));
    let retained_after = state
        .retained_entry_for_test(&retained)
        .expect("denied successor restores retained predecessor");
    assert_eq!(retained_after.sidecar.current(), Some(&predecessor_facts));
    assert_eq!(
        retained_after.sidecar.current_node_receipts(),
        predecessor_receipts
    );
    assert_eq!(state.membership_counts(), (1, 0, 0));

    let _ = session.shutdown();
}

#[test]
fn one_staged_entry_lowers_without_scanning_or_disturbing_many_retained_entries() {
    let (session, binding, _target, vector, theme) =
        crate::runtime::appearance::projection_test_inputs();
    let projection = crate::runtime::appearance::UiAppearanceResolver::new()
        .resolve_node(
            session.graph().snapshot(),
            session.capabilities(),
            &binding,
            &vector,
            &theme,
        )
        .unwrap();
    let session_identity = session.session_identity();
    let generation = session.active_generation_identity();
    let identities = context_identities();
    let mut state = UiMountedAppearanceFrameState::default();
    for ordinal in 10..42 {
        let retained = context(identities, session_identity, &generation, ordinal);
        state.retain_projection_for_test(&retained, projection.clone());
    }
    let staged = context(identities, session_identity, &generation, 100);
    state
        .stage(
            crate::runtime::appearance::UiAppearanceProjectionAttempt::denied(
                staged,
                crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ),
        )
        .unwrap();

    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let records = state.lower(presentation);
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0],
        crate::runtime::appearance::UiAppearanceInspectionRecord::Denial {
            denial: crate::runtime::appearance::UiAppearanceInspectionDenial::Basis,
            ..
        }
    ));
    assert_eq!(state.membership_counts(), (32, 0, 0));
    assert!(state
        .retained_entry_for_test(&context(identities, session_identity, &generation, 10))
        .is_some());
    let _ = session.shutdown();
}

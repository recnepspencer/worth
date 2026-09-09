//! Complete command retention is independent of the current damage selection.
use super::super::UiNativeTextAtlasTransaction;
use super::*;
use crate::mounting::qualified_text_test_support::inert_qualified_layout;
use worth_ui_host_contract::*;

#[test]
fn narrow_damage_preserves_complete_selected_command_and_rejects_filtered_reuse() {
    use crate::certification_support::{
        initial_presentation_mechanics_for_certification,
        semantic_text_projection_for_certification, UiSemanticTextProjectionCertificationMutation,
    };
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let binding = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let initial = initial_presentation_mechanics_for_certification(&projection, binding);
    let layout = inert_qualified_layout("ONLINE");
    let dpi = UiMountedEventTimeDpiAuthority::from_requirement(binding).unwrap();
    let initial_work = UiMountedPresentationWorkView::Initial(&initial);
    let selected = mounted_semantic_text(initial_work);
    let (command, mechanic) = selected.mechanics[0];
    let UiNativeTextPresentationPreparation::Prepared(complete) =
        prepare_mounted_semantic_text(initial_work, dpi, |_| Some(layout.as_ref())).unwrap()
    else {
        panic!("qualified initial text prepares");
    };
    let join = MountedTextDemandJoin {
        dpi,
        lane: UiGlyphRasterLane::Ordinary,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(initial.damage()),
        resolve: |_| Some(layout.as_ref()),
        _layout: std::marker::PhantomData,
    };
    let filtered = prepare_demands(&selected.mechanics, &join).unwrap();
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let successor = repaint(mechanic, successor_frame, &layout);
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: initial.affinity().successor(),
        successor: successor_frame,
        surface: binding.semantic_surface(),
        binding: binding.binding(),
        content: initial.affinity().content(),
        baseline: binding.baseline(),
        changes: vec![UiMountedPaintCommandChange::replacement(
            command,
            UiMountedPaintCommand::SemanticText {
                identity: command,
                mechanic: successor.clone(),
            },
        )],
        nodes: Vec::new(),
        order: Vec::new(),
        order_integrity: initial.order_integrity(),
        damage: initial.damage().to_vec(),
        auxiliary: None,
        production_cost: Default::default(),
    });
    let successor_work = UiMountedPresentationWorkView::Delta(&delta);
    let basis = |work| {
        UiMountedTextForegroundPresentationBasis::from_work(
            work,
            binding,
            dpi.dpi_milli(),
            None,
            presentation_damage_digest(work),
        )
    };
    let receipt = |demand| {
        UiMountedTextForegroundReuseReceipt::from_prepared(
            command,
            mechanic,
            demand,
            &[],
            basis(initial_work),
        )
    };
    let reused = prepare_from_foreground_reuse(
        successor_work,
        &[receipt(&complete.demand_batches()[0])],
        basis(successor_work),
        |_| Some(layout.as_ref()),
    )
    .expect("complete receipt admits paint-only reuse");
    assert!(reused.foreground_reused());
    assert!(
        prepare_from_foreground_reuse(
            successor_work,
            &[receipt(&filtered.demands[0])],
            basis(successor_work),
            |_| Some(layout.as_ref())
        )
        .is_none(),
        "filtered demand must rederive complete retention"
    );

    for damage in [
        vec![UiMountedLogicalDamage::from_runtime_mounting(
            UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                x: 500.0,
                y: 500.0,
                width: 1.0,
                height: 1.0,
                coordinate_space: mechanic.bounds().coordinate_space(),
            })
            .unwrap(),
        )],
        Vec::new(),
    ] {
        let changed =
            UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
                predecessor: initial.affinity().successor(),
                successor: successor_frame,
                surface: binding.semantic_surface(),
                binding: binding.binding(),
                content: initial.affinity().content(),
                baseline: binding.baseline(),
                changes: delta.changes().to_vec(),
                nodes: Vec::new(),
                order: Vec::new(),
                order_integrity: delta.order_integrity(),
                damage,
                auxiliary: None,
                production_cost: Default::default(),
            });
        let UiNativeTextPresentationPreparation::Prepared(prepared) =
            prepare_mounted_semantic_text(
                UiMountedPresentationWorkView::Delta(&changed),
                dpi,
                |_| Some(layout.as_ref()),
            )
            .unwrap()
        else {
            panic!("selected command prepares independently of replay damage");
        };
        assert_eq!(prepared.pin_commands(), &[command]);
        assert!(
            !prepared.pin_set_complete(),
            "one changed command cannot replace surface pins"
        );
        assert_eq!(
            prepared.demand_batches()[0].scope(),
            UiGlyphRasterDemandScope::CompleteLayout
        );
        assert_eq!(prepared.glyph_runs().len(), 6);
        assert_eq!(prepared.glyph_runs(), reused.glyph_runs());
        let ranges = prepared
            .glyph_runs()
            .iter()
            .map(|run| run.original_range())
            .collect::<Vec<_>>();
        assert_eq!(
            ranges,
            (0..6)
                .map(|i| UiTextOriginalRange::new(i, i + 1).unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(prepared.planning_inspection().unwrap().demand_batches(), 1);
        assert_eq!(prepared.performed_layout_work(), [0; 17]);
    }
}

fn repaint(
    previous: &UiMountedSemanticTextMechanic,
    frame: UiMountedFrameIdentity,
    layout: &worth_ui_text::UiQualifiedTextLayout,
) -> UiMountedSemanticTextMechanic {
    UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
        UiMountedSemanticTextCompletionInput {
            content_generation: previous.content_generation(),
            frame,
            surface: previous.surface(),
            binding: previous.binding(),
            mounted_instance: previous.mounted_instance(),
            node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(previous.mounted_instance()),
            allocation_basis: previous.allocation_basis(),
            bounds: previous.bounds(),
            clip_bounds: previous.clip_bounds(),
            origin_x: previous.origin_x(),
            origin_y: previous.origin_y(),
            text: previous.retained_text_for_runtime_mounting(),
            layout: layout.view(),
            slot: previous.slot(),
            collection_row: previous.collection_row().cloned(),
            foregrounds: previous
                .foregrounds()
                .iter()
                .map(|span| {
                    UiMountedTextForegroundSpan::from_runtime_mounting(
                        span.original_range(),
                        UiMountedRgba8::new(0, 255, 0, 255),
                        span.identity(),
                    )
                })
                .collect::<Vec<_>>()
                .into(),
            profile: previous.profile(),
            layer_semantic_order: previous.layer_semantic_order(),
            capability_generation: previous.capability_generation(),
            capability_profile_digest: previous.capability_profile_digest(),
        },
    )
    .unwrap()
}

#[test]
fn filtered_owned_demand_cannot_be_promoted_by_a_borrowed_scope_flag() {
    use crate::certification_support::{
        initial_presentation_mechanics_for_certification,
        semantic_text_projection_for_certification, UiSemanticTextProjectionCertificationMutation,
    };
    let projection = semantic_text_projection_for_certification(
        UiSemanticTextProjectionCertificationMutation::Exact,
    );
    let binding = UiMountedSurfaceBindingRequirement::new(
        projection.surface(),
        UiHostSurfaceIdentity::mint_unbound().unwrap(),
        projection.binding(),
        WorthUiHostCapabilityObservationGeneration::new(7),
        11,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let initial = initial_presentation_mechanics_for_certification(&projection, binding);
    let layout = inert_qualified_layout("ONLINE");
    let work = mounted_semantic_text(UiMountedPresentationWorkView::Initial(&initial));
    let join = MountedTextDemandJoin {
        dpi: UiMountedEventTimeDpiAuthority::from_requirement(binding).unwrap(),
        lane: UiGlyphRasterLane::Ordinary,
        selection: worth_ui_text::UiGlyphRasterDemandSelection::LogicalDamage(initial.damage()),
        resolve: |_| Some(layout.as_ref()),
        _layout: std::marker::PhantomData,
    };
    let demands = prepare_demands(&work.mechanics, &join).unwrap();
    let UiNativeTextPresentationPreparation::Prepared(prepared) =
        inspect_demand_boundary(&work, demands)
    else {
        panic!("ordinary text demand must prepare");
    };
    let mut cache = worth_ui_text::UiGlyphRasterCache::default();
    let mut transaction =
        UiNativeTextAtlasTransaction::prepare(&prepared, |_| Some(layout.as_ref()), &mut cache)
            .unwrap();
    transaction.with_mounted_work(
        UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]),
        &[],
        |work| {
            let demand = work.demands()[0];
            assert_eq!(demand.scope(), UiGlyphRasterDemandScope::DamageFiltered);
            let forged = UiGlyphRasterDemandBatchView::from_text_mechanics(
                UiGlyphRasterDemandBatchViewInput {
                    identity: demand.identity(),
                    layout: demand.layout_identity(),
                    dpi_milli: demand.dpi_milli(),
                    text_scale: demand.text_scale_generation(),
                    lane: demand.lane(),
                    scope: UiGlyphRasterDemandScope::CompleteLayout,
                    records: demand.records(),
                },
            )
            .unwrap();
            assert_eq!(
                work.validate_complete_demand(
                    prepared.pin_commands()[0],
                    forged,
                    work.glyph_runs()
                ),
                Err(UiMountedTextDemandValidationDenial::IncompleteDemand)
            );
        },
    );
    assert_eq!(transaction.cache_len(), 0);
}

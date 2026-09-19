//! Exact foreground insertion and removal through retained presentation staging.
use super::foreground_coverage_test_world::CoverageWorld;
use std::collections::HashMap;
use worth_ui_host_contract::*;
use worth_ui_host_native::UiNativeTextForegroundAtlasModel;

#[test]
fn finalized_foreground_insert_and_remove_follow_exact_text_commands() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first = CoverageWorld::with_commands(
        "W\tW\tW",
        instance,
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[(UiSemanticTextSlot::Value, 0.0)],
    );
    let inserted = CoverageWorld::with_commands(
        "W\tW\tW",
        instance,
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        Some(&first),
        &[
            (UiSemanticTextSlot::Value, 0.0),
            (UiSemanticTextSlot::Posture, 200.0),
        ],
    );
    let removed = CoverageWorld::with_commands(
        "W\tW\tW",
        instance,
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        Some(&inserted),
        &[(UiSemanticTextSlot::Value, 0.0)],
    );
    let mut model = UiNativeTextForegroundAtlasModel::new();
    first.with_native(first.attempt, |view| {
        let candidates = finalize_all(&mut model, &first, view);
        model
            .initialize_presentation_coverages(candidates, view, [400, 48])
            .unwrap();
    });
    inserted.with_native(inserted.attempt, |view| {
        let candidates = finalize_all(&mut model, &inserted, view);
        let coverage = model
            .prepare_presentation_coverage_changes(
                candidates,
                &inserted.fragment,
                &delta(&first, &inserted),
                view,
            )
            .unwrap();
        assert_eq!(coverage.len(), 6, "both exact commands retain images");
    });
    removed.with_native(removed.attempt, |view| {
        let candidates = finalize_all(&mut model, &removed, view);
        let coverage = model
            .prepare_presentation_coverage_changes(
                candidates,
                &removed.fragment,
                &delta(&inserted, &removed),
                view,
            )
            .unwrap();
        assert_eq!(
            coverage.as_ref(),
            removed.expected_images(0, [0, 0, 400, 48]),
            "removed command leaves no retained foreground coverage"
        );
    });
}

fn finalize_all(
    model: &mut UiNativeTextForegroundAtlasModel,
    world: &CoverageWorld,
    view: &UiMountedFrameConsumptionView<'_>,
) -> Vec<worth_ui_host_native::UiNativeTextForegroundCoverageCertification> {
    world
        .foregrounds
        .iter()
        .enumerate()
        .map(|(index, foreground)| {
            if index == 0 {
                model
                    .rasterize_with_simulated_submission(
                        &world.fragment,
                        view,
                        foreground,
                        [400, 48],
                    )
                    .unwrap()
            } else {
                model
                    .finalize_existing(&world.fragment, view, foreground, [400, 48])
                    .unwrap()
            }
        })
        .collect()
}

fn delta(predecessor: &CoverageWorld, successor: &CoverageWorld) -> UiMountedPresentationDelta {
    let old = commands(predecessor);
    let new = commands(successor);
    let old_by_id = old
        .iter()
        .map(|command| (command.identity(), command))
        .collect::<HashMap<_, _>>();
    let new_by_id = new
        .iter()
        .map(|command| (command.identity(), command))
        .collect::<HashMap<_, _>>();
    let mut changes = new
        .iter()
        .map(|command| match old_by_id.get(&command.identity()) {
            Some(_) => {
                UiMountedPaintCommandChange::replacement(command.identity(), command.clone())
            }
            None => UiMountedPaintCommandChange::Insert(command.clone()),
        })
        .collect::<Vec<_>>();
    changes.extend(
        old.iter()
            .filter(|command| !new_by_id.contains_key(&command.identity()))
            .map(|command| UiMountedPaintCommandChange::Remove(command.identity())),
    );
    let old_order = old
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
    let new_order = new
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
    let mut order = old_order
        .iter()
        .filter(|identity| !new_order.contains(identity))
        .copied()
        .map(UiMountedPaintOrderEdit::remove)
        .collect::<Vec<_>>();
    for (index, identity) in new_order.iter().copied().enumerate() {
        if !old_order.contains(&identity) {
            order.push(UiMountedPaintOrderEdit::place_after(
                identity,
                index.checked_sub(1).map(|previous| new_order[previous]),
            ));
        }
    }
    let affinity = successor.fragment.presentation_affinity();
    UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: predecessor.fragment.work().successor().frame(),
        successor: affinity.successor(),
        surface: affinity.surface(),
        binding: affinity.binding(),
        content: affinity.content(),
        baseline: affinity.baseline(),
        changes,
        nodes: vec![],
        order,
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&new_order),
        damage: old
            .iter()
            .chain(&new)
            .map(|command| UiMountedLogicalDamage::from_runtime_mounting(command.bounds()))
            .collect(),
        auxiliary: None,
        production_cost: Default::default(),
    })
    .with_successor_receipt_affinity(affinity.receipt_affinity())
}

fn commands(world: &CoverageWorld) -> Vec<UiMountedPaintCommand> {
    world
        .fragment
        .text_candidates()
        .iter()
        .cloned()
        .map(|mechanic| UiMountedPaintCommand::SemanticText {
            identity: UiMountedPaintCommandIdentity::semantic_text(&mechanic),
            mechanic,
        })
        .collect()
}

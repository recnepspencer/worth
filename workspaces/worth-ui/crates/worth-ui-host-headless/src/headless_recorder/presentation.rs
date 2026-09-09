use std::collections::{HashMap, HashSet};

use worth_ui_host_contract::{
    UiHostPresentationCostInput, UiHostPresentationCostReport, UiHostSurfacePresentationDenial,
    UiMountedFrameConsumptionView, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiMountedPaintOrderIdentity, UiMountedPresentationWorkView,
};

pub(super) fn production_cost(
    work: UiMountedPresentationWorkView<'_>,
) -> worth_ui_host_contract::UiMountedPresentationProductionCost {
    match work {
        UiMountedPresentationWorkView::Initial(work) => work.production_cost(),
        UiMountedPresentationWorkView::Delta(work) => work.production_cost(),
        UiMountedPresentationWorkView::Reconstruction(work) => work.production_cost(),
        UiMountedPresentationWorkView::Sample(work) => work.production_cost(),
        UiMountedPresentationWorkView::Unchanged(work) => work.production_cost(),
    }
}

use super::recorded_frame::UiHeadlessRecordedFrame;
use super::retained_order::UiHeadlessRetainedOrder;
use super::{UiHeadlessRecorderCapacity, UiHeadlessRetainedPresentation};
use crate::headless_translation::translate_headless_frame;

mod delta;
mod node_delta;

pub(super) fn apply_work(
    view: &UiMountedFrameConsumptionView<'_>,
    capacity: UiHeadlessRecorderCapacity,
    current: &mut Option<UiHeadlessRetainedPresentation>,
) -> Result<
    (
        Option<UiHeadlessRecordedFrame>,
        worth_ui_retained_order::UiRetainedOrderCost,
    ),
    UiHostSurfacePresentationDenial,
> {
    match view.presentation_work() {
        UiMountedPresentationWorkView::Initial(initial) => {
            if current.is_some() {
                return Err(UiHostSurfacePresentationDenial::MalformedProjection);
            }
            let commands = validated_initial_commands(initial)?;
            let transcript = translate_headless_frame(
                view,
                initial.projection(),
                capacity,
                initial.order(),
                initial.damage(),
            )?;
            let recorded = UiHeadlessRecordedFrame::complete(transcript);
            let order = UiHeadlessRetainedOrder::initial(
                initial.order(),
                initial.order_integrity(),
                capacity.mechanics_per_frame(),
            )
            .map_err(|_| malformed())?;
            let order_cost = order.take_cost();
            *current = Some(UiHeadlessRetainedPresentation::initial(
                view.frame(),
                view.surface(),
                view.binding(),
                initial.affinity().content(),
                initial.affinity().baseline(),
                commands,
                order,
                initial_node_positions(initial.projection())?,
                initial_nodes_by_position(initial.projection())?,
                initial.auxiliary().clone(),
            ));
            Ok((Some(recorded), order_cost))
        }
        UiMountedPresentationWorkView::Delta(work) => {
            let Some(current) = current.as_mut() else {
                return Err(malformed());
            };
            current.order.take_cost();
            let recorded = delta::apply(view, capacity, current, work)?;
            Ok((Some(recorded), current.order.take_cost()))
        }
        UiMountedPresentationWorkView::Reconstruction(work) => {
            if current.is_some() {
                return Err(malformed());
            }
            let commands = validated_complete_commands(
                work.commands(),
                work.projection(),
                work.order(),
                work.order_integrity(),
            )?;
            let transcript = translate_headless_frame(
                view,
                work.projection(),
                capacity,
                work.order(),
                work.damage(),
            )?;
            let recorded = UiHeadlessRecordedFrame::complete(transcript);
            let order = UiHeadlessRetainedOrder::initial(
                work.order(),
                work.order_integrity(),
                capacity.mechanics_per_frame(),
            )
            .map_err(|_| malformed())?;
            let order_cost = order.take_cost();
            let mut retained = UiHeadlessRetainedPresentation::initial(
                view.frame(),
                view.surface(),
                view.binding(),
                work.affinity().content(),
                work.affinity().baseline(),
                commands,
                order,
                initial_node_positions(work.projection())?,
                initial_nodes_by_position(work.projection())?,
                work.auxiliary().clone(),
            );
            retained
                .install_reconstruction_sample_overrides(work.sample_overrides(), work.damage())?;
            *current = Some(retained);
            Ok((Some(recorded), order_cost))
        }
        UiMountedPresentationWorkView::Sample(sample) => {
            let current = current.as_mut().ok_or_else(malformed)?;
            current.apply_sample(sample)?;
            Ok((
                None,
                worth_ui_retained_order::UiRetainedOrderCost::default(),
            ))
        }
        UiMountedPresentationWorkView::Unchanged(unchanged) => {
            let current = current.as_mut().ok_or_else(malformed)?;
            if !affinity_matches(current, unchanged.affinity()) {
                return Err(malformed());
            }
            current.frame = view.frame();
            current.content = unchanged.affinity().content();
            current.receipt_affinity = unchanged.affinity().receipt_affinity();
            Ok((
                None,
                worth_ui_retained_order::UiRetainedOrderCost::default(),
            ))
        }
    }
}

fn initial_node_positions(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> Result<
    HashMap<worth_ui_host_contract::UiMountedInstanceIdentity, u64>,
    UiHostSurfacePresentationDenial,
> {
    projection
        .nodes()
        .iter()
        .map(|node| Ok((node.mounted_instance(), node.authored_position())))
        .collect()
}

fn initial_nodes_by_position(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> Result<
    HashMap<u64, worth_ui_host_contract::UiMountedInstanceIdentity>,
    UiHostSurfacePresentationDenial,
> {
    projection
        .nodes()
        .iter()
        .map(|node| Ok((node.authored_position(), node.mounted_instance())))
        .collect()
}

fn affinity_matches(
    current: &UiHeadlessRetainedPresentation,
    affinity: worth_ui_host_contract::UiMountedPresentationAffinity,
) -> bool {
    affinity.predecessor() == Some(current.frame)
        && affinity.surface() == current.surface
        && affinity.binding() == current.binding
        && affinity.baseline() == current.baseline
}

fn malformed() -> UiHostSurfacePresentationDenial {
    UiHostSurfacePresentationDenial::MalformedProjection
}

fn validated_initial_commands(
    initial: &worth_ui_host_contract::UiMountedPresentationInitial,
) -> Result<
    super::retained_command_store::UiHeadlessRetainedCommandStore,
    UiHostSurfacePresentationDenial,
> {
    validated_complete_commands(
        initial.commands(),
        initial.projection(),
        initial.order(),
        initial.order_integrity(),
    )
}

fn validated_complete_commands(
    source: &[UiMountedPaintCommand],
    projection: &worth_ui_host_contract::UiMountedProjectionView,
    order: &[UiMountedPaintOrderIdentity],
    integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity,
) -> Result<
    super::retained_command_store::UiHeadlessRetainedCommandStore,
    UiHostSurfacePresentationDenial,
> {
    let mut commands =
        super::retained_command_store::UiHeadlessRetainedCommandStore::with_capacity(source.len());
    for command in source {
        if commands
            .insert(command.identity(), command.clone())
            .is_some()
        {
            return Err(UiHostSurfacePresentationDenial::MalformedProjection);
        }
    }
    validate_initial_projection(&commands, projection)?;
    validate_order(&commands, order, integrity)?;
    Ok(commands)
}

fn validate_initial_projection(
    commands: &super::retained_command_store::UiHeadlessRetainedCommandStore,
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let expected_count = projection.filled_rects().rows().len()
        + projection.portal_overlays().rows().len()
        + projection.semantic_text().rows().len();
    if commands.len() != expected_count {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    let aligned = commands.values().all(|command| match command {
        UiMountedPaintCommand::FilledRect { identity, mechanic } => {
            *identity == UiMountedPaintCommandIdentity::filled_rect(mechanic)
                && projection.filled_rects().rows().contains(mechanic)
        }
        UiMountedPaintCommand::SemanticText { identity, mechanic } => {
            *identity == UiMountedPaintCommandIdentity::semantic_text(mechanic)
                && projection.semantic_text().rows().contains(mechanic)
        }
        UiMountedPaintCommand::PortalOverlay { identity, mechanic } => {
            *identity == UiMountedPaintCommandIdentity::portal_overlay(mechanic)
                && projection.portal_overlays().rows().contains(mechanic)
        }
    });
    if !aligned {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    Ok(())
}

fn validate_order(
    commands: &super::retained_command_store::UiHeadlessRetainedCommandStore,
    order: &[UiMountedPaintOrderIdentity],
    integrity: worth_ui_host_contract::UiMountedPaintOrderIntegrity,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let ordered = order
        .iter()
        .map(|identity| identity.command())
        .collect::<HashSet<_>>();
    let commanded = commands.identities().collect::<HashSet<_>>();
    if ordered.len() != order.len() || ordered != commanded || !integrity.admits(order) {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    Ok(())
}

pub(super) fn work_cost(
    work: UiMountedPresentationWorkView<'_>,
) -> Result<UiHostPresentationCostReport, UiHostSurfacePresentationDenial> {
    let (presented_surfaces, rows, bytes, delta_rows, draw_mutations, order_mutations, damage) =
        match work {
            UiMountedPresentationWorkView::Initial(initial) => (
                1,
                initial.commands().len(),
                std::mem::size_of_val(initial.commands()),
                0,
                0,
                0,
                initial.damage().len(),
            ),
            UiMountedPresentationWorkView::Delta(delta) => (
                1,
                delta.changes().len() + delta.nodes().len(),
                std::mem::size_of_val(delta.changes()) + std::mem::size_of_val(delta.nodes()),
                delta.changes().len()
                    + delta.nodes().len()
                    + delta.order().len()
                    + delta.damage().len(),
                delta.changes().len(),
                delta.order().len(),
                delta.damage().len(),
            ),
            UiMountedPresentationWorkView::Reconstruction(work) => (
                1,
                work.commands().len(),
                std::mem::size_of_val(work.commands()),
                work.commands().len() + work.order().len() + work.damage().len(),
                work.commands().len(),
                work.order().len(),
                work.damage().len(),
            ),
            UiMountedPresentationWorkView::Sample(sample) => (
                1,
                sample.changes().len(),
                std::mem::size_of_val(sample.changes()),
                sample.changes().len() + sample.damage().len(),
                sample.changes().len(),
                0,
                sample.damage().len(),
            ),
            UiMountedPresentationWorkView::Unchanged(_) => (0, 0, 0, 0, 0, 0, 0),
        };
    Ok(UiHostPresentationCostReport::from_adapter(
        UiHostPresentationCostInput {
            presented_surfaces,
            translated_rows: u64::try_from(rows)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            translated_bytes: u64::try_from(bytes)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            native_resource_cache_hits: 0,
            native_resource_cache_misses: 0,
            asynchronous_handoffs: 0,
            delta_rows_carried: u64::try_from(delta_rows)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            draw_list_mutations: u64::try_from(draw_mutations)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            order_mutations: u64::try_from(order_mutations)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            logical_damage_regions: u64::try_from(damage)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            ..Default::default()
        },
    ))
}

pub(super) fn add_order_cost(
    base: UiHostPresentationCostReport,
    order: worth_ui_retained_order::UiRetainedOrderCost,
) -> UiHostPresentationCostReport {
    base.checked_add(UiHostPresentationCostReport::from_adapter(
        UiHostPresentationCostInput {
            order_index_lookups: order.identity_lookups(),
            order_index_node_touches: order.node_touches(),
            order_index_rotations: order.rotations(),
            order_index_high_water: order.high_water_entries(),
            ..Default::default()
        },
    ))
    .expect("profile-bounded retained-order evidence cannot overflow")
}

#[cfg(test)]
#[path = "presentation_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "presentation_reconstruction_tests.rs"]
mod reconstruction_tests;

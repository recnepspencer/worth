use super::super::super::{
    delta::settle_staged_delta, reserve_presentation_owners, settle_port_result,
    UiNativePendingExternalObligation, UiNativePendingSurfaceSettlement,
    UiNativePresentationEffects, UiNativePresentationFailure, UiNativePresentationPortFailure,
};
use super::*;

#[path = "delta_transaction_tests/sample_override.rs"]
mod sample_override;

#[path = "delta_transaction_tests/identity_replacement.rs"]
mod identity_replacement;

struct UnsettledPresentation;

impl UiNativePendingExternalObligation for UnsettledPresentation {
    fn poll_observation(
        &mut self,
        basis: crate::native::physical_work_signal::UiNativePhysicalSignalExternalBasis,
        _device: Option<&wgpu::Device>,
    ) -> crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation {
        basis.observe(crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Pending)
    }
}

#[test]
fn exact_delta_updates_draw_order_damage_and_replay_without_retained_scans() {
    let world = DrawListWorld::new();
    let initial_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial_rows = [
        world.rect(
            initial_frame,
            world.first,
            0.0,
            UiMountedRgba8::new(20, 30, 40, 255),
        ),
        world.rect(
            initial_frame,
            world.second,
            80.0,
            UiMountedRgba8::new(50, 60, 70, 255),
        ),
    ];
    let initial = world.initial(initial_frame, initial_rows);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let successor_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let replaced = world.rect(
        successor_frame,
        world.first,
        0.0,
        UiMountedRgba8::new(90, 100, 110, 255),
    );
    let inserted = world.rect(
        successor_frame,
        world.third,
        160.0,
        UiMountedRgba8::new(120, 130, 140, 255),
    );
    let replaced_command = command(replaced);
    let inserted_command = command(inserted);
    let removed_identity = UiMountedPaintOrderIdentity::for_command(
        UiMountedPaintCommand::identity(&command(initial_rows[1])),
    );
    let retained_identity = UiMountedPaintOrderIdentity::for_command(replaced_command.identity());
    let inserted_identity = UiMountedPaintOrderIdentity::for_command(inserted_command.identity());
    let damage = [
        initial_rows[0].bounds(),
        replaced.bounds(),
        initial_rows[1].bounds(),
        inserted.bounds(),
    ]
    .map(UiMountedLogicalDamage::from_runtime_mounting);
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: initial_frame,
        successor: successor_frame,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![
            UiMountedPaintCommandChange::replacement(
                command(initial_rows[0]).identity(),
                replaced_command,
            ),
            UiMountedPaintCommandChange::Remove(removed_identity.command()),
            UiMountedPaintCommandChange::Insert(inserted_command),
        ],
        nodes: Vec::new(),
        order: vec![
            UiMountedPaintOrderEdit::remove(removed_identity),
            UiMountedPaintOrderEdit::place_after(inserted_identity, Some(retained_identity)),
        ],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&[
            retained_identity,
            inserted_identity,
        ]),
        damage: damage.to_vec(),
        auxiliary: None,
        production_cost: Default::default(),
    });

    let (staged, undo) = retained.stage_delta(&delta, &[]).unwrap();
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        vec![retained_identity, inserted_identity]
    );
    let denied = settle_staged_delta(
        &mut retained,
        undo,
        UiNativePresentationEffects::default(),
        Err(UiNativePresentationFailure::BeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined,
        )),
    );
    assert!(matches!(
        denied,
        Err(UiNativePresentationFailure::BeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined
        ))
    ));
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        vec![retained_identity, removed_identity]
    );
    assert_eq!(
        retained.command(removed_identity.command()),
        Some(&initial.commands()[1])
    );
    let (_, indeterminate_undo) = retained.stage_delta(&delta, &[]).unwrap();
    let mut resources = crate::native::UiNativeResourceRegistry::new();
    let mut physical_signal =
        crate::native::physical_work_signal::UiNativePhysicalSignalOwner::new();
    let owners = reserve_presentation_owners(
        &mut resources,
        &mut physical_signal,
        crate::native::physical_work_signal::UiNativePhysicalPresentationBasis::test(),
    )
    .unwrap_or_else(|_| panic!("an empty registry admits the staged presentation obligations"));
    let indeterminate = settle_port_result(
        &mut resources,
        &mut physical_signal,
        owners,
        Err(UiNativePresentationPortFailure::ReadbackUnsettled(
            Box::new(UnsettledPresentation),
        )),
    );
    let settled = settle_staged_delta(
        &mut retained,
        indeterminate_undo,
        UiNativePresentationEffects::default(),
        indeterminate,
    );
    let Err(UiNativePresentationFailure::Pending(mut pending)) = settled else {
        panic!("an unsettled external presentation remains indeterminate");
    };
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        vec![retained_identity, inserted_identity]
    );
    assert_eq!(
        retained
            .command(inserted_identity.command())
            .map(UiMountedPaintCommand::identity),
        Some(inserted_identity.command())
    );
    assert_eq!(resources.current().readback_buffers, 1);
    assert_eq!(resources.current().pending_submissions, 1);
    let due = physical_signal
        .next_due_tick()
        .expect("pending presentation must retain one Signal-owned poll wake");
    physical_signal
        .advance_clock_to(due)
        .expect("the exact pending poll wake must become ready");
    let token = physical_signal
        .take_ready_presentation(pending.physical_work(), pending.physical_token())
        .unwrap()
        .current();
    assert!(matches!(
        physical_signal.reconcile(
            token.observe(
                crate::native::physical_work_signal::UiNativePhysicalSignalStatus::Completed,
            )
        ),
        crate::native::physical_work_signal::UiNativePhysicalSignalSettlement::Completed
    ));
    let Some(UiNativePendingSurfaceSettlement::Delta(settlement)) = pending.take_settlement()
    else {
        panic!("the pending delta retains its exact rollback authority");
    };
    settlement
        .rollback(&mut retained)
        .expect("completion without a physical observation restores the predecessor");
    pending.release(&mut resources);
    assert!(resources.current().is_zero());
    assert_eq!(
        retained.order.ordered().collect::<Vec<_>>(),
        vec![retained_identity, removed_identity]
    );
    assert_eq!(
        retained.command(removed_identity.command()),
        Some(&initial.commands()[1])
    );
    let plan = retained.apply_delta(&delta).unwrap();
    assert_eq!(plan, staged);
    assert_eq!(plan.baseline_rgba8, [0, 0, 0, 0]);
    assert_eq!(plan.regions.len(), 3);
    assert_eq!(
        plan.regions
            .iter()
            .flat_map(|region| region.replay.iter().copied())
            .collect::<std::collections::HashSet<_>>(),
        [retained_identity.command(), inserted_identity.command()]
            .into_iter()
            .collect()
    );
    assert_eq!(plan.counters.draw_mutations, 3);
    assert_eq!(plan.counters.order_mutations, 2);
    assert!(plan.counters.order_index_lookups > 0);
    assert!(plan.counters.order_index_node_touches > 0);
    assert!(plan.counters.order_index_rotations <= 4);
    assert_eq!(plan.counters.order_index_high_water, 2);
    assert_eq!(plan.counters.damage_rows_carried, 4);
    assert_eq!(plan.counters.damage_regions, 3);
    assert_eq!(
        plan.counters.damage_region_command_checks,
        plan.regions
            .iter()
            .map(|region| region.replay.len() as u64)
            .sum::<u64>()
    );
    assert_eq!(plan.counters.retained_command_scans, 0);
    assert!(retained.command(removed_identity.command()).is_none());
}

#[test]
fn full_capacity_swap_releases_predecessor_membership_before_successor_claims() {
    const CAPACITY: usize =
        crate::native_profile::UiNativeMechanicsCapacities::QUALIFIED.retained_commands as usize;
    let world = DrawListWorld::new();
    let initial_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let rows = (0..CAPACITY)
        .map(|_| {
            world.rect(
                initial_frame,
                UiMountedInstanceIdentity::mint_unbound().unwrap(),
                0.0,
                UiMountedRgba8::new(20, 30, 40, 255),
            )
        })
        .collect::<Vec<_>>();
    let initial_commands = rows.iter().copied().map(command).collect::<Vec<_>>();
    let initial_order = initial_commands
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
    let removed = *initial_order.last().unwrap();
    let mut retained = UiNativeRetainedDrawList::from_complete(
        initial_frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &initial_commands,
        &initial_order,
        UiMountedPaintOrderIntegrity::for_order(&initial_order),
        &[],
    )
    .unwrap();
    let mut reversed_retained = UiNativeRetainedDrawList::from_complete(
        initial_frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &initial_commands,
        &initial_order,
        UiMountedPaintOrderIntegrity::for_order(&initial_order),
        &[],
    )
    .unwrap();

    let successor = UiMountedFrameIdentity::mint_unbound().unwrap();
    let inserted = command(world.rect(
        successor,
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        UiMountedRgba8::new(90, 100, 110, 255),
    ));
    let inserted_order = UiMountedPaintOrderIdentity::for_command(inserted.identity());
    let mut final_order = Vec::with_capacity(CAPACITY);
    final_order.push(inserted_order);
    final_order.extend_from_slice(&initial_order[..CAPACITY - 1]);
    let damage = vec![
        UiMountedLogicalDamage::from_runtime_mounting(inserted.bounds()),
        UiMountedLogicalDamage::from_runtime_mounting(initial_commands[CAPACITY - 1].bounds()),
    ];
    let delta = UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor: initial_frame,
        successor,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        // Deliberately claim successor capacity before releasing the old row.
        changes: vec![
            UiMountedPaintCommandChange::Insert(inserted),
            UiMountedPaintCommandChange::Remove(removed.command()),
        ],
        nodes: Vec::new(),
        order: vec![
            UiMountedPaintOrderEdit::place_after(inserted_order, None),
            UiMountedPaintOrderEdit::remove(removed),
        ],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(&final_order),
        damage,
        auxiliary: None,
        production_cost: Default::default(),
    });

    let reversed =
        UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
            predecessor: initial_frame,
            successor,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            changes: delta.changes().iter().rev().cloned().collect(),
            nodes: Vec::new(),
            order: delta.order().iter().rev().copied().collect(),
            order_integrity: delta.order_integrity(),
            damage: delta.damage().to_vec(),
            auxiliary: None,
            production_cost: Default::default(),
        });

    retained.apply_delta(&delta).unwrap();
    reversed_retained.apply_delta(&reversed).unwrap();
    assert_eq!(retained.order.ordered().collect::<Vec<_>>(), final_order);
    assert_eq!(
        reversed_retained.order.ordered().collect::<Vec<_>>(),
        final_order
    );
    assert_eq!(retained.commands.len(), CAPACITY);
    assert!(retained.command(removed.command()).is_none());
    assert!(retained.command(inserted_order.command()).is_some());
}

#[path = "state/command_view.rs"]
mod command_view;

use worth_ui_host_contract::{
    UiMountedEffectFamily, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiMountedPaintOrderIdentity, UiMountedPaintOrderIntegrity, UiMountedPresentationAuxiliaryState,
    UiMountedProjectionView,
};

use crate::runtime::persistent_index::{UiPersistentOrdMap, UiPersistentOrder};

#[derive(Clone)]
pub(crate) struct UiMountedPresentationState {
    pub(super) predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    pub(super) rebound_from_binding: Option<worth_ui_host_contract::UiSurfaceBindingGeneration>,
    pub(super) content: worth_ui_host_contract::UiMountedContentGeneration,
    pub(super) receipt_affinity: Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity>,
    pub(super) commands_by_instance: UiPersistentOrdMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        super::command_bundle::UiMountedPresentationCommandBundle,
    >,
    portal_motion_groups: super::portal_motion_groups::UiMountedPortalMotionGroups,
    instance_order: UiPersistentOrder<worth_ui_host_contract::UiMountedInstanceIdentity>,
    command_order: UiPersistentOrdMap<PresentationOrderKey, UiMountedPaintCommandIdentity>,
    pub(super) order_integrity: UiMountedPaintOrderIntegrity,
    pub(super) auxiliary: UiMountedPresentationAuxiliaryState,
    pub(super) effects: Box<[UiMountedEffectFamily]>,
    pub(super) node_changes:
        std::sync::Arc<[worth_ui_host_contract::UiMountedPresentationNodeChange]>,
    pub(super) projection_rows_materialized: u64,
}

type PresentationOrderKey = (u32, u64, usize);

impl UiMountedPresentationState {
    pub(in crate::mounting::presentation) fn semantic_surface(
        &self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.requirement.semantic_surface()
    }

    pub(crate) fn from_projection(
        projection: &UiMountedProjectionView,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    ) -> Self {
        let mut instance_order = UiPersistentOrder::default();
        let mut command_order = UiPersistentOrdMap::default();
        let command_indices = projection.retained_command_indices();
        let retained_commands = projection.retained_paint_commands();
        let commands_by_instance = projection.retained_commands_by_instance();
        let mut persistent_commands = UiPersistentOrdMap::default();
        for node in projection.nodes() {
            let instance = node.mounted_instance();
            instance_order
                .append(instance)
                .expect("projection nodes are unique");
            let commands = commands_by_instance
                .get(&instance)
                .into_iter()
                .flat_map(|identities| identities.iter())
                .filter_map(|identity| {
                    command_indices
                        .get(identity)
                        .and_then(|index| retained_commands.get(*index))
                        .cloned()
                })
                .collect::<Vec<_>>();
            if !commands.is_empty() {
                let position = instance_order
                    .position(instance)
                    .expect("appended position");
                for (local, command) in commands.iter().enumerate() {
                    command_order.insert(
                        (command.layer_semantic_order(), position, local),
                        command.identity(),
                    );
                }
                persistent_commands.insert(
                    instance,
                    super::command_bundle::UiMountedPresentationCommandBundle::from_commands(
                        &commands,
                    ),
                );
            }
        }
        let portal_motion_groups =
            super::portal_motion_groups::UiMountedPortalMotionGroups::from_projection(
                projection,
                &persistent_commands,
            );
        Self {
            predecessor,
            frame: projection.frame(),
            requirement,
            rebound_from_binding: None,
            content: projection.content_generation(),
            receipt_affinity: projection.node_receipt_affinity(),
            commands_by_instance: persistent_commands,
            portal_motion_groups,
            instance_order,
            command_order,
            order_integrity: projection.retained_order_integrity(),
            auxiliary: UiMountedPresentationAuxiliaryState::from_runtime_mounting(projection),
            effects: super::super::effect_requirements::required_effects(
                requirement.presentation_mode(),
                projection,
            )
            .into_boxed_slice(),
            node_changes: std::sync::Arc::from([]),
            projection_rows_materialized: super::projection_row_count::projection_row_count(
                projection,
            ),
        }
    }

    pub(in crate::mounting::presentation) fn successor_from_source(
        predecessor: &Self,
        source: crate::mounting::UiMountedPresentationDeltaSource<'_>,
        projection: Option<&UiMountedProjectionView>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    ) -> Self {
        let mut successor = predecessor.clone();
        successor.predecessor = source.predecessor();
        successor.frame = source.frame().frame_identity();
        successor.requirement = requirement;
        successor.rebound_from_binding = None;
        successor.content = source.frame().content_generation();
        successor.receipt_affinity = source.frame().node_receipt_affinity();
        successor.projection_rows_materialized = source.frame().materialized_projection_rows();
        for instance in source
            .changed_instances()
            .iter()
            .filter(|instance| !source.frame().has_precise_command_delta(**instance))
        {
            successor.remove_bundle(*instance);
        }
        successor.instance_order = source
            .frame()
            .presentation_instance_order(requirement.semantic_surface(), requirement.binding());
        let mut insertions = source
            .changed_instances()
            .iter()
            .filter(|instance| !source.frame().has_precise_command_delta(**instance))
            .filter_map(|instance| {
                successor
                    .instance_order
                    .position(*instance)
                    .map(|position| (position, *instance))
            })
            .collect::<Vec<_>>();
        insertions.sort_unstable_by_key(|(position, _)| *position);
        for (position, instance) in insertions {
            let commands = source.frame().presentation_commands_for_instance(
                instance,
                requirement.semantic_surface(),
                requirement.binding(),
            );
            successor.insert_bundle(position, instance, commands);
            successor.inherit_unchanged_motion(predecessor, std::slice::from_ref(&instance));
        }
        successor.apply_precise_replacements(source.frame().presentation_command_changes());
        for instance in source.changed_instances().iter().copied() {
            let commands = successor.commands_by_instance.get(&instance).cloned();
            let affinity = source.frame().portal_presentation_affinity_for_instance(
                instance,
                requirement.semantic_surface(),
                requirement.binding(),
            );
            successor.portal_motion_groups.replace_instance(
                instance,
                commands.as_ref(),
                affinity,
                requirement.semantic_surface(),
            );
        }
        successor.effects = source
            .frame()
            .presentation_effects(requirement.presentation_mode(), requirement.binding());
        successor.node_changes = source
            .frame()
            .presentation_node_changes(
                source.node_changed_instances(),
                requirement.semantic_surface(),
                requirement.binding(),
            )
            .into_iter()
            .filter(|change| match change {
                worth_ui_host_contract::UiMountedPresentationNodeChange::Remove(instance) => {
                    predecessor.instance_order.position(*instance).is_some()
                }
                worth_ui_host_contract::UiMountedPresentationNodeChange::Upsert(_) => true,
            })
            .collect::<Vec<_>>()
            .into();
        let mut lane_candidate = successor.auxiliary.clone();
        source
            .frame()
            .refresh_presentation_lane_auxiliary(&mut lane_candidate, requirement);
        if !successor
            .auxiliary
            .same_lane_presentation_meaning(&lane_candidate)
        {
            successor.auxiliary = UiMountedPresentationAuxiliaryState::from_runtime_mounting(
                projection.expect("lane changes require the complete successor projection"),
            );
        }
        successor
    }

    pub(in crate::mounting::presentation) fn successor_projection_required(
        predecessor: &Self,
        source: crate::mounting::UiMountedPresentationDeltaSource<'_>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    ) -> bool {
        let mut candidate = predecessor.auxiliary.clone();
        source
            .frame()
            .refresh_presentation_lane_auxiliary(&mut candidate, requirement);
        !predecessor
            .auxiliary
            .same_lane_presentation_meaning(&candidate)
    }

    fn apply_precise_replacements(
        &mut self,
        changes: &[worth_ui_host_contract::UiMountedPaintCommandChange],
    ) {
        for change in changes {
            let worth_ui_host_contract::UiMountedPaintCommandChange::Replace {
                successor: command,
                ..
            } = change
            else {
                continue;
            };
            let instance = command.identity().mounted_instance();
            let Some(mut bundle) = self.commands_by_instance.get(&instance).cloned() else {
                continue;
            };
            assert!(
                bundle.replace(command.clone()),
                "precise replacement preserves key and order"
            );
            self.commands_by_instance.insert(instance, bundle);
        }
    }

    fn remove_bundle(&mut self, instance: worth_ui_host_contract::UiMountedInstanceIdentity) {
        let Some(commands) = self.commands_by_instance.get(&instance).cloned() else {
            return;
        };
        let position = self
            .instance_order
            .position(instance)
            .expect("retained command instance has an order position");
        let mut keys = commands
            .iter()
            .enumerate()
            .map(|(local, command)| {
                (
                    (command.layer_semantic_order(), position, local),
                    command.identity(),
                )
            })
            .collect::<Vec<_>>();
        keys.sort_unstable_by_key(|(key, _)| *key);
        for (key, command) in keys.into_iter().rev() {
            let identity = UiMountedPaintOrderIdentity::for_command(command);
            let predecessor = self
                .command_order
                .predecessor(&key)
                .map(|(_, identity)| UiMountedPaintOrderIdentity::for_command(*identity));
            let successor = self
                .command_order
                .successor(&key)
                .map(|(_, identity)| UiMountedPaintOrderIdentity::for_command(*identity));
            self.order_integrity = self
                .order_integrity
                .remove_edge(predecessor, identity, successor)
                .expect("retained order removal preserves bounded length");
            self.command_order.remove(&key);
        }
        self.commands_by_instance.remove(&instance);
    }

    fn insert_bundle(
        &mut self,
        position: u64,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        commands: std::sync::Arc<[UiMountedPaintCommand]>,
    ) {
        if commands.is_empty() {
            return;
        }
        for (local, command) in commands.iter().enumerate() {
            let key = (command.layer_semantic_order(), position, local);
            let identity = UiMountedPaintOrderIdentity::for_command(command.identity());
            let predecessor = self
                .command_order
                .predecessor(&key)
                .map(|(_, identity)| UiMountedPaintOrderIdentity::for_command(*identity));
            let successor = self
                .command_order
                .successor(&key)
                .map(|(_, identity)| UiMountedPaintOrderIdentity::for_command(*identity));
            self.order_integrity = self
                .order_integrity
                .insert_edge(predecessor, identity, successor)
                .expect("retained order insertion preserves bounded length");
            self.command_order.insert(key, command.identity());
        }
        self.commands_by_instance.insert(
            instance,
            super::command_bundle::UiMountedPresentationCommandBundle::from_commands(&commands),
        );
    }
}

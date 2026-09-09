use std::collections::BTreeMap;

use worth_ui_host_contract::{
    UiMountedLogicalDamage, UiMountedPaintCommand, UiMountedPresentationInitialInput,
    UiMountedPresentationProductionCost, UiMountedPresentationProductionCostInput,
    UiMountedPresentationReconstructionInput, UiMountedProjectionView,
};

use super::{UiMountedPresentationLease, UiMountedPresentationWork};

#[path = "work_producer/command_bundle.rs"]
mod command_bundle;
#[path = "work_producer/delta_diff.rs"]
mod delta_diff;
#[path = "work_producer/effect_expectations.rs"]
mod effect_expectations;
#[path = "work_producer/motion_evidence.rs"]
mod motion_evidence;
pub(in crate::mounting) use motion_evidence::motion_acceptance_reserved_bytes;
pub(super) use motion_evidence::UiPreparedCommandMotionAcceptance;
#[path = "work_producer/motion_sample.rs"]
mod motion_sample;
#[path = "work_producer/overlay_attribution.rs"]
mod overlay_attribution;
#[path = "work_producer/portal_motion_groups.rs"]
mod portal_motion_groups;
#[path = "work_producer/projection_row_count.rs"]
mod projection_row_count;
#[path = "work_producer/state.rs"]
mod state;
#[path = "work_producer/state_rebind.rs"]
mod state_rebind;
#[path = "work_producer/successor_issue.rs"]
mod successor_issue;
pub(super) use successor_issue::SuccessorIssueRequest;

pub(crate) use state::UiMountedPresentationState;

pub(super) type UiMountedPresentationCandidates =
    BTreeMap<worth_ui_host_contract::UiSurfaceBindingGeneration, UiMountedPresentationState>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiMountedPresentationWorkProductionDenial {
    StalePredecessor,
    SurfaceChanged,
    BindingChanged,
    BaselineChanged,
}

impl UiMountedPresentationState {
    pub(crate) fn issue_initial(
        &self,
        lease: &UiMountedPresentationLease,
        projection: &UiMountedProjectionView,
    ) -> UiMountedPresentationWork {
        assert!(self.predecessor.is_none());
        let commands = self.commands();
        let order = self.order();
        lease.issue_initial(UiMountedPresentationInitialInput {
            successor: self.frame,
            surface: self.requirement.semantic_surface(),
            binding: self.requirement.binding(),
            content: self.content,
            baseline: self.requirement.baseline(),
            projection: projection.clone(),
            commands: commands.clone(),
            order: order.clone(),
            order_integrity: self.order_integrity,
            damage: commands
                .iter()
                .filter_map(command_visible_bounds)
                .map(UiMountedLogicalDamage::from_runtime_mounting)
                .collect(),
            production_cost: production_cost(
                LocalWorkCost {
                    source_instances: self.source_instance_count(),
                    commands_considered: commands.len(),
                    command_index_lookups: order.len(),
                    order_lookups: order.len(),
                },
                RetainedTraversalCost::default(),
                self.projection_rows_materialized,
            ),
        })
    }

    pub(crate) fn issue_reconstruction(
        &self,
        lease: &UiMountedPresentationLease,
        projection: &UiMountedProjectionView,
        predecessor: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> UiMountedPresentationWork {
        assert_eq!(self.predecessor, Some(predecessor));
        let commands = self.commands();
        let order = self.order();
        lease.issue_reconstruction(UiMountedPresentationReconstructionInput {
            predecessor,
            successor: self.frame,
            surface: self.requirement.semantic_surface(),
            binding: self.requirement.binding(),
            content: self.content,
            baseline: self.requirement.baseline(),
            projection: projection.clone(),
            commands: commands.clone(),
            sample_overrides: self.reconstruction_motion_overrides(),
            order: order.clone(),
            order_integrity: self.order_integrity,
            damage: complete_damage(&commands),
            production_cost: production_cost(
                LocalWorkCost {
                    source_instances: self.source_instance_count(),
                    commands_considered: commands.len(),
                    command_index_lookups: order.len(),
                    order_lookups: order.len(),
                },
                RetainedTraversalCost {
                    scans: commands.len(),
                    clones: 0,
                },
                self.projection_rows_materialized,
            ),
        })
    }

    pub(super) fn issue_successor(
        &self,
        request: SuccessorIssueRequest<'_>,
    ) -> Result<UiMountedPresentationWork, UiMountedPresentationWorkProductionDenial> {
        successor_issue::SuccessorIssue {
            predecessor: self,
            request,
            retained_traversal: RetainedTraversalCost::default(),
        }
        .issue()
    }

    #[cfg(test)]
    pub(super) fn issue_successor_with_complete_retained_scan_mutant(
        &self,
        request: SuccessorIssueRequest<'_>,
    ) -> Result<UiMountedPresentationWork, UiMountedPresentationWorkProductionDenial> {
        let cloned = self.commands().to_vec();
        let traversed = cloned.len();
        std::hint::black_box(&cloned);
        successor_issue::SuccessorIssue {
            predecessor: self,
            request,
            retained_traversal: RetainedTraversalCost {
                scans: traversed,
                clones: cloned.len(),
            },
        }
        .issue()
    }
}

fn command_same_presentation_meaning(
    left: &UiMountedPaintCommand,
    right: &UiMountedPaintCommand,
) -> bool {
    match (left, right) {
        (
            UiMountedPaintCommand::FilledRect { mechanic: left, .. },
            UiMountedPaintCommand::FilledRect {
                mechanic: right, ..
            },
        ) => left.same_retained_paint_meaning(*right),
        (
            UiMountedPaintCommand::PortalOverlay { mechanic: left, .. },
            UiMountedPaintCommand::PortalOverlay {
                mechanic: right, ..
            },
        ) => left.same_retained_paint_meaning(*right),
        (
            UiMountedPaintCommand::SemanticText { mechanic: left, .. },
            UiMountedPaintCommand::SemanticText {
                mechanic: right, ..
            },
        ) => left.same_retained_paint_meaning(right),
        _ => false,
    }
}

fn command_same_presentation_meaning_after_binding_replacement(
    left: &UiMountedPaintCommand,
    right: &UiMountedPaintCommand,
    affected: worth_ui_host_contract::UiSurfaceBindingGeneration,
    replacement: worth_ui_host_contract::UiSurfaceBindingGeneration,
) -> bool {
    match (left, right) {
        (
            UiMountedPaintCommand::FilledRect { mechanic: left, .. },
            UiMountedPaintCommand::FilledRect {
                mechanic: right, ..
            },
        ) => left.same_retained_paint_meaning_after_binding_replacement(
            *right,
            affected,
            replacement,
        ),
        (
            UiMountedPaintCommand::PortalOverlay { mechanic: left, .. },
            UiMountedPaintCommand::PortalOverlay {
                mechanic: right, ..
            },
        ) => left.same_retained_paint_meaning_after_binding_replacement(
            *right,
            affected,
            replacement,
        ),
        (
            UiMountedPaintCommand::SemanticText { mechanic: left, .. },
            UiMountedPaintCommand::SemanticText {
                mechanic: right, ..
            },
        ) => {
            left.same_retained_paint_meaning_after_binding_replacement(right, affected, replacement)
        }
        _ => false,
    }
}

pub(super) fn command_visible_bounds(
    command: &UiMountedPaintCommand,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    let (bounds, clip) = match command {
        UiMountedPaintCommand::FilledRect { mechanic, .. } => {
            (mechanic.bounds(), mechanic.clip_bounds())
        }
        UiMountedPaintCommand::PortalOverlay { mechanic, .. } => {
            (mechanic.bounds(), mechanic.clip_bounds())
        }
        UiMountedPaintCommand::SemanticText { mechanic, .. } => {
            (mechanic.bounds(), mechanic.clip_bounds())
        }
    };
    if bounds.coordinate_space() != clip.coordinate_space() {
        return None;
    }
    let x = bounds.x().max(clip.x());
    let y = bounds.y().max(clip.y());
    let right = (bounds.x() + bounds.width()).min(clip.x() + clip.width());
    let bottom = (bounds.y() + bounds.height()).min(clip.y() + clip.height());
    if right <= x || bottom <= y {
        return None;
    }
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width: right - x,
            height: bottom - y,
            coordinate_space: bounds.coordinate_space(),
        },
    )
    .ok()
}

fn complete_damage(commands: &[UiMountedPaintCommand]) -> Vec<UiMountedLogicalDamage> {
    commands
        .iter()
        .filter_map(command_visible_bounds)
        .map(UiMountedLogicalDamage::from_runtime_mounting)
        .collect()
}

#[derive(Clone, Copy, Default)]
pub(super) struct RetainedTraversalCost {
    scans: usize,
    clones: usize,
}

impl RetainedTraversalCost {
    pub(super) fn add_scans(mut self, scans: usize) -> Self {
        self.scans = self.scans.saturating_add(scans);
        self
    }
}

#[derive(Clone, Copy)]
pub(super) struct LocalWorkCost {
    source_instances: usize,
    commands_considered: usize,
    command_index_lookups: usize,
    order_lookups: usize,
}

pub(super) fn production_cost(
    local: LocalWorkCost,
    retained: RetainedTraversalCost,
    projection_rows_materialized: u64,
) -> UiMountedPresentationProductionCost {
    UiMountedPresentationProductionCost::from_runtime_mounting(
        UiMountedPresentationProductionCostInput {
            source_instances: exact_count(local.source_instances),
            commands_considered: exact_count(local.commands_considered),
            command_index_lookups: exact_count(local.command_index_lookups),
            order_lookups: exact_count(local.order_lookups),
            retained_command_scans: exact_count(retained.scans),
            retained_command_clones: exact_count(retained.clones),
            projection_rows_materialized,
        },
    )
}

fn exact_count(value: usize) -> u64 {
    u64::try_from(value).expect("presentation work count fits the governed u64 counter")
}

use worth_ui::facade::app::{WorthUiNativeApplicationShell, WorthUiNativeReducedMotionPosture};
use worth_ui::facade::inspection::{UiFocusMoveInspectionOutcome, UiPortalClosedInspectionReason};
use worth_ui_native_platform::{
    UiNativeApplicationProgramDenial, UiNativeComponentSemanticTextChange,
};
use worth_ui_platform_pulse::product_world::{
    PlatformPulseCommandStory, PlatformPulseProductComponent, PlatformPulseQueryDenialStory,
};

mod intent_feedback;

#[derive(Default)]
pub(super) struct PlatformPulseProductStory {
    source: StoryCopy,
    command: StoryCopy,
    query_denial_label: StoryCopy,
    query_denial: StoryCopy,
    service: StoryCopy,
    status: StoryCopy,
    action: StoryCopy,
    confirmation: StoryCopy,
    portal: StoryCopy,
    portal_cancel: StoryCopy,
    portal_primary: StoryCopy,
}

#[derive(Clone)]
struct StoryCopy {
    value: Option<String>,
    revision: u64,
}

impl Default for StoryCopy {
    fn default() -> Self {
        Self {
            value: None,
            revision: 1,
        }
    }
}

impl PlatformPulseProductStory {
    pub(super) fn publish_source(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        sequence: u64,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let source = format!("Source {sequence} ·\napplication current");
        publish_changed(
            shell,
            &mut self.source,
            source,
            PlatformPulseProductComponent::EvidenceBody,
        )
    }

    pub(super) fn publish_query_denial(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        denial: worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionPreconditionDenial,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let predecessor_label = self.query_denial_label.clone();
        let predecessor_body = self.query_denial.clone();
        let story = PlatformPulseQueryDenialStory::new(denial);
        let changes = match (|| {
            Ok::<_, UiNativeApplicationProgramDenial>(
                [
                    changed(
                        &mut self.query_denial,
                        story.explanation().to_owned(),
                        PlatformPulseProductComponent::QueryDenialBody,
                    ),
                    Some(text_change(
                        &mut self.query_denial_label,
                        PlatformPulseProductComponent::QueryDenialLabel,
                        story.title(),
                    )?),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>(),
            )
        })() {
            Ok(changes) => changes,
            Err(denial) => {
                self.query_denial_label = predecessor_label;
                self.query_denial = predecessor_body;
                return Err(denial);
            }
        };
        if let Err(denial) = apply(shell, changes) {
            self.query_denial_label = predecessor_label;
            self.query_denial = predecessor_body;
            return Err(denial);
        }
        Ok(())
    }

    pub(super) fn refresh_runtime(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) -> Result<(), UiNativeApplicationProgramDenial> {
        let predecessor_command = self.command.clone();
        let predecessor_service = self.service.clone();
        let predecessor_status = self.status.clone();
        let census = shell.runtime_service_resource_census();
        let mut changes = Vec::new();
        if let Some(summary) = shell.why_command_won() {
            let story = PlatformPulseCommandStory::from_inspection(&summary);
            let command = format!("{}\n{}", story.winner(), story.explanation());
            if let Some(change) = changed(
                &mut self.command,
                command,
                PlatformPulseProductComponent::NativeBody,
            ) {
                changes.push(change);
            }
        }
        let service = shell.why_portal_closed().map_or_else(
            || {
                if census.active_portals() > 0 {
                    "Portal open · focus coordinated".to_owned()
                } else {
                    "Portal ready · bounded evidence".to_owned()
                }
            },
            |summary| format!("Portal closed · {}", close_reason(summary.reason())),
        );
        if let Some(change) = changed(
            &mut self.service,
            service,
            PlatformPulseProductComponent::EvidenceServiceBody,
        ) {
            changes.push(change);
        }
        let focus = shell
            .why_focus_moved()
            .map(|summary| focus_posture(summary.outcome()))
            .unwrap_or("idle");
        let status = format!(
            "Focus {focus}   ·   Portal {}   ·   Motion {}   ·   Reduce {}",
            census.active_portals(),
            census.active_motion_tracks(),
            reduced_motion(shell.native_reduced_motion_posture()),
        );
        if let Some(change) = changed(
            &mut self.status,
            status,
            PlatformPulseProductComponent::StatusText,
        ) {
            changes.push(change);
        }
        if let Err(denial) = apply(shell, changes) {
            self.command = predecessor_command;
            self.service = predecessor_service;
            self.status = predecessor_status;
            return Err(denial);
        }
        Ok(())
    }
}

fn publish_changed(
    shell: &mut WorthUiNativeApplicationShell,
    retained: &mut StoryCopy,
    text: String,
    component: PlatformPulseProductComponent,
) -> Result<(), UiNativeApplicationProgramDenial> {
    let predecessor = retained.clone();
    let outcome = apply(
        shell,
        changed(retained, text, component).into_iter().collect(),
    );
    if outcome.is_err() {
        *retained = predecessor;
    }
    outcome
}

fn changed(
    retained: &mut StoryCopy,
    text: String,
    component: PlatformPulseProductComponent,
) -> Option<UiNativeComponentSemanticTextChange> {
    if retained.value.as_deref() == Some(text.as_str()) {
        return None;
    }
    let change = UiNativeComponentSemanticTextChange::successor(
        component.authored_semantic_identity(),
        retained.revision,
        text.clone(),
    )
    .expect("Pulse dynamic copy is valid");
    retained.value = Some(text);
    retained.revision = retained
        .revision
        .checked_add(1)
        .expect("Pulse semantic-copy revision remains bounded");
    Some(change)
}

fn text_change(
    retained: &mut StoryCopy,
    component: PlatformPulseProductComponent,
    text: impl Into<Box<str>>,
) -> Result<UiNativeComponentSemanticTextChange, UiNativeApplicationProgramDenial> {
    let change = UiNativeComponentSemanticTextChange::successor(
        component.authored_semantic_identity(),
        retained.revision,
        text,
    )?;
    retained.revision = retained
        .revision
        .checked_add(1)
        .ok_or(UiNativeApplicationProgramDenial::ChangeCapacityExceeded)?;
    Ok(change)
}

fn apply(
    shell: &mut WorthUiNativeApplicationShell,
    changes: Vec<UiNativeComponentSemanticTextChange>,
) -> Result<(), UiNativeApplicationProgramDenial> {
    if changes.is_empty() {
        Ok(())
    } else {
        shell.apply_component_semantic_text(&changes)
    }
}

const fn focus_posture(outcome: UiFocusMoveInspectionOutcome) -> &'static str {
    match outcome {
        UiFocusMoveInspectionOutcome::Moved => "moved",
        UiFocusMoveInspectionOutcome::Unchanged => "steady",
        UiFocusMoveInspectionOutcome::Cleared => "clear",
        UiFocusMoveInspectionOutcome::NoEligibleParticipant => "unavailable",
    }
}

const fn close_reason(reason: UiPortalClosedInspectionReason) -> &'static str {
    match reason {
        UiPortalClosedInspectionReason::Escape => "escape",
        UiPortalClosedInspectionReason::OutsidePress => "outside press",
        UiPortalClosedInspectionReason::AcceptedSelection => "accepted action",
        UiPortalClosedInspectionReason::ExplicitOwnerRequest => "owner request",
        UiPortalClosedInspectionReason::AnchorLoss => "anchor moved",
        UiPortalClosedInspectionReason::ParentClosed => "parent closed",
        UiPortalClosedInspectionReason::OwnerLoss => "owner removed",
        UiPortalClosedInspectionReason::ApplicationShutdown => "shutdown",
        UiPortalClosedInspectionReason::WindowFocusPolicy => "window focus",
    }
}

const fn reduced_motion(posture: WorthUiNativeReducedMotionPosture) -> &'static str {
    match posture {
        WorthUiNativeReducedMotionPosture::NoPreference => "off",
        WorthUiNativeReducedMotionPosture::Reduce => "on",
        WorthUiNativeReducedMotionPosture::Unavailable => "unknown",
    }
}

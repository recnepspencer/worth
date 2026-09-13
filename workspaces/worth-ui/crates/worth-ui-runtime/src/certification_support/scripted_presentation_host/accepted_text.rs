use std::collections::BTreeMap;
use worth_ui_host_contract::{
    UiMountedFrameConsumptionView, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationWorkView, UiSemanticSurfaceIdentity,
};

/// Text consumed by accepted scripted host calls. A pending request is retained
/// by its completion token and becomes visible only when that token is presented.
#[derive(Default)]
pub(super) struct ScriptedAcceptedText {
    pending: BTreeMap<
        u64,
        (
            UiSemanticSurfaceIdentity,
            UiMountedPresentationAttemptIdentity,
            Vec<UiMountedPaintCommand>,
        ),
    >,
    surfaces: BTreeMap<
        UiSemanticSurfaceIdentity,
        (
            UiMountedPresentationAttemptIdentity,
            Vec<UiMountedPaintCommand>,
        ),
    >,
}

impl ScriptedAcceptedText {
    pub(super) fn record(&mut self, request: &UiMountedFrameConsumptionView<'_>) {
        let commands = self.project(request);
        self.surfaces
            .insert(request.surface(), (request.attempt(), commands));
    }

    pub(super) fn retain_pending(
        &mut self,
        token: u64,
        request: &UiMountedFrameConsumptionView<'_>,
    ) {
        let commands = self.project(request);
        assert!(self
            .pending
            .insert(token, (request.surface(), request.attempt(), commands))
            .is_none());
    }

    pub(super) fn accept_pending(&mut self, token: u64) {
        let (surface, attempt, commands) = self
            .pending
            .remove(&token)
            .expect("presented completion has its own retained request");
        self.surfaces.insert(surface, (attempt, commands));
    }

    pub(super) fn discard_pending(&mut self, token: u64) {
        self.pending.remove(&token);
    }

    fn project(&self, request: &UiMountedFrameConsumptionView<'_>) -> Vec<UiMountedPaintCommand> {
        let mut commands = self
            .surfaces
            .get(&request.surface())
            .map(|(_, commands)| commands.clone())
            .unwrap_or_default();
        match request.presentation_work() {
            UiMountedPresentationWorkView::Initial(work) => {
                commands = text_commands(work.commands());
            }
            UiMountedPresentationWorkView::Reconstruction(work) => {
                commands = text_commands(work.commands());
            }
            UiMountedPresentationWorkView::Delta(work) => {
                for change in work.changes() {
                    match change {
                        UiMountedPaintCommandChange::Insert(command) => {
                            commands.extend(text_commands(std::slice::from_ref(command)));
                        }
                        UiMountedPaintCommandChange::Replace {
                            predecessor,
                            successor,
                        } => {
                            commands.retain(|command| command.identity() != *predecessor);
                            commands.extend(text_commands(std::slice::from_ref(successor)));
                        }
                        UiMountedPaintCommandChange::Remove(identity) => {
                            commands.retain(|command| command.identity() != *identity);
                        }
                    }
                }
            }
            UiMountedPresentationWorkView::Sample(_)
            | UiMountedPresentationWorkView::Unchanged(_) => {}
        }
        commands
    }
}

impl super::ScriptedPresentationHost {
    pub fn accepted_text_commands(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<(
        UiMountedPresentationAttemptIdentity,
        Vec<UiMountedPaintCommand>,
    )> {
        self.state
            .lock()
            .unwrap()
            .accepted_text
            .surfaces
            .get(&surface)
            .cloned()
    }
}

fn text_commands(commands: &[UiMountedPaintCommand]) -> Vec<UiMountedPaintCommand> {
    commands
        .iter()
        .filter(|command| matches!(command, UiMountedPaintCommand::SemanticText { .. }))
        .cloned()
        .collect()
}

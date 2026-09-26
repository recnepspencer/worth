use std::collections::BTreeMap;
use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedFrameConsumptionView,
    UiMountedPaintCommand, UiMountedPaintCommandChange, UiMountedPaintCommandIdentity,
    UiMountedPresentationAttemptIdentity, UiMountedPresentationSampleChange,
    UiMountedPresentationWorkView, UiSemanticSurfaceIdentity,
};

/// Text consumed by accepted scripted host calls, with the sample each text
/// command is drawn through. A pending request is retained by its completion
/// token and changes what is shown only when that token is presented, in the
/// order the host acknowledged it.
#[derive(Default)]
pub(super) struct ScriptedAcceptedText {
    pending: BTreeMap<
        u64,
        (
            UiSemanticSurfaceIdentity,
            UiMountedPresentationAttemptIdentity,
            ScriptedTextWork,
            Vec<UiMountedPresentationSampleChange>,
        ),
    >,
    surfaces: BTreeMap<UiSemanticSurfaceIdentity, ScriptedShownText>,
}

struct ScriptedShownText {
    attempt: UiMountedPresentationAttemptIdentity,
    commands: Vec<UiMountedPaintCommand>,
    samples: Vec<UiMountedPresentationSampleChange>,
}

/// The part of one request that changes shown text: the retained draw list's
/// own rules, where a complete projection replaces commands and samples, a
/// delta retires the samples of every command it touches, and a re-issued
/// sample replaces its predecessor.
enum ScriptedTextWork {
    Complete {
        commands: Vec<UiMountedPaintCommand>,
        samples: Vec<UiMountedPresentationSampleChange>,
    },
    Delta(Vec<UiMountedPaintCommandChange>),
    Sample(Vec<UiMountedPresentationSampleChange>),
    Unchanged,
}

impl ScriptedAcceptedText {
    pub(super) fn record(&mut self, request: &UiMountedFrameConsumptionView<'_>) {
        let work = ScriptedTextWork::of(request);
        self.apply(
            request.surface(),
            request.attempt(),
            work,
            samples_of(request),
        );
    }

    pub(super) fn retain_pending(
        &mut self,
        token: u64,
        request: &UiMountedFrameConsumptionView<'_>,
    ) {
        let retained = (
            request.surface(),
            request.attempt(),
            ScriptedTextWork::of(request),
            samples_of(request),
        );
        assert!(self.pending.insert(token, retained).is_none());
    }

    pub(super) fn accept_pending(&mut self, token: u64) {
        let (surface, attempt, work, appearance) = self
            .pending
            .remove(&token)
            .expect("presented completion has its own retained request");
        self.apply(surface, attempt, work, appearance);
    }

    pub(super) fn discard_pending(&mut self, token: u64) {
        self.pending.remove(&token);
    }

    fn apply(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        attempt: UiMountedPresentationAttemptIdentity,
        work: ScriptedTextWork,
        appearance: Vec<UiMountedPresentationSampleChange>,
    ) {
        let shown = self.surfaces.entry(surface).or_insert(ScriptedShownText {
            attempt,
            commands: Vec::new(),
            samples: Vec::new(),
        });
        shown.attempt = attempt;
        match work {
            ScriptedTextWork::Complete { commands, samples } => {
                shown.commands = commands;
                shown.samples = samples;
            }
            ScriptedTextWork::Delta(changes) => {
                for change in &changes {
                    for identity in touched(change) {
                        shown.samples.retain(|sample| sample.command() != identity);
                    }
                    match change {
                        UiMountedPaintCommandChange::Insert(command) => {
                            shown
                                .commands
                                .extend(text_commands(std::slice::from_ref(command)));
                        }
                        UiMountedPaintCommandChange::Replace {
                            predecessor,
                            successor,
                        } => {
                            shown
                                .commands
                                .retain(|command| command.identity() != *predecessor);
                            shown
                                .commands
                                .extend(text_commands(std::slice::from_ref(successor)));
                        }
                        UiMountedPaintCommandChange::Remove(identity) => {
                            shown
                                .commands
                                .retain(|command| command.identity() != *identity);
                        }
                    }
                }
            }
            ScriptedTextWork::Sample(samples) => shown.reissue(samples),
            ScriptedTextWork::Unchanged => {}
        }
        shown.reissue(appearance);
    }
}

impl ScriptedShownText {
    fn reissue(&mut self, samples: Vec<UiMountedPresentationSampleChange>) {
        for sample in samples {
            self.samples
                .retain(|shown| shown.command() != sample.command());
            self.samples.push(sample);
        }
    }

    /// Each text command where the host draws it: its committed bounds, moved
    /// through the transform of the sample it is shown with.
    fn drawn(&self) -> Vec<(UiMountedPaintCommandIdentity, UiMountedCanonicalBox)> {
        self.commands
            .iter()
            .map(|command| {
                let bounds = command.bounds();
                let transform = self
                    .samples
                    .iter()
                    .find(|sample| sample.command() == command.identity())
                    .and_then(|sample| sample.transform());
                let drawn = transform.map_or(bounds, |transform| {
                    let (source, sampled) = (transform.source(), transform.sampled());
                    let scale = [
                        sampled.width() / source.width(),
                        sampled.height() / source.height(),
                    ];
                    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
                        x: sampled.x() + (bounds.x() - source.x()) * scale[0],
                        y: sampled.y() + (bounds.y() - source.y()) * scale[1],
                        width: bounds.width() * scale[0],
                        height: bounds.height() * scale[1],
                        coordinate_space: bounds.coordinate_space(),
                    })
                    .expect("an accepted sample maps its text to a canonical box")
                });
                (command.identity(), drawn)
            })
            .collect()
    }
}

impl ScriptedTextWork {
    fn of(request: &UiMountedFrameConsumptionView<'_>) -> Self {
        match request.presentation_work() {
            UiMountedPresentationWorkView::Initial(work) => Self::Complete {
                commands: text_commands(work.commands()),
                samples: Vec::new(),
            },
            UiMountedPresentationWorkView::Reconstruction(work) => Self::Complete {
                commands: text_commands(work.commands()),
                samples: work.sample_overrides().to_vec(),
            },
            UiMountedPresentationWorkView::Delta(work) => Self::Delta(work.changes().to_vec()),
            UiMountedPresentationWorkView::Sample(work) => Self::Sample(work.changes().to_vec()),
            UiMountedPresentationWorkView::Unchanged(_) => Self::Unchanged,
        }
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
            .map(|shown| (shown.attempt, shown.commands.clone()))
    }

    /// The commands on `surface` whose samples a request the host holds open
    /// retires or re-issues once it lands.
    pub fn held_open_displaced_commands(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Vec<UiMountedPaintCommandIdentity> {
        let state = self.state.lock().unwrap();
        state
            .accepted_text
            .pending
            .values()
            .filter(|(pending, ..)| *pending == surface)
            .flat_map(|(_, _, work, samples)| {
                let touched = match work {
                    ScriptedTextWork::Delta(changes) => changes.iter().flat_map(touched).collect(),
                    _ => Vec::new(),
                };
                touched
                    .into_iter()
                    .chain(samples.iter().map(|sample| sample.command()))
            })
            .collect()
    }

    /// Where the host draws each accepted text command on `surface`, through
    /// the samples it has accepted since that command was committed.
    pub fn accepted_text_drawn_bounds(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Vec<(UiMountedPaintCommandIdentity, UiMountedCanonicalBox)> {
        self.state
            .lock()
            .unwrap()
            .accepted_text
            .surfaces
            .get(&surface)
            .map(ScriptedShownText::drawn)
            .unwrap_or_default()
    }
}

fn samples_of(
    request: &UiMountedFrameConsumptionView<'_>,
) -> Vec<UiMountedPresentationSampleChange> {
    request
        .appearance_work()
        .map(|work| work.sample_overrides().to_vec())
        .unwrap_or_default()
}

/// Every command identity a delta change touches: the retained draw list
/// retires the sample of each one.
fn touched(change: &UiMountedPaintCommandChange) -> Vec<UiMountedPaintCommandIdentity> {
    match change {
        UiMountedPaintCommandChange::Insert(command) => vec![command.identity()],
        UiMountedPaintCommandChange::Replace {
            predecessor,
            successor,
        } => vec![*predecessor, successor.identity()],
        UiMountedPaintCommandChange::Remove(identity) => vec![*identity],
    }
}

fn text_commands(commands: &[UiMountedPaintCommand]) -> Vec<UiMountedPaintCommand> {
    commands
        .iter()
        .filter(|command| matches!(command, UiMountedPaintCommand::SemanticText { .. }))
        .cloned()
        .collect()
}

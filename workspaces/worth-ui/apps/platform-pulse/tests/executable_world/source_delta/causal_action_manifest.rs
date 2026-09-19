use std::collections::HashSet;
use std::fmt;
use std::time::Duration;

use serde::Deserialize;

const SOURCE: &str = include_str!("../adjudication/platform_pulse_causal_actions.json");
const SCHEMA: &str = "worth-ui-platform-pulse-causal-actions-v1";

#[derive(Deserialize)]
struct ManifestDefinition {
    schema: String,
    deadlines_ms: ManifestDeadlines,
    actions: Vec<ManifestAction>,
}

#[derive(Deserialize)]
struct ManifestDeadlines {
    first_frame: u64,
    transition: u64,
    idle_interval: u64,
    host_journey: u64,
}

#[derive(Deserialize)]
struct ManifestAction {
    id: String,
    after: Option<String>,
}

pub(crate) struct PulseCausalActionManifest {
    definition: ManifestDefinition,
    digest: [u8; 32],
}

pub(crate) struct PulseCausalActionCursor<'manifest> {
    manifest: &'manifest PulseCausalActionManifest,
    next: usize,
}

#[derive(Debug)]
pub(crate) enum PulseCausalActionManifestFailure {
    Decode(serde_json::Error),
    WrongSchema(String),
    Empty,
    Duplicate(String),
    BrokenOrdering {
        action: String,
        expected: Option<String>,
        observed: Option<String>,
    },
    UnexpectedAction {
        expected: Option<String>,
        observed: String,
    },
    Incomplete {
        remaining: usize,
    },
}

impl PulseCausalActionManifest {
    pub(crate) fn checked_in() -> Result<Self, PulseCausalActionManifestFailure> {
        let definition = serde_json::from_str::<ManifestDefinition>(SOURCE)
            .map_err(PulseCausalActionManifestFailure::Decode)?;
        if definition.schema != SCHEMA {
            return Err(PulseCausalActionManifestFailure::WrongSchema(
                definition.schema,
            ));
        }
        if definition.actions.is_empty() {
            return Err(PulseCausalActionManifestFailure::Empty);
        }
        let mut identities = HashSet::with_capacity(definition.actions.len());
        for (index, action) in definition.actions.iter().enumerate() {
            if !identities.insert(action.id.as_str()) {
                return Err(PulseCausalActionManifestFailure::Duplicate(
                    action.id.clone(),
                ));
            }
            let expected = index
                .checked_sub(1)
                .map(|predecessor| definition.actions[predecessor].id.as_str());
            if action.after.as_deref() != expected {
                return Err(PulseCausalActionManifestFailure::BrokenOrdering {
                    action: action.id.clone(),
                    expected: expected.map(str::to_owned),
                    observed: action.after.clone(),
                });
            }
        }
        let digest = crate::adjudication::content_fingerprint(SOURCE);
        Ok(Self { definition, digest })
    }

    pub(crate) fn cursor(&self) -> PulseCausalActionCursor<'_> {
        PulseCausalActionCursor {
            manifest: self,
            next: 0,
        }
    }

    pub(crate) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub(crate) fn action_count(&self) -> usize {
        self.definition.actions.len()
    }

    pub(crate) fn first_frame_deadline(&self) -> Duration {
        Duration::from_millis(self.definition.deadlines_ms.first_frame)
    }

    pub(crate) fn transition_deadline(&self) -> Duration {
        Duration::from_millis(self.definition.deadlines_ms.transition)
    }

    pub(crate) fn idle_interval(&self) -> Duration {
        Duration::from_millis(self.definition.deadlines_ms.idle_interval)
    }

    pub(crate) fn host_journey_deadline(&self) -> Duration {
        Duration::from_millis(self.definition.deadlines_ms.host_journey)
    }
}

impl PulseCausalActionCursor<'_> {
    pub(crate) fn advance(
        &mut self,
        observed: &'static str,
    ) -> Result<(), PulseCausalActionManifestFailure> {
        let expected = self
            .manifest
            .definition
            .actions
            .get(self.next)
            .map(|action| action.id.clone());
        if expected.as_deref() != Some(observed) {
            return Err(PulseCausalActionManifestFailure::UnexpectedAction {
                expected,
                observed: observed.to_owned(),
            });
        }
        self.next += 1;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<(), PulseCausalActionManifestFailure> {
        let remaining = self
            .manifest
            .definition
            .actions
            .len()
            .saturating_sub(self.next);
        if remaining == 0 {
            Ok(())
        } else {
            Err(PulseCausalActionManifestFailure::Incomplete { remaining })
        }
    }
}

impl fmt::Display for PulseCausalActionManifestFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "decode causal manifest: {error}"),
            Self::WrongSchema(schema) => write!(formatter, "unsupported causal schema {schema}"),
            Self::Empty => formatter.write_str("causal manifest has no actions"),
            Self::Duplicate(action) => write!(formatter, "duplicate causal action {action}"),
            Self::BrokenOrdering {
                action,
                expected,
                observed,
            } => write!(
                formatter,
                "causal action {action} follows {observed:?}, expected {expected:?}"
            ),
            Self::UnexpectedAction { expected, observed } => {
                write!(
                    formatter,
                    "observed action {observed}, expected {expected:?}"
                )
            }
            Self::Incomplete { remaining } => {
                write!(
                    formatter,
                    "causal manifest has {remaining} unconsumed actions"
                )
            }
        }
    }
}

//! Canonical coupled commits retain each genuine producer route and provenance.
use super::*;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalCoupledAuthoredSources {
    /// The prefix source followed by every consumed later stage, in apply order.
    Edit {
        sources: Vec<crate::identity::data::EntityId>,
    },
    Retirement {
        prefix: Arc<CanonicalAuthoredRetirement>,
        later: Vec<crate::identity::data::EntityId>,
    },
    Restoration {
        prefix: Arc<CanonicalAuthoredRestoration>,
        later: Vec<crate::identity::data::EntityId>,
    },
    MemberEditRetirement {
        sources: Vec<crate::identity::data::EntityId>,
        prefix: Arc<CanonicalAuthoredRetirement>,
    },
    MemberEditRestoration {
        sources: Vec<crate::identity::data::EntityId>,
        prefix: Arc<CanonicalAuthoredRestoration>,
    },
}
impl CanonicalCoupledAuthoredSources {
    pub fn edit_sources(&self) -> Option<&[crate::identity::data::EntityId]> {
        match self {
            Self::Edit { sources }
            | Self::MemberEditRetirement { sources, .. }
            | Self::MemberEditRestoration { sources, .. } => Some(sources),
            _ => None,
        }
    }
    pub fn is_edit(&self) -> bool {
        self.edit_sources().is_some()
    }
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalCoupledParticipation {
    Edit,
    Required,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalCoupledParticipant {
    pub origin: Arc<CanonicalRequiredOutputOrigin>,
    pub retirement: Option<Arc<CanonicalProducerRetirement>>,
    pub participation: CanonicalCoupledParticipation,
}
impl CanonicalCoupledParticipant {
    pub fn is_required(&self) -> bool {
        matches!(self.participation, CanonicalCoupledParticipation::Required)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CanonicalCoupledAuthoredPublication {
    pub sources: CanonicalCoupledAuthoredSources,
    pub participants: Vec<CanonicalCoupledParticipant>,
    #[serde(skip)]
    pub _decoded_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl CanonicalCoupledAuthoredPublication {
    pub fn participant_decode_storage(capacity: usize) -> Option<u64> {
        u64::try_from(
            std::alloc::Layout::array::<CanonicalCoupledParticipant>(capacity)
                .ok()?
                .size(),
        )
        .ok()
    }
    pub fn output_origins(&self) -> impl Iterator<Item = &Arc<CanonicalRequiredOutputOrigin>> {
        let sources: &[Arc<CanonicalRequiredOutputOrigin>] = match &self.sources {
            CanonicalCoupledAuthoredSources::Retirement { prefix, .. }
            | CanonicalCoupledAuthoredSources::MemberEditRetirement { prefix, .. } => {
                &prefix.publications
            }
            CanonicalCoupledAuthoredSources::Restoration { prefix, .. }
            | CanonicalCoupledAuthoredSources::MemberEditRestoration { prefix, .. } => {
                &prefix.publications
            }
            CanonicalCoupledAuthoredSources::Edit { .. } => &[],
        };
        sources
            .iter()
            .chain(self.participants.iter().flat_map(|part| {
                std::iter::once(&part.origin).chain(
                    part.retirement
                        .iter()
                        .flat_map(|event| event.publications.iter()),
                )
            }))
    }
}

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn authored_source_retirement(&self) -> Option<&Arc<CanonicalAuthoredRetirement>> {
        self.authored_cascade_retirement.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Retirement { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRetirement { prefix, .. } => {
                    Some(prefix)
                }
                _ => None,
            }
        })
    }
    pub(crate) fn authored_source_restoration(&self) -> Option<&Arc<CanonicalAuthoredRestoration>> {
        self.authored_cascade_restoration.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Restoration { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRestoration { prefix, .. } => {
                    Some(prefix)
                }
                _ => None,
            }
        })
    }
}

// Decode backing is transient custody, independent of publication meaning.
impl PartialEq for CanonicalCoupledAuthoredPublication {
    fn eq(&self, other: &Self) -> bool {
        self.sources == other.sources && self.participants == other.participants
    }
}
impl Eq for CanonicalCoupledAuthoredPublication {}

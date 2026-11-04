//! The admitted publication supplies selectors, never ownership facts.
use super::*;
use crate::transactions::data::RecordRef;
impl RequiredOutputPublication {
    pub(crate) fn ownership_seeds(&self, into: &mut Vec<RecordRef>) {
        match self {
            Self::Birth { meaning, .. } | Self::Preserve { meaning, .. } => {
                into.push(RecordRef::Entity(meaning.source_scope))
            }
            Self::AuthoredRetirement { predecessor, .. } => {
                into.push(RecordRef::Entity(predecessor.origin.meaning.source_scope))
            }
            Self::AuthoredCascadeRetirement { predecessor, .. } => {
                into.push(RecordRef::Entity(predecessor.anchor));
                into.extend(
                    predecessor
                        .co_anchors
                        .iter()
                        .map(|(id, _)| RecordRef::Entity(*id)),
                );
            }
            Self::AuthoredCascadeRestoration { restoration, .. } => {
                into.extend(
                    restoration
                        .publications
                        .iter()
                        .map(|p| RecordRef::Entity(p.origin.meaning.source_scope)),
                );
            }
            Self::CoupledAuthored { group } => {
                into.push(RecordRef::Entity(group.binding.owners.anchor));
                for part in &group.parts {
                    part.publication.ownership_seeds(into);
                }
            }
        }
    }
}

impl RequiredOutputPublication {
    pub(crate) fn current_ownership_evidence(
        &self,
    ) -> Option<&std::sync::Arc<crate::publication::ownership::CurrentPublicationRoster>> {
        use crate::publication::ownership::PublicationEvidence;
        let evidence = match self {
            Self::Preserve { predecessor, .. } | Self::AuthoredRetirement { predecessor, .. } => {
                Some(&predecessor.evidence)
            }
            Self::AuthoredCascadeRetirement { predecessor, .. } => {
                predecessor.publications.first().map(|p| &p.evidence)
            }
            Self::CoupledAuthored { group } => {
                group.binding.owners.current.first().map(|p| &p.evidence)
            }
            _ => None,
        }?;
        if let PublicationEvidence::Current(current) = evidence {
            Some(current)
        } else {
            None
        }
    }
}

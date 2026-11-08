//! The admitted publication supplies selectors, never ownership facts.
use super::*;
use crate::transactions::data::RecordRef;
impl RequiredOutputPublication {
    /// The distinguished source for an admission failure, without walking a roster.
    pub(crate) fn ownership_subject(
        &self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
    ) -> Option<crate::identity::data::EntityId> {
        if let Some((producer, _)) = self.lineage_attribution()
            && installed
                .source_scope_ownership
                .scope_kind(producer)
                .is_none()
        {
            return None;
        }
        Some(match self {
            Self::Birth { meaning, .. } | Self::Preserve { meaning, .. } => meaning.source_scope,
            Self::AuthoredRetirement { predecessor, .. } => predecessor.origin.meaning.source_scope,
            Self::AuthoredCascadeRetirement { predecessor, .. } => predecessor.anchor,
            Self::AuthoredCascadeRestoration { restoration, .. } => {
                restoration
                    .publications
                    .first()?
                    .origin
                    .meaning
                    .source_scope
            }
            Self::HistoricalSelection { selections, .. } => {
                selections.first()?.publication.origin.meaning.source_scope
            }
            Self::CoupledAuthored { group } => group.binding.owners.source.subject(),
        })
    }

    pub(crate) fn ownership_seeds(&self, into: &mut Vec<RecordRef>) {
        match self {
            Self::HistoricalSelection { selections, .. } => into.extend(
                selections
                    .iter()
                    .map(|s| RecordRef::Entity(s.publication.origin.meaning.source_scope)),
            ),
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
                into.push(RecordRef::Entity(group.binding.owners.source.subject()));
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

use std::collections::{BTreeMap, BTreeSet};

use worth_ui_inspection::{UiAppearanceInspectionWorld, UiEvidenceAuthorityGeneration};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceInspectionGenerationSuccessionDenial {
    StaleInspectionGeneration,
    InspectionEvidenceGenerationExhausted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceInspectionScope {
    pub(super) session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
}

impl UiAppearanceInspectionScope {
    pub(super) fn new(
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            session: generation.session_identity(),
            generation,
        }
    }

    pub(super) fn from_parts(
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            session,
            generation: generation.clone(),
        }
    }

    fn matches_generation(
        &self,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.session == generation.session_identity() && self.generation == *generation
    }

    pub(super) fn world(
        &self,
        evidence_generation: UiEvidenceAuthorityGeneration,
        surface_identity: u64,
    ) -> UiAppearanceInspectionWorld {
        UiAppearanceInspectionWorld::new(
            self.session.as_u64(),
            evidence_generation,
            surface_identity,
        )
    }
}

pub(crate) struct UiPreparedAppearanceInspectionGenerationSuccession {
    predecessor: UiAppearanceInspectionScope,
    successor: UiAppearanceInspectionScope,
    evidence_generation: UiEvidenceAuthorityGeneration,
    records: InspectionRecordSuccession,
}

enum InspectionRecordSuccession {
    Retain,
    Clear,
}

impl super::UiAppearanceInspectionProducer {
    pub(crate) fn new(
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            entries: BTreeMap::new(),
            expired: BTreeSet::new(),
            current_scope: Some(UiAppearanceInspectionScope::new(generation)),
            evidence_generation: UiEvidenceAuthorityGeneration::new(1),
            next_sequence: 1,
            #[cfg(test)]
            test_current_world: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test() -> Self {
        Self {
            entries: BTreeMap::new(),
            expired: BTreeSet::new(),
            current_scope: None,
            evidence_generation: UiEvidenceAuthorityGeneration::new(1),
            next_sequence: 1,
            test_current_world: None,
        }
    }

    pub(crate) fn current_world(&self, surface_identity: u64) -> UiAppearanceInspectionWorld {
        self.current_scope
            .as_ref()
            .expect("active appearance inspection has an exact generation scope")
            .world(self.evidence_generation, surface_identity)
    }

    pub(crate) fn prepare_generation_succession(
        &self,
        predecessor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<
        UiPreparedAppearanceInspectionGenerationSuccession,
        UiAppearanceInspectionGenerationSuccessionDenial,
    > {
        let Some(current_scope) = self.current_scope.as_ref() else {
            return Err(
                UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration,
            );
        };
        if !current_scope.matches_generation(predecessor)
            || predecessor.session_identity() != successor.session_identity()
        {
            return Err(
                UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration,
            );
        }
        let evidence_generation = if predecessor == successor {
            self.evidence_generation
        } else {
            UiEvidenceAuthorityGeneration::new(
                self.evidence_generation
                    .as_u64()
                    .checked_add(1)
                    .ok_or(
                        UiAppearanceInspectionGenerationSuccessionDenial::
                            InspectionEvidenceGenerationExhausted,
                    )?,
            )
        };
        Ok(UiPreparedAppearanceInspectionGenerationSuccession {
            predecessor: current_scope.clone(),
            successor: UiAppearanceInspectionScope::new(successor.clone()),
            evidence_generation,
            records: InspectionRecordSuccession::Clear,
        })
    }

    pub(crate) fn prepare_retained_generation_succession(
        &self,
        predecessor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<
        UiPreparedAppearanceInspectionGenerationSuccession,
        UiAppearanceInspectionGenerationSuccessionDenial,
    > {
        let mut prepared = self.prepare_generation_succession(predecessor, successor)?;
        prepared.records = InspectionRecordSuccession::Retain;
        Ok(prepared)
    }

    pub(crate) fn commit_generation_succession(
        &mut self,
        prepared: UiPreparedAppearanceInspectionGenerationSuccession,
    ) {
        debug_assert_eq!(self.current_scope.as_ref(), Some(&prepared.predecessor));
        if prepared.predecessor == prepared.successor {
            debug_assert_eq!(prepared.evidence_generation, self.evidence_generation);
            return;
        }
        match prepared.records {
            InspectionRecordSuccession::Retain => {
                let successor_key = |(world, node, aspect): super::InspectionKey| {
                    (
                        prepared
                            .successor
                            .world(prepared.evidence_generation, world.surface_identity()),
                        node,
                        aspect,
                    )
                };
                // Reconciliation may record newer evidence during detached retry.
                // Move the current bounded records; never overwrite them with a
                // snapshot captured before rejection. Decision evidence is unchanged.
                self.entries = std::mem::take(&mut self.entries)
                    .into_iter()
                    .map(|(key, mut entry)| {
                        let key = successor_key(key);
                        entry.explanation = entry.explanation.with_query(
                            worth_ui_inspection::UiAppearanceInspectionQuery::new(
                                key.0, key.1, key.2,
                            ),
                        );
                        (key, entry)
                    })
                    .collect();
                self.expired = std::mem::take(&mut self.expired)
                    .into_iter()
                    .map(successor_key)
                    .collect();
            }
            InspectionRecordSuccession::Clear => {
                self.entries.clear();
                self.expired.clear();
            }
        }
        self.current_scope = Some(prepared.successor);
        self.evidence_generation = prepared.evidence_generation;
    }
}

#[cfg(test)]
#[path = "generation_tests.rs"]
mod tests;

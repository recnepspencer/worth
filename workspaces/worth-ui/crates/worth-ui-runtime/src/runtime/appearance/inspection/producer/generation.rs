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
        })
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
        self.entries.clear();
        self.expired.clear();
        self.current_scope = Some(prepared.successor);
        self.evidence_generation = prepared.evidence_generation;
    }
}

#[cfg(test)]
mod tests {
    use super::{UiAppearanceInspectionGenerationSuccessionDenial, UiAppearanceInspectionScope};
    use crate::runtime::tests::appearance_component_session_test_support as support;
    use worth_ui_dsl::{UiAppearanceAspect, UiAppearanceAxisClass, UiThemeColor, UiThemeValue};
    use worth_ui_inspection::{
        UiAppearanceInspectionChangeDistinctions, UiAppearanceInspectionCost,
        UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
        UiAppearanceInspectionExplanation, UiAppearanceInspectionMountedMechanic,
        UiAppearanceInspectionOutcome, UiAppearanceInspectionPhysicalSuppression,
        UiAppearanceInspectionQuery, UiAppearanceInspectionSourceSpan,
        UiAppearanceInspectionSupport, UiAppearanceInspectionValue, UiEvidenceAuthorityGeneration,
    };

    use super::super::UiAppearanceInspectionProducer;

    #[test]
    fn same_generation_succession_preserves_current_owner_evidence() {
        let session = support::source_backed_static_paint_consumer_session();
        let generation = session.active_generation_identity().clone();
        let mut producer = UiAppearanceInspectionProducer::new(generation.clone());
        let query = UiAppearanceInspectionQuery::new(
            producer.current_world(3),
            7,
            UiAppearanceAspect::Background,
        );
        producer.record(query, owner_explanation(query));
        let world = query.world();
        let prepared = producer
            .prepare_generation_succession(&generation, &generation)
            .expect("same generation is a no-op succession");
        producer.commit_generation_succession(prepared);

        assert_eq!(producer.current_world(3), world);
        assert!(matches!(
            producer.query(query),
            UiAppearanceInspectionOutcome::Found(_)
        ));
        let _ = session.shutdown();
    }

    #[test]
    fn stale_foreign_and_exhausted_succession_are_refused_without_mutation() {
        let role = support::validation_background_role(support::APPEARANCE_TOKEN);
        let mut session = support::source_backed_static_paint_consumer_session();
        let predecessor = session.active_generation_identity().clone();
        let mut producer = UiAppearanceInspectionProducer::new(predecessor.clone());
        let query = UiAppearanceInspectionQuery::new(
            producer.current_world(3),
            7,
            UiAppearanceAspect::Background,
        );
        producer.record(query, owner_explanation(query));
        let entry_count = producer.entries.len();
        let next_sequence = producer.next_sequence;

        let candidate = support::appearance_candidate_submission(
            &session,
            "inspection-generation-owner-successor",
            Some(&role),
        );
        let mut observation = session.begin_observation_turn().unwrap();
        observation.admit_source(candidate).unwrap();
        let observations = observation.seal().unwrap();
        let evidence = match session.classify_observations(observations).unwrap() {
            crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(evidence) => {
                evidence
            }
            _ => panic!("equal authored semantics must produce evidence-only succession"),
        };
        let plan = session
            .compile_preservation_rebind(
                evidence,
                crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
            )
            .unwrap();
        let prepared = session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(1),
            )
            .unwrap();
        let published = match prepared.execute(1) {
            crate::runtime::rebind::UiRebindOutcome::Published(receipt) => receipt,
            _ => panic!("real evidence-only succession must publish"),
        };
        let successor = session.active_generation_identity().clone();

        let current_world = producer.current_world(3);
        let current_scope = producer.current_scope.clone();
        let current_entries = producer
            .entries
            .iter()
            .map(|(key, entry)| (*key, entry.sequence))
            .collect::<Vec<_>>();
        let current_expired = producer.expired.clone();
        let current_sequence = producer.next_sequence;
        let stale_predecessor = producer.prepare_generation_succession(&successor, &successor);
        assert!(matches!(
            stale_predecessor,
            Err(UiAppearanceInspectionGenerationSuccessionDenial::StaleInspectionGeneration)
        ));
        assert_eq!(producer.current_scope, current_scope);
        assert_eq!(producer.current_world(3), current_world);
        assert_eq!(
            producer
                .entries
                .iter()
                .map(|(key, entry)| (*key, entry.sequence))
                .collect::<Vec<_>>(),
            current_entries
        );
        assert_eq!(producer.expired, current_expired);
        assert_eq!(producer.next_sequence, current_sequence);

        let stale_scope = UiAppearanceInspectionScope::new(successor.clone());
        let stale_query = UiAppearanceInspectionQuery::new(
            stale_scope.world(UiEvidenceAuthorityGeneration::new(1), 3),
            7,
            UiAppearanceAspect::Background,
        );
        producer.record_scoped(&stale_scope, stale_query, owner_explanation(stale_query));
        assert_eq!(producer.entries.len(), entry_count);
        assert_eq!(producer.next_sequence, next_sequence);
        assert!(matches!(
            producer.query(query),
            UiAppearanceInspectionOutcome::Found(_)
        ));

        let foreign_session = support::source_backed_static_paint_consumer_session();
        let foreign_scope =
            UiAppearanceInspectionScope::new(foreign_session.active_generation_identity().clone());
        let foreign_query = UiAppearanceInspectionQuery::new(
            foreign_scope.world(UiEvidenceAuthorityGeneration::new(1), 3),
            7,
            UiAppearanceAspect::Background,
        );
        producer.record_scoped(
            &foreign_scope,
            foreign_query,
            owner_explanation(foreign_query),
        );
        assert_eq!(producer.entries.len(), entry_count);
        assert_eq!(producer.next_sequence, next_sequence);
        assert!(matches!(
            producer.query(query),
            UiAppearanceInspectionOutcome::Found(_)
        ));

        let predecessor_scope = producer.current_scope.clone();
        producer.evidence_generation = UiEvidenceAuthorityGeneration::new(u64::MAX);
        let denied = producer.prepare_generation_succession(&predecessor, &successor);
        assert!(matches!(
            denied,
            Err(UiAppearanceInspectionGenerationSuccessionDenial::
                InspectionEvidenceGenerationExhausted)
        ));
        assert_eq!(producer.current_scope, predecessor_scope);
        assert_eq!(producer.entries.len(), entry_count);
        assert_eq!(producer.next_sequence, next_sequence);
        assert_eq!(
            producer.evidence_generation,
            UiEvidenceAuthorityGeneration::new(u64::MAX)
        );

        drop(published);
        let _ = foreign_session.shutdown();
        let _ = session.shutdown();
    }

    fn owner_explanation(query: UiAppearanceInspectionQuery) -> UiAppearanceInspectionExplanation {
        UiAppearanceInspectionExplanation::new(
            query,
            "test.role",
            1,
            "test.theme",
            1,
            Box::<[UiAppearanceAxisClass]>::from([]),
            UiAppearanceInspectionDecisionCell::new(1, Box::<[UiAppearanceAxisClass]>::from([])),
            UiAppearanceInspectionSourceSpan::Unavailable,
            "surface.background",
            "surface.background",
            UiAppearanceInspectionSupport::Supported,
            UiAppearanceInspectionValue::Resolved(UiThemeValue::Color(
                UiThemeColor::from_channels([1, 2, 3, 255]),
            )),
            UiAppearanceInspectionChangeDistinctions::new(false, false, false, false, false, false),
            UiAppearanceInspectionMountedMechanic::NotAttempted,
            UiAppearanceInspectionPhysicalSuppression::NotAttempted,
            query.graph_node_digest(),
            UiAppearanceInspectionEvidence::new(1, 1, [0; 6]),
            UiAppearanceInspectionCost::new(0, 1, 1, 1, 0),
        )
    }
}

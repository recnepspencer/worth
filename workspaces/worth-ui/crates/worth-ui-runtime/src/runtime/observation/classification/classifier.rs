use super::{
    owner, UiAuthoredSourceClassification, UiChangeClassificationDenial,
    UiChangeClassificationOutcome, UiClassifiedChange, UiEvidenceOnlySourceChange,
    UiObservedNoChangeReceipt,
};
use crate::fact_contract::UiProducedFact;
use crate::runtime::observation::turn::UiAdmittedObservationPayload;
use crate::runtime::observation::UiAdmittedObservationSet;

pub(crate) struct UiChangeClassifier;

pub(crate) struct UiChangeClassificationRequest<F> {
    pub(crate) set: UiAdmittedObservationSet,
    pub(crate) expected_session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    pub(crate) expected_source_basis: u64,
    pub(crate) predecessor_generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    pub(crate) fact_limit: usize,
    pub(crate) classify_source: F,
}

impl UiChangeClassifier {
    pub(crate) fn validate_basis(
        set: &UiAdmittedObservationSet,
        expected_session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        expected_source_basis: u64,
    ) -> Result<(), UiChangeClassificationDenial> {
        require_basis(set, expected_session, expected_source_basis)
    }

    pub(crate) fn classify<F>(
        request: UiChangeClassificationRequest<F>,
    ) -> Result<UiChangeClassificationOutcome, UiChangeClassificationDenial>
    where
        F: FnOnce(
            crate::runtime::WorthUiWatchedCandidateSubmission,
        ) -> Result<UiAuthoredSourceClassification, UiChangeClassificationDenial>,
    {
        let UiChangeClassificationRequest {
            set,
            expected_session,
            expected_source_basis,
            predecessor_generation,
            fact_limit,
            classify_source,
        } = request;
        require_basis(&set, expected_session, expected_source_basis)?;
        let turn = set.turn();
        let observation_count = set.summary().admitted_count();
        let mut facts = Vec::new();
        let mut source_succession = None;
        let mut classify_source = Some(classify_source);

        for observation in set.into_observations() {
            match observation.into_payload() {
                UiAdmittedObservationPayload::Source(candidate) => {
                    let source = classify_source
                        .take()
                        .expect("source duplicate policy admits at most one source observation");
                    match source(candidate)? {
                        UiAuthoredSourceClassification::ObservedNoChange => {}
                        UiAuthoredSourceClassification::EvidenceOnly(succession) => {
                            source_succession = Some(succession);
                        }
                        UiAuthoredSourceClassification::Changed {
                            facts: source_facts,
                            succession,
                        } => {
                            append_bounded(&mut facts, source_facts, fact_limit)?;
                            source_succession = Some(succession);
                        }
                    }
                }
                UiAdmittedObservationPayload::Host(observation) => {
                    push_bounded(&mut facts, owner::host::classify(observation)?, fact_limit)?;
                }
                UiAdmittedObservationPayload::PointerPresence(transition) => {
                    let expected_generation =
                        crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
                            expected_session,
                            &predecessor_generation,
                        );
                    if transition.generation() != &expected_generation {
                        return Err(UiChangeClassificationDenial::ForeignApplicationGeneration);
                    }
                    push_bounded(
                        &mut facts,
                        owner::pointer_presence::classify(transition),
                        fact_limit,
                    )?;
                }
                UiAdmittedObservationPayload::Measurement(result) => {
                    push_bounded(&mut facts, owner::measurement::classify(result), fact_limit)?;
                }
                UiAdmittedObservationPayload::Query(observation) => {
                    push_bounded(&mut facts, owner::query::classify(observation), fact_limit)?;
                }
                UiAdmittedObservationPayload::IntentPosture(observation) => {
                    push_bounded(&mut facts, owner::intent::classify(observation), fact_limit)?;
                }
                UiAdmittedObservationPayload::CommittedScrollExtent(observation) => {
                    push_bounded(
                        &mut facts,
                        owner::runtime_state::classify_scroll(observation),
                        fact_limit,
                    )?;
                }
                UiAdmittedObservationPayload::CommittedPortalAnchor(observation) => {
                    push_bounded(
                        &mut facts,
                        owner::runtime_state::classify_portal(observation),
                        fact_limit,
                    )?;
                }
            }
        }

        let basis = super::UiChangeClassificationBasis::new(
            expected_session,
            expected_source_basis,
            turn,
            observation_count,
            predecessor_generation,
        );
        Ok(resolve_outcome(basis, facts, source_succession))
    }

    pub(crate) fn classify_intent_consequence(
        set: UiAdmittedObservationSet,
        expected_session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        expected_source_basis: u64,
        predecessor_generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        fact_limit: usize,
    ) -> UiClassifiedChange {
        assert_eq!(set.session(), expected_session);
        assert_eq!(set.source_basis(), expected_source_basis);
        assert!(set.summary().admitted_count() <= fact_limit);
        assert!(set.summary().families().iter().all(|family| matches!(
            family,
            crate::runtime::observation::UiObservationFamily::Query
                | crate::runtime::observation::UiObservationFamily::IntentPosture
        )));
        let turn = set.turn();
        let observation_count = set.summary().admitted_count();
        let facts = set
            .into_observations()
            .into_vec()
            .into_iter()
            .map(|observation| match observation.into_payload() {
                UiAdmittedObservationPayload::Query(observation) => {
                    owner::query::classify(observation)
                }
                UiAdmittedObservationPayload::IntentPosture(observation) => {
                    owner::intent::classify(observation)
                }
                _ => unreachable!("intent consequence admission seals only declared families"),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        UiClassifiedChange::new(
            super::UiChangeClassificationBasis::new(
                expected_session,
                expected_source_basis,
                turn,
                observation_count,
                predecessor_generation,
            ),
            facts,
            None,
        )
    }
}

fn require_basis(
    set: &UiAdmittedObservationSet,
    expected_session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    expected_source_basis: u64,
) -> Result<(), UiChangeClassificationDenial> {
    if set.session() != expected_session {
        return Err(UiChangeClassificationDenial::ForeignSession);
    }
    if set.source_basis() != expected_source_basis {
        return Err(UiChangeClassificationDenial::ForeignSourceBasis);
    }
    Ok(())
}

fn append_bounded(
    facts: &mut Vec<UiProducedFact>,
    additions: Box<[UiProducedFact]>,
    limit: usize,
) -> Result<(), UiChangeClassificationDenial> {
    let observed = facts.len().saturating_add(additions.len());
    if observed > limit {
        return Err(UiChangeClassificationDenial::ChangedFactCapacityExceeded { limit, observed });
    }
    facts.extend(additions);
    Ok(())
}

fn push_bounded(
    facts: &mut Vec<UiProducedFact>,
    fact: UiProducedFact,
    limit: usize,
) -> Result<(), UiChangeClassificationDenial> {
    let observed = facts.len().saturating_add(1);
    if observed > limit {
        return Err(UiChangeClassificationDenial::ChangedFactCapacityExceeded { limit, observed });
    }
    facts.push(fact);
    Ok(())
}

fn resolve_outcome(
    basis: super::UiChangeClassificationBasis,
    facts: Vec<UiProducedFact>,
    source_succession: Option<super::UiAuthoredSourceSuccession>,
) -> UiChangeClassificationOutcome {
    if !facts.is_empty() {
        return UiChangeClassificationOutcome::Changed(UiClassifiedChange::new(
            basis,
            facts.into_boxed_slice(),
            source_succession,
        ));
    }
    match source_succession {
        Some(succession) => UiChangeClassificationOutcome::EvidenceOnly(
            UiEvidenceOnlySourceChange::new(basis, succession),
        ),
        None => {
            UiChangeClassificationOutcome::ObservedNoChange(UiObservedNoChangeReceipt::new(basis))
        }
    }
}

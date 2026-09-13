use super::super::super::observer_evidence::RecoveryObserverSelectorEvidence;
use super::super::super::observer_evidence_accumulation::RecoveryObserverSelectorObservation;

pub(crate) struct SelectorEvidenceAccumulator {
    selectors: Vec<RecoveryObserverSelectorObservation>,
}

impl SelectorEvidenceAccumulator {
    pub(crate) fn new() -> Self {
        Self {
            selectors: Vec::new(),
        }
    }

    pub(crate) fn observe(&mut self, selector: RecoveryObserverSelectorObservation) {
        self.selectors.push(selector);
    }

    pub(crate) fn finish(self) -> RecoveryObserverSelectorEvidence {
        super::super::selectors::summarize(self.selectors)
    }
}

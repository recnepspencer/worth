use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey;

use crate::domain_computation::application_outcome_identity::WorthQueryApplicationOutcomeIdentity;

/// Whether a query result's disclosure was public or governed by authorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationDisclosureReceiptPosture {
    /// The query has no governed disclosure; nothing was withheld by policy.
    Public,
    /// Disclosure was decided per result slot under a capability authorization.
    Governed,
}

/// The decision made for one result slot under governed disclosure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationDisclosureOutcome {
    /// The slot's value was disclosed to the caller.
    Disclosed,
    /// The slot's value was withheld from the caller.
    Omitted,
}

/// One governed disclosure decision: a result slot, the disclosure it
/// requires, and whether it was disclosed or omitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationDisclosureDecisionFact {
    slot: ApplicationQueryResultSlotKey,
    required_disclosure: AspectValue,
    outcome: WorthQueryApplicationDisclosureOutcome,
}

/// Record of what a query result disclosed and withheld, and under what
/// authorization.
///
/// Read it from a query access receipt's `disclosure()`. A public receipt is
/// empty. A governed receipt lists each slot decision, the disclosed and
/// omitted disclosure values, and the capability authorization that decided
/// them. It is descriptive evidence, not authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationDisclosureReceipt {
    outcome_identity: Option<WorthQueryApplicationDisclosureOutcomeIdentity>,
    posture: WorthQueryApplicationDisclosureReceiptPosture,
    classification: Option<String>,
    decisions: Vec<WorthQueryApplicationDisclosureDecisionFact>,
    disclosed: Vec<AspectValue>,
    omitted: Vec<AspectValue>,
    capability_authority_identity: Option<String>,
    decision_identity: Option<[u8; 32]>,
    authorization_decision_fact_count: usize,
}

impl WorthQueryApplicationDisclosureReceipt {
    pub(in crate::domain_computation) const fn public() -> Self {
        Self {
            outcome_identity: None,
            posture: WorthQueryApplicationDisclosureReceiptPosture::Public,
            classification: None,
            decisions: Vec::new(),
            disclosed: Vec::new(),
            omitted: Vec::new(),
            capability_authority_identity: None,
            decision_identity: None,
            authorization_decision_fact_count: 0,
        }
    }

    pub(in crate::domain_computation) fn governed(
        classification: impl Into<String>,
        disclosed: Vec<(ApplicationQueryResultSlotKey, AspectValue)>,
        omitted: Vec<(ApplicationQueryResultSlotKey, AspectValue)>,
        capability_authority_identity: impl Into<String>,
        decision_identity: [u8; 32],
        authorization_decision_fact_count: usize,
    ) -> Self {
        let mut decisions = disclosed
            .iter()
            .map(
                |(slot, required_disclosure)| WorthQueryApplicationDisclosureDecisionFact {
                    slot: slot.clone(),
                    required_disclosure: required_disclosure.clone(),
                    outcome: WorthQueryApplicationDisclosureOutcome::Disclosed,
                },
            )
            .chain(omitted.iter().map(|(slot, required_disclosure)| {
                WorthQueryApplicationDisclosureDecisionFact {
                    slot: slot.clone(),
                    required_disclosure: required_disclosure.clone(),
                    outcome: WorthQueryApplicationDisclosureOutcome::Omitted,
                }
            }))
            .collect::<Vec<_>>();
        decisions.sort_by(|left, right| left.slot.cmp(&right.slot));
        Self {
            outcome_identity: WorthQueryApplicationDisclosureOutcomeIdentity::mint(),
            posture: WorthQueryApplicationDisclosureReceiptPosture::Governed,
            classification: Some(classification.into()),
            decisions,
            disclosed: disclosed.into_iter().map(|(_, value)| value).collect(),
            omitted: omitted.into_iter().map(|(_, value)| value).collect(),
            capability_authority_identity: Some(capability_authority_identity.into()),
            decision_identity: Some(decision_identity),
            authorization_decision_fact_count,
        }
    }

    pub const fn posture(&self) -> WorthQueryApplicationDisclosureReceiptPosture {
        self.posture
    }

    pub const fn outcome_identity(&self) -> Option<WorthQueryApplicationDisclosureOutcomeIdentity> {
        self.outcome_identity
    }

    pub fn classification(&self) -> Option<&str> {
        self.classification.as_deref()
    }

    pub fn decisions(&self) -> &[WorthQueryApplicationDisclosureDecisionFact] {
        &self.decisions
    }

    pub const fn disclosure_decision_count(&self) -> usize {
        self.decisions.len()
    }

    pub fn disclosed(&self) -> &[AspectValue] {
        &self.disclosed
    }

    pub fn omitted(&self) -> &[AspectValue] {
        &self.omitted
    }

    pub const fn has_omissions(&self) -> bool {
        !self.omitted.is_empty()
    }

    pub fn capability_authority_identity(&self) -> Option<&str> {
        self.capability_authority_identity.as_deref()
    }

    pub const fn decision_identity(&self) -> Option<&[u8; 32]> {
        self.decision_identity.as_ref()
    }

    pub const fn authorization_decision_fact_count(&self) -> usize {
        self.authorization_decision_fact_count
    }
}

impl WorthQueryApplicationDisclosureDecisionFact {
    pub const fn slot(&self) -> &ApplicationQueryResultSlotKey {
        &self.slot
    }

    pub const fn required_disclosure(&self) -> &AspectValue {
        &self.required_disclosure
    }

    pub const fn outcome(&self) -> WorthQueryApplicationDisclosureOutcome {
        self.outcome
    }
}
/// Process-local identity of one governed disclosure outcome, for correlation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryApplicationDisclosureOutcomeIdentity(WorthQueryApplicationOutcomeIdentity);

impl WorthQueryApplicationDisclosureOutcomeIdentity {
    fn mint() -> Option<Self> {
        WorthQueryApplicationOutcomeIdentity::mint().map(Self)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

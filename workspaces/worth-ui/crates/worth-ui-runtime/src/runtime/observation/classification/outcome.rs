use crate::fact_contract::UiProducedFact;

pub enum UiChangeClassificationOutcome {
    ObservedNoChange(UiObservedNoChangeReceipt),
    EvidenceOnly(UiEvidenceOnlySourceChange),
    Changed(UiClassifiedChange),
}

pub struct UiObservedNoChangeReceipt {
    basis: super::UiChangeClassificationBasis,
}

pub struct UiEvidenceOnlySourceChange {
    basis: super::UiChangeClassificationBasis,
    succession: UiAuthoredSourceSuccession,
}

pub struct UiClassifiedChange {
    basis: super::UiChangeClassificationBasis,
    facts: Box<[UiProducedFact]>,
    source_succession: Option<UiAuthoredSourceSuccession>,
    theme_switch: Option<crate::runtime::appearance::UiThemeSwitchChange>,
}

pub(crate) enum UiAuthoredSourceClassification {
    ObservedNoChange,
    EvidenceOnly(UiAuthoredSourceSuccession),
    Changed {
        facts: Box<[UiProducedFact]>,
        succession: UiAuthoredSourceSuccession,
    },
}

pub(crate) enum UiAuthoredSourceSuccession {
    EvidenceOnly {
        successor_authority:
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        admitted_candidate: crate::runtime::WorthUiAdmittedReplacementCandidate,
        comparison: crate::runtime::WorthUiRuntimeArtifactComparison,
    },
    Changed {
        successor_authority:
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        comparison: crate::runtime::WorthUiRuntimeArtifactComparison,
        replacement: crate::runtime::replacement::WorthUiReplacementNodePlanReady,
        identity_lifecycle_index: crate::runtime::rebind::UiSourceIdentityLifecycleIndex,
    },
}

impl UiAuthoredSourceSuccession {
    pub(crate) fn successor_authority(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority {
        match self {
            Self::EvidenceOnly {
                successor_authority,
                ..
            }
            | Self::Changed {
                successor_authority,
                ..
            } => successor_authority,
        }
    }

    pub(crate) fn comparison(&self) -> &crate::runtime::WorthUiRuntimeArtifactComparison {
        match self {
            Self::EvidenceOnly { comparison, .. } | Self::Changed { comparison, .. } => comparison,
        }
    }

    pub(crate) fn admitted_candidate(
        &self,
    ) -> Option<&crate::runtime::WorthUiAdmittedReplacementCandidate> {
        match self {
            Self::EvidenceOnly {
                admitted_candidate, ..
            } => Some(admitted_candidate),
            Self::Changed { .. } => None,
        }
    }

    pub(crate) fn identity_lifecycle_index(
        &self,
    ) -> Option<&crate::runtime::rebind::UiSourceIdentityLifecycleIndex> {
        match self {
            Self::EvidenceOnly { .. } => None,
            Self::Changed {
                identity_lifecycle_index,
                ..
            } => Some(identity_lifecycle_index),
        }
    }

    pub(crate) fn into_changed_parts(
        self,
    ) -> Option<(
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        crate::runtime::WorthUiRuntimeArtifactComparison,
        crate::runtime::replacement::WorthUiReplacementNodePlanReady,
    )> {
        match self {
            Self::EvidenceOnly { .. } => None,
            Self::Changed {
                successor_authority,
                comparison,
                replacement,
                ..
            } => Some((successor_authority, comparison, replacement)),
        }
    }
}

impl UiObservedNoChangeReceipt {
    pub(crate) const fn new(basis: super::UiChangeClassificationBasis) -> Self {
        Self { basis }
    }

    pub const fn turn(&self) -> super::super::UiObservationTurnIdentity {
        self.basis.turn()
    }

    pub const fn observation_count(&self) -> usize {
        self.basis.observation_count()
    }

    pub const fn basis(&self) -> &super::UiChangeClassificationBasis {
        &self.basis
    }
}

impl UiEvidenceOnlySourceChange {
    pub(super) const fn new(
        basis: super::UiChangeClassificationBasis,
        succession: UiAuthoredSourceSuccession,
    ) -> Self {
        Self { basis, succession }
    }

    pub const fn basis(&self) -> &super::UiChangeClassificationBasis {
        &self.basis
    }

    pub fn active_artifact_digest(&self) -> u64 {
        self.succession.comparison().active_artifact_digest()
    }

    pub fn candidate_artifact_digest(&self) -> u64 {
        self.succession.comparison().candidate_artifact_digest()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::UiChangeClassificationBasis,
        UiAuthoredSourceSuccession,
    ) {
        (self.basis, self.succession)
    }
}

impl UiClassifiedChange {
    pub(super) fn new(
        basis: super::UiChangeClassificationBasis,
        facts: Box<[UiProducedFact]>,
        source_succession: Option<UiAuthoredSourceSuccession>,
    ) -> Self {
        debug_assert!(!facts.is_empty());
        Self {
            basis,
            facts,
            source_succession,
            theme_switch: None,
        }
    }

    pub(crate) fn theme_switch(&self) -> Option<&crate::runtime::appearance::UiThemeSwitchChange> {
        self.theme_switch.as_ref()
    }

    pub(crate) fn from_theme_switch(
        basis: super::UiChangeClassificationBasis,
        change: crate::runtime::appearance::UiThemeSwitchChange,
    ) -> Self {
        Self {
            basis,
            facts: Box::new([]),
            source_succession: None,
            theme_switch: Some(change),
        }
    }

    pub const fn basis(&self) -> &super::UiChangeClassificationBasis {
        &self.basis
    }

    pub fn facts(&self) -> &[UiProducedFact] {
        &self.facts
    }

    pub(crate) fn source_succession(&self) -> Option<&UiAuthoredSourceSuccession> {
        self.source_succession.as_ref()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::UiChangeClassificationBasis,
        Box<[UiProducedFact]>,
        Option<UiAuthoredSourceSuccession>,
        Option<crate::runtime::appearance::UiThemeSwitchChange>,
    ) {
        (
            self.basis,
            self.facts,
            self.source_succession,
            self.theme_switch,
        )
    }
}

use crate::declaration::UiAspectName;
use crate::fact_contract::UiProducedFact;
use crate::runtime::observation::UiAuthoredSourceSuccession;

use super::{UiAffectedConsumer, UiAffectedFactLookup, UiAffectedScopeBasis, UiAffectedScopeCost};

pub struct UiResolvedAffectedScope {
    basis: UiAffectedScopeBasis,
    facts: Box<[UiProducedFact]>,
    affected_aspects: Box<[UiAspectName]>,
    consumers: Box<[UiAffectedConsumer]>,
    lookups: Box<[UiAffectedFactLookup]>,
    cost: UiAffectedScopeCost,
    source_succession: Option<UiAuthoredSourceSuccession>,
    theme_switch: Option<crate::runtime::appearance::UiThemeSwitchChange>,
}

pub(crate) struct UiResolvedAffectedScopeInput {
    pub(crate) basis: UiAffectedScopeBasis,
    pub(crate) facts: Box<[UiProducedFact]>,
    pub(crate) affected_aspects: Box<[UiAspectName]>,
    pub(crate) consumers: Box<[UiAffectedConsumer]>,
    pub(crate) lookups: Box<[UiAffectedFactLookup]>,
    pub(crate) cost: UiAffectedScopeCost,
    pub(crate) source_succession: Option<UiAuthoredSourceSuccession>,
    pub(crate) theme_switch: Option<crate::runtime::appearance::UiThemeSwitchChange>,
}

impl UiResolvedAffectedScope {
    pub(crate) fn new(input: UiResolvedAffectedScopeInput) -> Self {
        Self {
            basis: input.basis,
            facts: input.facts,
            affected_aspects: input.affected_aspects,
            consumers: input.consumers,
            lookups: input.lookups,
            cost: input.cost,
            source_succession: input.source_succession,
            theme_switch: input.theme_switch,
        }
    }

    pub const fn basis(&self) -> &UiAffectedScopeBasis {
        &self.basis
    }

    pub fn facts(&self) -> &[UiProducedFact] {
        &self.facts
    }

    pub(crate) fn into_facts(self) -> Box<[UiProducedFact]> {
        self.facts
    }

    pub fn affected_aspects(&self) -> &[UiAspectName] {
        &self.affected_aspects
    }

    pub fn consumers(&self) -> &[UiAffectedConsumer] {
        &self.consumers
    }

    pub fn lookups(&self) -> &[UiAffectedFactLookup] {
        &self.lookups
    }

    pub const fn cost(&self) -> UiAffectedScopeCost {
        self.cost
    }

    pub(crate) fn source_succession(&self) -> Option<&UiAuthoredSourceSuccession> {
        self.source_succession.as_ref()
    }

    pub(crate) fn take_theme_switch(
        &mut self,
    ) -> Option<crate::runtime::appearance::UiThemeSwitchChange> {
        self.theme_switch.take()
    }

    pub(crate) fn take_source_succession(&mut self) -> Option<UiAuthoredSourceSuccession> {
        self.source_succession.take()
    }

    pub fn resolve_identity_lifecycle(
        self,
    ) -> Result<
        crate::runtime::rebind::UiResolvedIdentityLifecycle,
        crate::runtime::rebind::UiIdentityLifecycleDenial,
    > {
        crate::runtime::rebind::UiIdentityLifecycleResolver::resolve(self)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn replace_planning_session_for_certification(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    ) {
        self.basis.replace_session_for_certification(session);
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn replace_planning_predecessor_for_certification(
        &mut self,
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    ) {
        self.basis
            .replace_predecessor_generation_for_certification(generation);
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn replace_planning_candidate_for_certification(
        &mut self,
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    ) {
        self.basis
            .replace_candidate_generation_for_certification(generation);
    }
}

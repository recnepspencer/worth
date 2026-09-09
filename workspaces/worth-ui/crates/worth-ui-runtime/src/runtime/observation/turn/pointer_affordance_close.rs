pub(crate) struct UiPointerAffordanceObservationOwners<'owner> {
    pub(crate) clock: Option<&'owner worth_ui_host_native::UiNativeObservationClock>,
    pub(crate) generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    pub(crate) mounted: &'owner crate::mounting::WorthUiMountedSessionState,
    pub(crate) application_facts: &'owner crate::runtime::intent::UiIntentApplicationFactState,
    pub(crate) occupancy: &'owner crate::runtime::intent::UiIntentOccupancyState,
    pub(crate) confirmation: &'owner crate::runtime::intent::UiIntentConfirmationState,
    pub(crate) interaction: &'owner crate::runtime::interaction::UiInteractionRuntimeState,
}

pub(crate) struct UiPointerAffordanceObservationCloseInput<'owner> {
    owners: UiPointerAffordanceObservationOwners<'owner>,
    catalog: &'owner crate::declaration::UiIntentCatalog,
    definitions: &'owner crate::capability::FrozenIntentDefinitionCapabilities,
    bindings: &'owner crate::runtime::intent_execution::FrozenIntentExecutionBindings,
}

impl<'owner> UiPointerAffordanceObservationCloseInput<'owner> {
    pub(crate) fn new(
        owners: UiPointerAffordanceObservationOwners<'owner>,
        catalog: &'owner crate::declaration::UiIntentCatalog,
        definitions: &'owner crate::capability::FrozenIntentDefinitionCapabilities,
        bindings: &'owner crate::runtime::intent_execution::FrozenIntentExecutionBindings,
    ) -> Self {
        Self {
            owners,
            catalog,
            definitions,
            bindings,
        }
    }

    pub(super) fn has_clock(&self) -> bool {
        self.owners.clock.is_some()
    }

    pub(super) fn seal(
        self,
        authority: &super::UiObservationTurnCloseAuthority,
        turn: super::UiObservationTurnIdentity,
        source_basis: u64,
        host_time: Option<worth_ui_host_contract::UiHostObservationTimeBasis>,
    ) -> crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot {
        let owners = self.owners;
        let host_time = owners
            .clock
            .map(|clock| {
                worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(
                    clock.sample_millis(),
                )
            })
            .or(host_time);
        let presence = owners
            .interaction
            .pointer_presence_appearance_snapshot()
            .expect("installed pointer owner remains borrowed through close");
        crate::runtime::pointer_affordance::UiPointerAffordanceSnapshot::seal_pointer_at_turn_close(
            authority,
            turn,
            owners.generation.clone(),
            source_basis,
            &presence,
            |target| {
                crate::runtime::intent::observe_activation_operability(
                    target,
                    self.catalog,
                    self.definitions,
                    self.bindings,
                    &owners.generation,
                    owners.mounted,
                    owners.application_facts,
                    owners.occupancy,
                    owners.confirmation,
                    host_time,
                )
            },
        )
    }
}

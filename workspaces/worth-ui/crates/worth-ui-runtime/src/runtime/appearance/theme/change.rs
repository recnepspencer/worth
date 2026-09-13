use crate::runtime::appearance::UiAppearanceInvalidationBatch;

/// A classified binding change and its exact mounted dependency selection.
pub(crate) struct UiThemeSwitchChange {
    prepared: super::UiPreparedThemeSwitch,
    changed_slots: Box<[crate::capability::ThemeTokenId]>,
    invalidation: UiAppearanceInvalidationBatch,
    cost: UiThemeSwitchSelectionCost,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct UiThemeSwitchSelectionCost {
    pub(crate) theme_slots_compared: usize,
    pub(crate) index_probes: usize,
    pub(crate) graph_and_mounted_entries: usize,
}

#[derive(Debug, PartialEq)]
pub enum UiThemeSwitchSelectionDenial {
    MissingBundle,
    Resolution(super::UiThemeResolutionDenial),
    Consumer(crate::graph::UiGraphFactLookupDenial),
    Mounted(crate::mounting::UiMountedIdentityDenial),
}

impl UiThemeSwitchChange {
    pub(crate) fn prepare(
        prepared: super::UiPreparedThemeSwitch,
        predecessor: &super::UiActiveThemeBinding,
        application: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<Self, UiThemeSwitchSelectionDenial> {
        let themes = application
            .capabilities()
            .appearance_themes()
            .ok_or(UiThemeSwitchSelectionDenial::MissingBundle)?;
        let before =
            super::UiThemeResolutionView::from_capability(predecessor.capability(), themes)
                .map_err(UiThemeSwitchSelectionDenial::Resolution)?;
        let after = super::UiThemeResolutionView::from_capability(
            prepared.successor().capability(),
            themes,
        )
        .map_err(UiThemeSwitchSelectionDenial::Resolution)?;
        let mut changed_slots = Vec::new();
        let mut cost = UiThemeSwitchSelectionCost::default();
        // Bundle admission indexes deviations from the initial definition. The
        // union is a complete candidate set for any two admitted definitions,
        // so live switching never walks the whole slot catalog.
        let candidates = themes
            .transition_candidates(
                predecessor.capability().definition(),
                prepared.successor().capability().definition(),
            )
            .ok_or(UiThemeSwitchSelectionDenial::Resolution(
                super::UiThemeResolutionDenial::MissingDefinition,
            ))?;
        for candidate in candidates {
            let slot = themes.catalog().get(&candidate).ok_or(
                UiThemeSwitchSelectionDenial::Resolution(
                    super::UiThemeResolutionDenial::MissingSlot,
                ),
            )?;
            let identity = worth_ui_dsl::UiThemeSlotIdentity::new(slot.identity().as_str()).ok_or(
                UiThemeSwitchSelectionDenial::Resolution(
                    super::UiThemeResolutionDenial::InvalidSlotIdentity,
                ),
            )?;
            let before = before
                .resolve(&identity, slot.kind())
                .map_err(|failure| UiThemeSwitchSelectionDenial::Resolution(failure.denial()))?;
            let after = after
                .resolve(&identity, slot.kind())
                .map_err(|failure| UiThemeSwitchSelectionDenial::Resolution(failure.denial()))?;
            cost.theme_slots_compared += before.work().theme_slots_compared() as usize
                + after.work().theme_slots_compared() as usize;
            if before.value() != after.value() || before.terminal() != after.terminal() {
                changed_slots.push(slot.identity().clone());
            }
        }
        let mut change = Self {
            prepared,
            changed_slots: changed_slots.into_boxed_slice(),
            cost,
            invalidation: UiAppearanceInvalidationBatch::empty(application.consumed_fact_index()),
        };
        change.refresh(application, mounted, graph)?;
        Ok(change)
    }

    pub(crate) fn refresh(
        &mut self,
        application: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        graph: crate::graph::UiGraphAuthority<'_>,
    ) -> Result<(), UiThemeSwitchSelectionDenial> {
        let index = application.consumed_fact_index();
        let declarations = application.authored_declaration_lookup();
        let mut nodes = std::collections::BTreeSet::new();
        let mut probes = 0;
        let mut entries = 0;
        for slot in &self.changed_slots {
            let authored = declarations
                .theme_token_declaration_identity(slot.as_str())
                .unwrap_or(slot.as_str());
            let selection = index
                .select_appearance_slot_consumers(index.basis(), slot.as_str(), authored)
                .map_err(UiThemeSwitchSelectionDenial::Consumer)?;
            probes += 1 + selection.index_probes(); // includes authored declaration lookup
            entries += selection.entries_examined();
            nodes.extend(selection.into_consumers());
        }
        let nodes = nodes.into_iter().collect::<Vec<_>>();
        let (invalidation, mounted_entries) = UiAppearanceInvalidationBatch::theme_surface(
            index,
            mounted,
            graph,
            self.prepared.successor().surface(),
            &nodes,
        )
        .map_err(UiThemeSwitchSelectionDenial::Mounted)?;
        self.invalidation = invalidation;
        self.cost.index_probes = probes + nodes.len() * 2 + mounted_entries - nodes.len();
        self.cost.graph_and_mounted_entries = entries + mounted_entries;
        Ok(())
    }

    pub(crate) fn prepared(&self) -> &super::UiPreparedThemeSwitch {
        &self.prepared
    }
    pub(crate) fn invalidation(&self) -> &UiAppearanceInvalidationBatch {
        &self.invalidation
    }
    pub(crate) const fn cost(&self) -> UiThemeSwitchSelectionCost {
        self.cost
    }
    pub(crate) fn into_prepared(self) -> super::UiPreparedThemeSwitch {
        self.prepared
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceGenerationSuccessionDenial {
    StaleAdmission,
    StaleBinding(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    ForeignSession,
    OutstandingPreparedSwitch,
    InconsistentEmptyAdmission,
    BindingGenerationExhausted(worth_ui_host_contract::UiSemanticSurfaceIdentity),
}

pub(crate) struct UiPreparedAppearanceGenerationSuccession {
    predecessor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    successor: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    admission: Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
    bindings: Box<[crate::runtime::appearance::UiActiveThemeBinding]>,
    text_tokens: std::sync::Arc<
        std::collections::BTreeMap<
            crate::capability::ThemeTokenId,
            crate::capability::ThemeTokenValue,
        >,
    >,
}

impl UiPreparedAppearanceGenerationSuccession {
    pub(crate) fn semantic_text_theme_values(&self) -> crate::mounting::UiMountedThemeValueSource {
        crate::mounting::UiMountedThemeValueSource::from_admitted(std::sync::Arc::clone(
            &self.text_tokens,
        ))
    }

    pub(crate) fn predecessor(
        &self,
    ) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.predecessor
    }
    pub(crate) fn successor(&self) -> &crate::runtime::WorthUiActiveApplicationGenerationIdentity {
        &self.successor
    }
    pub(crate) fn binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&crate::runtime::appearance::UiActiveThemeBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.surface() == surface)
    }

    pub(crate) fn admission(
        &self,
    ) -> Option<&crate::runtime::appearance::UiPreparedThemeBindingAdmission> {
        self.admission.as_ref()
    }
}

fn map_rebinding_denial(
    denial: crate::runtime::appearance::UiThemeCapabilityReceiptDenial,
    active_bindings: &[crate::runtime::appearance::UiActiveThemeBinding],
) -> UiAppearanceGenerationSuccessionDenial {
    match denial {
        crate::runtime::appearance::UiThemeCapabilityReceiptDenial::BindingGenerationExhausted => {
            active_bindings
                .iter()
                .find(|binding| binding.binding_generation() == u64::MAX)
                .map_or(
                    UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                    |binding| {
                        UiAppearanceGenerationSuccessionDenial::BindingGenerationExhausted(
                            binding.surface(),
                        )
                    },
                )
        }
        crate::runtime::appearance::UiThemeCapabilityReceiptDenial::StaleBinding
        | crate::runtime::appearance::UiThemeCapabilityReceiptDenial::GenerationMismatch => {
            active_bindings.first().map_or(
                UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                |binding| UiAppearanceGenerationSuccessionDenial::StaleBinding(binding.surface()),
            )
        }
        _ => UiAppearanceGenerationSuccessionDenial::StaleAdmission,
    }
}

impl super::UiApplicationPresentationState {
    pub(crate) fn prepare_appearance_generation_succession(
        &self,
        succession: &crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationSuccession,
        predecessor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        admission: Option<&crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
    ) -> Result<UiPreparedAppearanceGenerationSuccession, UiAppearanceGenerationSuccessionDenial>
    {
        let predecessor_prepared = succession.predecessor();
        let successor_prepared = succession.successor();
        if predecessor.session_identity() != successor.session_identity()
            || predecessor.prepared_generation() != predecessor_prepared
            || successor.prepared_generation() != successor_prepared
        {
            return Err(UiAppearanceGenerationSuccessionDenial::ForeignSession);
        }
        if admission.is_some_and(|admission| admission.application() != predecessor) {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        let state = self.appearance_theme_state.as_ref();
        if state.is_some_and(|state| state.has_prepared_switches()) {
            return Err(UiAppearanceGenerationSuccessionDenial::OutstandingPreparedSwitch);
        }
        let active_bindings = state
            .map(|state| state.active_bindings().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        if admission.is_none() {
            if !active_bindings.is_empty() {
                return Err(UiAppearanceGenerationSuccessionDenial::InconsistentEmptyAdmission);
            }
            return Ok(UiPreparedAppearanceGenerationSuccession {
                text_tokens: std::sync::Arc::clone(&self.token_values),
                predecessor: predecessor.clone(),
                successor: successor.clone(),
                admission: None,
                bindings: Box::new([]),
            });
        }

        for binding in &active_bindings {
            if binding.surface() != binding.capability().surface()
                || binding.capability().application() != predecessor
            {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleBinding(
                    binding.surface(),
                ));
            }
        }
        let carried_bindings: Vec<crate::runtime::appearance::UiActiveThemeBinding> =
            active_bindings
                .iter()
                .map(|binding| binding.for_successor_application(successor.clone()))
                .collect::<Vec<_>>();
        Ok(UiPreparedAppearanceGenerationSuccession {
            text_tokens: std::sync::Arc::clone(&self.token_values),
            predecessor: predecessor.clone(),
            successor: successor.clone(),
            admission: Some(
                admission
                    .expect("appearance succession admission is present")
                    .for_successor_application(successor.clone()),
            ),
            bindings: carried_bindings.into_boxed_slice(),
        })
    }

    pub(crate) fn prepare_appearance_replacement_succession(
        &self,
        predecessor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        successor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        current_admission: Option<&crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
        successor_admission: Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
        rebinding: Option<&crate::runtime::appearance::UiPreparedThemeGenerationRebinding>,
        themes: Option<&crate::capability::FrozenAppearanceThemeCapabilities>,
        text_tokens: &crate::capability::FrozenThemeTokenCapabilities,
    ) -> Result<UiPreparedAppearanceGenerationSuccession, UiAppearanceGenerationSuccessionDenial>
    {
        if predecessor.session_identity() != successor.session_identity() {
            return Err(UiAppearanceGenerationSuccessionDenial::ForeignSession);
        }
        if self
            .appearance_theme_state
            .as_ref()
            .is_some_and(|state| state.has_prepared_switches())
        {
            return Err(UiAppearanceGenerationSuccessionDenial::OutstandingPreparedSwitch);
        }
        let Some(successor_admission) = successor_admission else {
            if rebinding.is_some() || themes.is_some() {
                return Err(UiAppearanceGenerationSuccessionDenial::InconsistentEmptyAdmission);
            }
            return Ok(UiPreparedAppearanceGenerationSuccession {
                text_tokens: super::semantic_text_tokens::admit(text_tokens),
                predecessor: predecessor.clone(),
                successor: successor.clone(),
                admission: None,
                bindings: Box::new([]),
            });
        };
        if successor_admission.application() != successor {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        if themes.is_none() {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        let active_bindings = self
            .appearance_theme_state
            .as_ref()
            .map(|state| state.active_bindings().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let stale_surface = active_bindings.first().map(|binding| binding.surface());
        if current_admission.is_some_and(|admission| admission.application() != predecessor)
            || (!active_bindings.is_empty() && current_admission.is_none())
        {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        let Some(rebinding) = rebinding else {
            if !active_bindings.is_empty() {
                return Err(stale_surface.map_or(
                    UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                    UiAppearanceGenerationSuccessionDenial::StaleBinding,
                ));
            }
            return Ok(UiPreparedAppearanceGenerationSuccession {
                text_tokens: super::semantic_text_tokens::admit(text_tokens),
                predecessor: predecessor.clone(),
                successor: successor.clone(),
                admission: Some(successor_admission),
                bindings: Box::new([]),
            });
        };
        if rebinding.predecessor() != predecessor || rebinding.successor() != successor {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        if rebinding.replacements().len() != active_bindings.len() {
            return Err(stale_surface.map_or(
                UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                UiAppearanceGenerationSuccessionDenial::StaleBinding,
            ));
        }
        for binding in &active_bindings {
            if binding.surface() != binding.capability().surface()
                || binding.capability().application() != predecessor
            {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleBinding(
                    binding.surface(),
                ));
            }
        }
        let successor_bindings = self
            .appearance_theme_state
            .as_ref()
            .ok_or_else(|| {
                stale_surface.map_or(
                    UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                    UiAppearanceGenerationSuccessionDenial::StaleBinding,
                )
            })?
            .prepare_generation_rebinding(rebinding)
            .map_err(|denial| map_rebinding_denial(denial, &active_bindings))?;
        Ok(UiPreparedAppearanceGenerationSuccession {
            text_tokens: super::semantic_text_tokens::admit(text_tokens),
            predecessor: predecessor.clone(),
            successor: successor.clone(),
            admission: Some(successor_admission),
            bindings: successor_bindings,
        })
    }

    pub(crate) fn commit_appearance_generation_succession(
        &mut self,
        prepared: UiPreparedAppearanceGenerationSuccession,
    ) {
        self.token_values = prepared.text_tokens;
        if prepared.admission.is_none() {
            self.appearance_theme_state = None;
        } else if let Some(state) = self.appearance_theme_state.as_mut() {
            state.replace_carried_bindings(prepared.bindings);
        } else if !prepared.bindings.is_empty() {
            let mut state = crate::runtime::appearance::UiAppearanceThemeState::default();
            state.replace_carried_bindings(prepared.bindings);
            self.appearance_theme_state = Some(state);
        }
    }
}

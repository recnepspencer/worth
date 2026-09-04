use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceGenerationSuccessionDenial {
    StaleAdmission,
    StaleBinding(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    StaleTypedValues(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    ForeignSession,
    OutstandingPreparedSwitch,
    InconsistentEmptyAdmission,
    SuccessorValueMissingSlot(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    SuccessorValueKindMismatch(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    SuccessorValueAliasTerminalChanged(worth_ui_host_contract::UiSemanticSurfaceIdentity),
    BindingGenerationExhausted(worth_ui_host_contract::UiSemanticSurfaceIdentity),
}

pub(crate) struct UiPreparedAppearanceGenerationSuccession {
    admission: Option<crate::runtime::appearance::UiPreparedThemeBindingAdmission>,
    bindings: Box<[crate::runtime::appearance::UiActiveThemeBinding]>,
    typed_values: BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        super::theme_values::UiApplicationThemeTypedValues,
    >,
}

impl UiPreparedAppearanceGenerationSuccession {
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

fn map_successor_value_denial(
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    denial: crate::runtime::appearance::UiThemeResolutionDenial,
) -> UiAppearanceGenerationSuccessionDenial {
    match denial {
        crate::runtime::appearance::UiThemeResolutionDenial::MissingSlot
        | crate::runtime::appearance::UiThemeResolutionDenial::InvalidSlotIdentity
        | crate::runtime::appearance::UiThemeResolutionDenial::MissingAliasTarget => {
            UiAppearanceGenerationSuccessionDenial::SuccessorValueMissingSlot(surface)
        }
        crate::runtime::appearance::UiThemeResolutionDenial::ValueKindMismatch => {
            UiAppearanceGenerationSuccessionDenial::SuccessorValueKindMismatch(surface)
        }
        _ => UiAppearanceGenerationSuccessionDenial::StaleTypedValues(surface),
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
            if !active_bindings.is_empty() || !self.appearance_theme_values.is_empty() {
                return Err(UiAppearanceGenerationSuccessionDenial::InconsistentEmptyAdmission);
            }
            return Ok(UiPreparedAppearanceGenerationSuccession {
                admission: None,
                bindings: Box::new([]),
                typed_values: BTreeMap::new(),
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
        for (surface, values) in &self.appearance_theme_values {
            let Some(binding) = active_bindings
                .iter()
                .find(|binding| binding.surface() == *surface)
            else {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            };
            if values.capability() != binding.capability() {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            }
        }

        let carried_bindings: Vec<crate::runtime::appearance::UiActiveThemeBinding> =
            active_bindings
                .iter()
                .map(|binding| binding.for_successor_application(successor.clone()))
                .collect::<Vec<_>>();
        let typed_values = self
            .appearance_theme_values
            .iter()
            .map(|(surface, values)| {
                let binding = carried_bindings
                    .iter()
                    .find(|binding| binding.surface() == *surface)
                    .expect("validated typed values retain their active binding");
                (
                    *surface,
                    values.for_successor_application(binding.capability()),
                )
            })
            .collect();
        Ok(UiPreparedAppearanceGenerationSuccession {
            admission: Some(
                admission
                    .expect("appearance succession admission is present")
                    .for_successor_application(successor.clone()),
            ),
            bindings: carried_bindings.into_boxed_slice(),
            typed_values,
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
                admission: None,
                bindings: Box::new([]),
                typed_values: BTreeMap::new(),
            });
        };
        if successor_admission.application() != successor {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        let Some(themes) = themes else {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        };
        let active_bindings = self
            .appearance_theme_state
            .as_ref()
            .map(|state| state.active_bindings().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let stale_surface = active_bindings
            .first()
            .map(|binding| binding.surface())
            .or_else(|| self.appearance_theme_values.keys().next().copied());
        if current_admission.is_some_and(|admission| admission.application() != predecessor)
            || (!active_bindings.is_empty() && current_admission.is_none())
        {
            return Err(UiAppearanceGenerationSuccessionDenial::StaleAdmission);
        }
        let Some(rebinding) = rebinding else {
            if !active_bindings.is_empty() || !self.appearance_theme_values.is_empty() {
                return Err(stale_surface.map_or(
                    UiAppearanceGenerationSuccessionDenial::StaleAdmission,
                    UiAppearanceGenerationSuccessionDenial::StaleBinding,
                ));
            }
            return Ok(UiPreparedAppearanceGenerationSuccession {
                admission: Some(successor_admission),
                bindings: Box::new([]),
                typed_values: BTreeMap::new(),
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
        let typed_values =
            self.prepare_successor_typed_values(&active_bindings, &successor_bindings, themes)?;
        Ok(UiPreparedAppearanceGenerationSuccession {
            admission: Some(successor_admission),
            bindings: successor_bindings,
            typed_values,
        })
    }

    fn prepare_successor_typed_values(
        &self,
        current_bindings: &[crate::runtime::appearance::UiActiveThemeBinding],
        successor_bindings: &[crate::runtime::appearance::UiActiveThemeBinding],
        themes: &crate::capability::FrozenAppearanceThemeCapabilities,
    ) -> Result<
        BTreeMap<
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            super::theme_values::UiApplicationThemeTypedValues,
        >,
        UiAppearanceGenerationSuccessionDenial,
    > {
        for (surface, values) in &self.appearance_theme_values {
            let mut matching = current_bindings
                .iter()
                .filter(|binding| binding.surface() == *surface);
            let Some(binding) = matching.next() else {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            };
            if matching.next().is_some()
                || values.capability().surface() != *surface
                || values.capability() != binding.capability()
            {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            }
        }

        let mut typed_values = BTreeMap::new();
        for (surface, values) in &self.appearance_theme_values {
            let mut matching = successor_bindings
                .iter()
                .filter(|binding| binding.surface() == *surface);
            let Some(binding) = matching.next() else {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            };
            if matching.next().is_some() || values.capability().surface() != *surface {
                return Err(UiAppearanceGenerationSuccessionDenial::StaleTypedValues(
                    *surface,
                ));
            }
            let view = crate::runtime::appearance::UiThemeResolutionView::from_capability(
                binding.capability(),
                themes,
            )
            .map_err(|_| UiAppearanceGenerationSuccessionDenial::StaleTypedValues(*surface))?;
            let values_map = values.values();
            for (terminal, value) in values_map.iter() {
                let requested = worth_ui_dsl::UiThemeSlotIdentity::new(terminal.as_str()).ok_or(
                    UiAppearanceGenerationSuccessionDenial::SuccessorValueMissingSlot(*surface),
                )?;
                let resolved = view
                    .resolve(&requested, value.kind())
                    .map_err(|denial| map_successor_value_denial(*surface, denial))?;
                if resolved.terminal() != &requested {
                    return Err(
                        UiAppearanceGenerationSuccessionDenial::SuccessorValueAliasTerminalChanged(
                            *surface,
                        ),
                    );
                }
                if resolved.value().kind() != value.kind() {
                    return Err(
                        UiAppearanceGenerationSuccessionDenial::SuccessorValueKindMismatch(
                            *surface,
                        ),
                    );
                }
            }
            typed_values.insert(
                *surface,
                values.for_successor_application(binding.capability()),
            );
        }
        Ok(typed_values)
    }

    pub(crate) fn commit_appearance_generation_succession(
        &mut self,
        prepared: UiPreparedAppearanceGenerationSuccession,
    ) {
        if prepared.admission.is_none() {
            self.appearance_theme_state = None;
        } else if let Some(state) = self.appearance_theme_state.as_mut() {
            state.replace_carried_bindings(prepared.bindings);
        } else if !prepared.bindings.is_empty() {
            let mut state = crate::runtime::appearance::UiAppearanceThemeState::default();
            state.replace_carried_bindings(prepared.bindings);
            self.appearance_theme_state = Some(state);
        }
        self.appearance_theme_values = prepared.typed_values;
    }
}

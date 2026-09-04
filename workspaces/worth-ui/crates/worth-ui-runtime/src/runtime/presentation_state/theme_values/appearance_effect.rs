use std::collections::BTreeMap;
use std::sync::Arc;

use super::UiApplicationThemeTypedValues;

pub(super) struct UiAppearanceThemeEffect {
    pub(super) values:
        BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, UiApplicationThemeTypedValues>,
    pub(super) changed_terminals: Vec<crate::capability::ThemeTokenId>,
}

struct BindingThemeSuccessor {
    binding: crate::runtime::appearance::UiActiveThemeBinding,
    values: BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>,
}

pub(super) fn prepare_successor(
    state: &crate::runtime::appearance::UiAppearanceThemeState,
    current_values: &BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        UiApplicationThemeTypedValues,
    >,
    themes: &crate::capability::FrozenAppearanceThemeCapabilities,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    changes: &[(crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue)],
) -> Result<UiAppearanceThemeEffect, ()> {
    let bindings = state.active_bindings().cloned().collect::<Vec<_>>();
    validate_binding_associations(&bindings, current_values, generation)?;

    let mut successors = Vec::with_capacity(bindings.len());
    let mut changed_terminals = Vec::new();
    for binding in bindings {
        let base_view = crate::runtime::appearance::UiThemeResolutionView::from_capability(
            binding.capability(),
            themes,
        )
        .map_err(|_| ())?;
        let mut values = current_values
            .get(&binding.surface())
            .map_or_else(BTreeMap::new, |current| (*current.values()).clone());
        let view = base_view
            .clone()
            .with_typed_values(Arc::new(values.clone()));
        validate_current_overrides(&view, &values)?;

        let mut canonical_changes = BTreeMap::new();
        for (token, value) in changes {
            let requested = worth_ui_dsl::UiThemeSlotIdentity::new(token.as_str()).ok_or(())?;
            let resolved = view.resolve(&requested, value.kind()).map_err(|_| ())?;
            let terminal = crate::capability::ThemeTokenId::new(resolved.terminal().as_str())
                .map_err(|_| ())?;
            if let Some(previous) = canonical_changes.get(&terminal) {
                if previous != value {
                    return Err(());
                }
            } else {
                if resolved.value() != *value {
                    changed_terminals.push(terminal.clone());
                }
                canonical_changes.insert(terminal, *value);
            }
        }
        values.extend(canonical_changes);
        successors.push(BindingThemeSuccessor { binding, values });
    }

    changed_terminals.sort();
    changed_terminals.dedup();
    let values = successors
        .into_iter()
        .map(|successor| {
            (
                successor.binding.surface(),
                UiApplicationThemeTypedValues::new(
                    successor.binding.capability(),
                    successor.values,
                ),
            )
        })
        .collect();
    Ok(UiAppearanceThemeEffect {
        values,
        changed_terminals,
    })
}

fn validate_binding_associations(
    bindings: &[crate::runtime::appearance::UiActiveThemeBinding],
    current_values: &BTreeMap<
        worth_ui_host_contract::UiSemanticSurfaceIdentity,
        UiApplicationThemeTypedValues,
    >,
    generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
) -> Result<(), ()> {
    if bindings.iter().any(|binding| {
        binding.binding_generation() == 0
            || binding.surface() != binding.capability().surface()
            || binding.capability().application() != generation
    }) {
        return Err(());
    }
    for (surface, values) in current_values {
        let Some(binding) = bindings
            .iter()
            .find(|binding| binding.surface() == *surface)
        else {
            return Err(());
        };
        if values.capability() != binding.capability() {
            return Err(());
        }
    }
    Ok(())
}

fn validate_current_overrides(
    view: &crate::runtime::appearance::UiThemeResolutionView,
    values: &BTreeMap<crate::capability::ThemeTokenId, worth_ui_dsl::UiThemeValue>,
) -> Result<(), ()> {
    for (terminal, value) in values {
        let requested = worth_ui_dsl::UiThemeSlotIdentity::new(terminal.as_str()).ok_or(())?;
        let resolved = view.resolve(&requested, value.kind()).map_err(|_| ())?;
        if resolved.terminal() != &requested || resolved.value() != *value {
            return Err(());
        }
    }
    Ok(())
}

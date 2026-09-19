pub(super) fn admit(
    tokens: &crate::capability::FrozenThemeTokenCapabilities,
) -> std::sync::Arc<
    std::collections::BTreeMap<crate::capability::ThemeTokenId, crate::capability::ThemeTokenValue>,
> {
    std::sync::Arc::new(
        tokens
            .entries()
            .iter()
            .map(|entry| {
                let value = tokens
                    .get(entry.resolved_target_id())
                    .and_then(crate::capability::ThemeTokenDescriptor::value)
                    .expect("frozen theme-token alias target has a value")
                    .clone();
                (entry.descriptor().id().clone(), value)
            })
            .collect(),
    )
}

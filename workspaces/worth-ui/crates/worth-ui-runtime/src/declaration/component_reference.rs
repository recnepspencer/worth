#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiDeclarationComponentReferenceDenial {
    InvalidIdentity,
    UnknownComponent,
}

pub(crate) fn admit_component_reference(
    reference: &worth_ui_dsl::UiDslComponentReference,
    snapshot: &crate::capability::CapabilitySnapshot,
) -> Result<crate::capability::ComponentId, UiDeclarationComponentReferenceDenial> {
    let component = crate::capability::ComponentId::new(reference.as_str())
        .map_err(|_| UiDeclarationComponentReferenceDenial::InvalidIdentity)?;
    snapshot
        .components()
        .get(&component)
        .ok_or(UiDeclarationComponentReferenceDenial::UnknownComponent)?;
    Ok(component)
}

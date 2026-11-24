use super::EntityAuthoritativeAspectStateDenial;

pub(super) fn entity_authoritative_aspect_state_denial_detail(
    denial: &EntityAuthoritativeAspectStateDenial,
) -> String {
    match denial {
        EntityAuthoritativeAspectStateDenial::MissingAspectContract { aspect_key } => format!(
            "entity authoritative state references undeclared aspect {}",
            aspect_key.as_str()
        ),

        EntityAuthoritativeAspectStateDenial::PatchConstructionDenied { denial } => format!(
            "authoritative aspect admission could not construct foundational patch: {:?}",
            denial
        ),
    }
}

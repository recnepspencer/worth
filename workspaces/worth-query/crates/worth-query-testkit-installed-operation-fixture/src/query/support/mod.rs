pub use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey};
pub use worth_query::facade::runtime::WorthQueryAspectTouch;

pub mod installed_domain;
pub mod public_bridge_runtime;
pub mod task_backend_schema;
pub mod test_entity_identities;

pub fn aspect_touch(authored_touch_text: &str) -> WorthQueryAspectTouch {
    let mut segments = authored_touch_text.split('.');
    let aspect = segments
        .next()
        .and_then(AspectKey::new)
        .expect("fixture authored touch aspect should admit");
    let fields = segments
        .map(|segment| FieldKey::new(segment).expect("fixture authored touch field should admit"))
        .collect::<Vec<_>>();
    if fields.is_empty() {
        WorthQueryAspectTouch::whole_aspect(aspect)
    } else {
        WorthQueryAspectTouch::aspect_field_path(
            aspect,
            CanonicalFieldPath::new(fields).expect("fixture authored touch should have fields"),
        )
    }
}

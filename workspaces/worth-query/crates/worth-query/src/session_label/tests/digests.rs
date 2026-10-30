use super::super::WorthQuerySessionLabelError;
use super::{render_collision_labels, typed_temporal_preview_label};

#[test]
fn canonical_session_label_outputs_are_non_empty_and_stable() {
    let typed = typed_temporal_preview_label();
    let strings =
        super::super::WorthQuerySessionLabel::scoped_strs("worth-kernel", ["temporal", "preview"])
            .expect("string label should build");
    let (render_collision_left, render_collision_right) = render_collision_labels();

    let session_label_identity_digest = typed.identity_digest().as_str().to_string();
    let session_label_scope_token = typed.identity_digest().scope().as_str().to_string();
    let session_label_display = typed.display().to_string();
    let failure_digest = crate::identity::hash_parts(&[
        WorthQuerySessionLabelError::EmptyNamespace.to_string(),
        WorthQuerySessionLabelError::EmptyNameSegment.to_string(),
        WorthQuerySessionLabelError::MissingNameSegments.to_string(),
    ]);

    assert!(!session_label_identity_digest.is_empty());
    assert!(!session_label_scope_token.is_empty());
    assert!(!session_label_display.is_empty());
    assert!(!failure_digest.is_empty());

    assert_eq!(typed.identity_digest(), strings.identity_digest());
    assert_eq!(typed.display(), strings.display());
    assert_eq!(session_label_scope_token, "session-label-identity");
    assert_eq!(session_label_display, "worth-kernel.temporal.preview");

    assert_eq!(
        render_collision_left.display(),
        render_collision_right.display()
    );
    assert_ne!(
        render_collision_left.identity_digest(),
        render_collision_right.identity_digest()
    );

    assert_ne!(session_label_identity_digest, failure_digest);
}

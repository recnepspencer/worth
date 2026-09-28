use sha2::Digest;
use std::sync::Arc;
use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{UiQualifiedFontFaceIdentity, UiQualifiedFontFamilyIdentity};

use crate::layout_artifact::{
    UiQualifiedTextColorGlyph, UiQualifiedTextColorSource, UiQualifiedTextFaceResource,
};

fn face(source: UiQualifiedTextColorSource) -> UiQualifiedTextFaceResource {
    UiQualifiedTextFaceResource::new(
        UiQualifiedFontFaceIdentity::from_text_mechanics([1; 32], 0),
        UiQualifiedFontFamilyIdentity::from_text_mechanics([2; 32]),
        None,
        Arc::from([0_u8]),
        true,
        vec![
            UiQualifiedTextColorGlyph::new(7, source),
            UiQualifiedTextColorGlyph::new(8, UiQualifiedTextColorSource::Bitmap),
        ]
        .into_boxed_slice(),
    )
}

fn face_digest(face: &UiQualifiedTextFaceResource) -> [u8; 32] {
    let mut digest = UiCountedSha256::new();
    super::hash_faces(&mut digest, std::slice::from_ref(face));
    digest.finalize().into()
}

#[test]
fn layout_resource_keeps_glyph_source_twins_and_source_substitution_changes_identity() {
    let outline = face(UiQualifiedTextColorSource::Outline);
    let bitmap = face(UiQualifiedTextColorSource::Bitmap);
    assert_eq!(
        outline.color_source(7),
        Some(UiQualifiedTextColorSource::Outline)
    );
    assert_eq!(
        outline.color_source(8),
        Some(UiQualifiedTextColorSource::Bitmap)
    );
    assert_ne!(face_digest(&outline), face_digest(&bitmap));
}

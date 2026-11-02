//! Pure registry rebinding cannot change the portable content contract.
use super::*;
use crate::content::*;
use std::{any::Any, sync::Arc};
use worth_foundational::facade::{CanonicalDigestId, ContentFormat};

#[derive(Debug)]
struct RefusingCodec {
    format: ContentFormat,
    maximum: usize,
}
impl RelationalContentCodec for RefusingCodec {
    fn format(&self) -> &ContentFormat {
        &self.format
    }
    fn maximum_frame_bytes(&self) -> usize {
        self.maximum
    }
    fn payload_length(
        &self,
        _: &dyn Any,
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        Err(RelationalContentDenial::InputTypeMismatch)
    }
    fn encode(
        &self,
        _: &dyn Any,
        _: &mut RelationalContentSink<'_>,
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        Err(RelationalContentDenial::InputTypeMismatch)
    }
    fn validate(
        &self,
        _: &[u8],
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        Err(RelationalContentDenial::NonCanonicalContent)
    }
    fn verify_encoding(
        &self,
        _: &dyn Any,
        _: &[u8],
        _: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        Err(RelationalContentDenial::EncodingInconsistency)
    }
}
fn registry(version: u32, maximum: usize, alias: u8) -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_content_format(
            CanonicalDigestId::new([1; 32]),
            Arc::new(RefusingCodec {
                format: ContentFormat::new("sample-grid", version).unwrap(),
                maximum,
            }),
        )
        .unwrap()
        .bind_content_field(
            CanonicalDigestId::new([alias; 32]),
            CanonicalDigestId::new([1; 32]),
        )
        .unwrap()
}

#[test]
fn content_schema_descriptor_ceiling_and_alias_cannot_rebind_old_authority() {
    let old = registry(1, 4096, 2);
    let snapshot = old.authority_snapshot();
    assert!(registry_for_snapshot(&old, &snapshot).is_some());
    for changed in [
        registry(2, 4096, 2),
        registry(1, 2048, 2),
        registry(1, 4096, 3),
    ] {
        assert_eq!(
            snapshot.entity_kinds,
            changed.authority_snapshot().entity_kinds
        );
        assert_eq!(
            snapshot.relation_kinds,
            changed.authority_snapshot().relation_kinds
        );
        assert_ne!(
            snapshot.content_formats,
            changed.authority_snapshot().content_formats
        );
        assert_ne!(
            old.authority_digest_bytes(),
            changed.authority_digest_bytes()
        );
        assert!(registry_for_snapshot(&changed, &snapshot).is_none());
        assert!(!crate::branch::root::schema_meaning::registry_meaning_matches(&old, &changed));
    }
}

#[test]
fn content_recorded_declarations_cannot_resolve_a_handle_until_exact_installed_binding() {
    let available = registry(1, 4096, 2);
    let recorded =
        RelationalContentFormatRegistry::from_recorded(available.content_formats.snapshot())
            .unwrap();
    let id = CanonicalDigestId::new([1; 32]);
    assert!(matches!(
        recorded.get(id),
        Err(RelationalContentDenial::FormatMismatch)
    ));
    assert!(recorded
        .bind_recorded(&available.content_formats)
        .unwrap()
        .get(id)
        .is_ok());
    assert!(matches!(
        recorded.bind_recorded(&registry(2, 4096, 2).content_formats),
        Err(RelationalContentDenial::UnsupportedVersion)
    ));
    assert!(matches!(
        recorded.bind_recorded(&registry(1, 2048, 2).content_formats),
        Err(RelationalContentDenial::FormatMismatch)
    ));
}

#[test]
fn content_schema_compact_roundtrip_preserves_empty_and_nonempty_optional_tails() {
    for snapshot in [
        RelationalSchemaRegistry::new().authority_snapshot(),
        registry(1, 4096, 2).authority_snapshot(),
    ] {
        let bytes = rmp_serde::to_vec(&snapshot).unwrap();
        let decoded: SchemaAuthoritySnapshot = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded, snapshot);
        assert_eq!(
            crate::schema::data::schema_authority_snapshot_digest_bytes(&decoded),
            crate::schema::data::schema_authority_snapshot_digest_bytes(&snapshot)
        );
    }
}

#[test]
fn content_schema_carrier_named_and_compact_preserve_every_optional_tail_position() {
    use crate::durability::data::DurableBranchRootSchemaImage;
    for registry in [RelationalSchemaRegistry::new(), registry(1, 4096, 2)] {
        let authority = RelationalBranchRootSchemaAuthority::capture(
            1,
            &registry,
            &registry.authority_snapshot(),
            crate::schema::data::runtime_descriptor_semantics_policy().current_write_version(),
            None,
        )
        .unwrap();
        let image = DurableBranchRootSchemaImage::capture(&authority).unwrap();
        for bytes in [
            rmp_serde::to_vec(&image).unwrap(),
            rmp_serde::to_vec_named(&image).unwrap(),
        ] {
            let decoded: DurableBranchRootSchemaImage = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(decoded, image);
            let (restored, _) = decoded.readmit_registry().unwrap();
            assert_eq!(restored.authority_snapshot(), registry.authority_snapshot());
        }
    }
}

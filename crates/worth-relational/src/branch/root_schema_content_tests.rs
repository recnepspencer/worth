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
    // Python hashlib over manually specified domains/counts; no Rust serializer.
    assert_eq!(
        RelationalSchemaRegistry::new().authority_digest_bytes(),
        [
            0xd8, 0x89, 0x18, 0xc5, 0xd7, 0xc1, 0xc3, 0x1b, 0xc3, 0xdf, 0x72, 0xd4, 0x5e, 0xac,
            0x0c, 0x68, 0x34, 0x88, 0xdb, 0xff, 0x01, 0x8b, 0xca, 0x17, 0x26, 0xe0, 0x3b, 0x70,
            0x04, 0x68, 0x57, 0x07,
        ]
    );
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
        assert_eq!(bytes[0], 0x97, "all seven positional fields are present");
        let decoded: SchemaAuthoritySnapshot = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(decoded, snapshot);
        assert_eq!(
            crate::schema::data::schema_authority_snapshot_digest_bytes(&decoded),
            crate::schema::data::schema_authority_snapshot_digest_bytes(&snapshot)
        );
        let mut json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_value::<SchemaAuthoritySnapshot>(json.clone()).unwrap(),
            snapshot
        );
        assert!(json
            .as_object_mut()
            .unwrap()
            .remove("content_formats")
            .is_some());
        assert_eq!(
            serde_json::from_value::<SchemaAuthoritySnapshot>(json)
                .unwrap_err()
                .to_string(),
            "missing field `content_formats`"
        );
        if snapshot.content_formats.is_empty() {
            assert_eq!(&bytes[bytes.len() - 3..], &[0x92, 0x90, 0x90]);
            let mut missing = bytes;
            missing.truncate(missing.len() - 3);
            missing[0] = 0x96;
            assert!(rmp_serde::from_slice::<SchemaAuthoritySnapshot>(&missing).is_err());
        }
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
        let mut json = serde_json::to_value(&image).unwrap();
        assert_eq!(
            serde_json::from_value::<DurableBranchRootSchemaImage>(json.clone()).unwrap(),
            image
        );
        assert!(json
            .as_object_mut()
            .unwrap()
            .remove("content_formats")
            .is_some());
        assert_eq!(
            serde_json::from_value::<DurableBranchRootSchemaImage>(json)
                .unwrap_err()
                .to_string(),
            "missing field `content_formats`"
        );
        let compact = rmp_serde::to_vec(&image).unwrap();
        assert_eq!(compact[0], 0x9a, "all ten positional fields are present");
        if registry.content_formats.snapshot().is_empty() {
            // Manual MessagePack tokens plus the independently frozen empty root.
            assert_eq!(
                image.carrier_digest(),
                [
                    0xf3, 0x82, 0x19, 0x00, 0xde, 0xb7, 0xea, 0x73, 0xbd, 0x91, 0xb4, 0x77, 0x78,
                    0x99, 0xc0, 0x03, 0xa4, 0xc5, 0x21, 0x8a, 0x57, 0x19, 0x09, 0x35, 0x24, 0x21,
                    0xa9, 0x9a, 0x49, 0x72, 0x46, 0xb2,
                ]
            );
            assert_eq!(&compact[compact.len() - 3..], &[0x92, 0x90, 0x90]);
            let mut missing = compact.clone();
            missing.truncate(missing.len() - 3);
            missing[0] = 0x99;
            assert!(rmp_serde::from_slice::<DurableBranchRootSchemaImage>(&missing).is_err());
        }
        for bytes in [compact, rmp_serde::to_vec_named(&image).unwrap()] {
            let decoded: DurableBranchRootSchemaImage = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(decoded, image);
            let (restored, _) = decoded.readmit_registry().unwrap();
            assert_eq!(restored.authority_snapshot(), registry.authority_snapshot());
        }
    }
}

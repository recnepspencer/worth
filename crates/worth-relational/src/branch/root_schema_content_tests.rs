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
fn content_readmission_refuses_registered_formats_without_reference_fields() {
    let available = registry(1, 4096, 2);
    assert!(available.authority_snapshot().entity_kinds.is_empty());
    assert!(matches!(
        crate::content::require_content_free_readmission(&available),
        Err(crate::branch::RelationalBranchRootCaptureDenial::UnadmittedContent)
    ));
    assert!(
        crate::content::require_content_free_readmission(&RelationalSchemaRegistry::new()).is_ok()
    );
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

//! Fixed contractual bytes exercise the public decoder without its serializer.
use super::*;

const DOCUMENT_HISTORY_HEX: &str = concat!(
    "0012000800000000000001b40000000f617263686976652e686973746f72790000000e446f63756d656e74536368656d",
    "610000000100000000000000010033000100020000000f446f63756d656e74486973746f727900000021776f7274682e",
    "617263686976652e646f63756d656e742d686973746f72792e763100000018776f7274682e617263686976652e62696e",
    "64696e672e763100000016776f7274682e617263686976652e696e7075742e76310000000c52656164446f63756d656e",
    "7400000016776f7274682e617263686976652e71756572792e76310000001b776f7274682e617263686976652e706172",
    "616d65746572732e763100000017776f7274682e617263686976652e726573756c742e763100000016776f7274682e61",
    "7263686976652e73636f70652e763100000008446f63756d656e74000000084964656e74697479000000034b65790000",
    "0014776f7274682e617263686976652e6b65792e76310000000b696e7075742d6669656c64000000095072696e636970",
    "616c0001000000067075626c6963000000000000000000000002000000055a45646974000000064d6f7665205a000000",
    "054145646974000000064d6f76652041",
);

fn header_offset(bytes: &[u8]) -> usize {
    bytes
        .windows(4)
        .position(|window| window == [0, 51, 0, 1])
        .unwrap()
}

#[test]
fn frozen_history_frame_preserves_exact_declaration_without_installing_a_runtime() {
    let original = current_frame();
    for (tag, domain) in [
        (1, ApplicationHistoryModelDomain::WholeComposite),
        (2, ApplicationHistoryModelDomain::ScopeEntityRecord),
    ] {
        let mut bytes = original.clone();
        let offset = header_offset(&bytes);
        bytes[offset + 5] = tag;
        let mut decoder =
            WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
        let decoded = decoder.decode_frame(&bytes).unwrap();
        assert_eq!(decoded.canonical_index(), 0);
        let WorthQueryPortablePackageRecord::ApplicationSchema(schema) = decoded.record() else {
            panic!("the frame must remain an application schema description")
        };
        assert_eq!(
            (
                schema.owner(),
                schema.name(),
                schema.major(),
                schema.minor()
            ),
            ("archive.history", "DocumentSchema", 1, 0)
        );
        assert_eq!(schema.members().len(), 1);
        let ApplicationSchemaMember::ApplicationHistory { definition } = &schema.members()[0]
        else {
            panic!("a history definition must not become tag36's history query")
        };
        definition.validate().unwrap();
        let parts = definition.parts();
        assert_eq!(parts.identity, "DocumentHistory");
        assert_eq!(
            parts.history_type.as_str(),
            "worth.archive.document-history.v1"
        );
        assert_eq!(parts.model_domain, domain);
        assert_eq!(
            parts
                .access
                .canonical_fields()
                .into_iter()
                .map(|(_, value)| value)
                .collect::<Vec<_>>(),
            vec![
                "worth.archive.binding.v1",
                "worth.archive.input.v1",
                "ReadDocument",
                "worth.archive.query.v1",
                "worth.archive.parameters.v1",
                "worth.archive.result.v1",
                "worth.archive.scope.v1",
                "Document",
                "Identity",
                "Key",
                "worth.archive.key.v1",
                "input-field",
                "Principal",
            ]
        );
        assert_eq!(
            parts.metadata_disclosure.posture(),
            ApplicationQueryDisclosurePosture::Public
        );
        assert_eq!(parts.metadata_disclosure.classification(), "public");
        assert!(parts.metadata_disclosure.rules().is_empty());
        assert!(parts.metadata_disclosure.capability_name().is_none());
        assert!(parts.metadata_disclosure.capability_identity().is_none());
        assert_eq!(
            parts
                .operations
                .iter()
                .map(|operation| (operation.binding.as_str(), operation.label.as_str()))
                .collect::<Vec<_>>(),
            vec![("ZEdit", "Move Z"), ("AEdit", "Move A")]
        );
        assert!(decoder.work().nested_entries() > 0);
    }
}

#[test]
fn history_frame_rejects_unsupported_headers_and_duplicate_meaning_without_committing_work() {
    let original = current_frame();
    let offset = header_offset(&original);
    for field in [2, 4] {
        let mut bytes = original.clone();
        bytes[offset + field..offset + field + 2].copy_from_slice(&99_u16.to_be_bytes());
        let mut decoder =
            WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
        assert_eq!(
            decoder.decode_frame(&bytes).unwrap_err().kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
        );
        assert_eq!(
            decoder.work(),
            WorthQueryPackageArchiveDecodeWork::default()
        );
    }
    let mut bytes = original;
    let operation = bytes
        .windows(5)
        .position(|window| window == b"ZEdit")
        .unwrap();
    bytes[operation] = b'A';
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn history_frame_depth_and_entry_limits_preserve_the_original_decoder_on_refusal() {
    let bytes = current_frame();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    for (limits, kind) in [
        (
            defaults.with_maximum_nesting_depth(1),
            WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded,
        ),
        (
            defaults.with_maximum_nested_entries(1),
            WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
        ),
    ] {
        let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
        for _ in 0..2 {
            assert_eq!(decoder.decode_frame(&bytes).unwrap_err().kind(), kind);
            assert_eq!(
                decoder.work(),
                WorthQueryPackageArchiveDecodeWork::default()
            );
        }
    }
    WorthQueryPackageArchiveRecordDecoder::new(defaults)
        .decode_frame(&bytes)
        .unwrap();
}

fn current_frame() -> Vec<u8> {
    let mut bytes = decode_hex(DOCUMENT_HISTORY_HEX);
    assert_eq!(&bytes[..2], &18_u16.to_be_bytes());
    bytes[..2].copy_from_slice(&19_u16.to_be_bytes());
    bytes
}

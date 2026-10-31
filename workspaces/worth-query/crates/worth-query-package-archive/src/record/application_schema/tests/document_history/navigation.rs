//! Independent member grammar for the explicit navigation extension and legacy twin.
use super::*;
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryNavigationDeclaration,
        WorthQueryPortableApplicationHistoryNavigationParts,
    },
    portable_identity::WorthQueryPortableTypeIdentity as Identity,
};

fn navigation() -> ApplicationHistoryNavigationDeclaration {
    ApplicationHistoryNavigationDeclaration::from_untrusted_parts(
        WorthQueryPortableApplicationHistoryNavigationParts {
            binding: "NavigateDocument".into(),
            capability: "NavigationPermission".into(),
            capability_type: Identity::declared("example.document-navigation.capability.v1"),
            operation: "SelectDocumentSnapshot".into(),
            input_type: Identity::declared("example.document-navigation.input.v1"),
        },
    )
}

// Unchanged access/disclosure codecs remain their own owners. This test writes
// the member version, domain, operation order and navigation fields directly;
// it never asks the document-history writer to create its decode input.
fn frame(version: u16, navigation: bool) -> Vec<u8> {
    let definition = definition(Domain::ScopeEntityRecord, false);
    let parts = definition.parts();
    let mut output = BinaryOutput::with_capacity(0);
    output.u16(51);
    output.u16(version);
    output.u16(2);
    output.text("DocumentHistory");
    output.text(parts.history_type.as_str());
    super::super::super::query_binding_reference::write(&mut output, &parts.access).unwrap();
    crate::record::application_query::disclosure::write(&mut output, &parts.metadata_disclosure)
        .unwrap();
    output.u32(2);
    output.text("ZEdit");
    output.text("Move Z");
    output.text("AEdit");
    output.text("Move A");
    if navigation {
        output.text("NavigateDocument");
        output.text("NavigationPermission");
        output.text("example.document-navigation.capability.v1");
        output.text("SelectDocumentSnapshot");
        output.text("example.document-navigation.input.v1");
    }
    output.into_bytes()
}

#[test]
fn absent_navigation_preserves_the_exact_legacy_member_and_decodes_disabled() {
    let legacy = frame(1, false);
    let original = ApplicationSchemaMember::ApplicationHistory {
        definition: definition(Domain::ScopeEntityRecord, false),
    };
    assert_eq!(encode_member(&original), legacy);
    assert_eq!(decode_member(&legacy), original);
    let ApplicationSchemaMember::ApplicationHistory { definition } = decode_member(&legacy) else {
        panic!("legacy member retains its family")
    };
    assert!(definition.parts().navigation.is_none());
}

#[test]
fn independently_written_navigation_member_retains_every_policy_reference() {
    let bytes = frame(2, true);
    let mut parts = definition(Domain::ScopeEntityRecord, false).parts().clone();
    parts.navigation = Some(navigation());
    let expected = ApplicationSchemaMember::ApplicationHistory {
        definition: Definition::from_untrusted_parts(parts),
    };
    assert_eq!(decode_member(&bytes), expected);
    assert_eq!(encode_member(&expected), bytes);
    assert_ne!(bytes, frame(1, false));
}

#[test]
fn navigation_version_requires_complete_valid_explicit_scope_meaning() {
    let mut missing = frame(1, false);
    missing[3] = 2;
    assert_eq!(
        try_decode_member(&missing).unwrap_err().kind(),
        Kind::Truncated
    );
    let complete = frame(2, true);
    for cut in missing.len()..complete.len() {
        assert_eq!(
            try_decode_member(&complete[..cut]).unwrap_err().kind(),
            Kind::Truncated,
            "navigation prefix {cut}"
        );
    }
    let mut parts = definition(Domain::WholeComposite, false).parts().clone();
    parts.navigation = Some(navigation());
    let mut output = BinaryOutput::with_capacity(0);
    let unsupported = ApplicationSchemaMember::ApplicationHistory {
        definition: Definition::from_untrusted_parts(parts),
    };
    assert_eq!(
        super::super::super::member::write(&mut output, &unsupported)
            .unwrap_err()
            .kind(),
        Kind::InvalidRecordShape
    );
    assert!(output.into_bytes().is_empty());
    let mut invalid = complete;
    let start = invalid
        .windows(16)
        .position(|bytes| bytes == b"NavigateDocument")
        .unwrap();
    invalid[start] = b' ';
    assert_eq!(
        try_decode_member(&invalid).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
}

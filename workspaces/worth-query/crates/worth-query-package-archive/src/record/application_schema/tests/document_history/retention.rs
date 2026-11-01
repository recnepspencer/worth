//! Independent extension grammar protects role position, flags and legacy bytes.
use super::*;
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryRetentionDeclaration as Policy,
        WorthQueryPortableApplicationHistoryRetentionParts as PolicyParts,
    },
    portable_identity::WorthQueryPortableTypeIdentity as Identity,
};

fn policy(role: &str) -> Policy {
    Policy::from_untrusted_parts(PolicyParts {
        binding: format!("{role}Policy"),
        capability: format!("{role}Permission"),
        capability_type: Identity::declared("example.retention.capability.v1"),
        operation: format!("Change{role}"),
        input_type: Identity::declared("example.retention.input.v1"),
    })
}
fn frame(flags: u16) -> Vec<u8> {
    let mut bytes = navigation::frame(1, false);
    bytes[3] = 3;
    let mut tail = BinaryOutput::with_capacity(0);
    tail.u16(flags);
    if flags & 1 != 0 {
        tail.text("NavigateDocument");
        tail.text("NavigationPermission");
        tail.text("example.document-navigation.capability.v1");
        tail.text("SelectDocumentSnapshot");
        tail.text("example.document-navigation.input.v1");
    }
    for (bit, role) in [(2, "Window"), (4, "Bookmark")] {
        if flags & bit != 0 {
            tail.text(&format!("{role}Policy"));
            tail.text(&format!("{role}Permission"));
            tail.text("example.retention.capability.v1");
            tail.text(&format!("Change{role}"));
            tail.text("example.retention.input.v1");
        }
    }
    bytes.extend(tail.into_bytes());
    bytes
}
#[test]
fn independent_window_and_bookmark_frames_keep_role_meaning_and_legacy_bytes() {
    let legacy = navigation::frame(1, false);
    for flags in [2, 3, 4, 5, 6, 7] {
        let mut parts = definition(Domain::ScopeEntityRecord, false).parts().clone();
        parts.navigation = (flags & 1 != 0).then(navigation::navigation);
        parts.window = (flags & 2 != 0).then(|| policy("Window"));
        parts.bookmarks = (flags & 4 != 0).then(|| policy("Bookmark"));
        let expected = ApplicationSchemaMember::ApplicationHistory {
            definition: Definition::from_untrusted_parts(parts),
        };
        let independent = frame(flags);
        assert_eq!(decode_member(&independent), expected);
        assert_eq!(encode_member(&expected), independent);
        for cut in legacy.len()..independent.len() {
            assert_eq!(
                try_decode_member(&independent[..cut]).unwrap_err().kind(),
                Kind::Truncated
            );
        }
    }
    assert_eq!(
        encode_member(&ApplicationSchemaMember::ApplicationHistory {
            definition: definition(Domain::ScopeEntityRecord, false)
        }),
        legacy
    );
    assert_ne!(
        frame(2),
        frame(4),
        "one valid role must not silently become the other"
    );
}
#[test]
fn retention_extensions_refuse_unknown_flags_and_invalid_policy_meaning() {
    let offset = navigation::frame(1, false).len();
    for flags in [0u16, 1, 8, 0x8002] {
        let mut bytes = frame(2);
        bytes[offset..offset + 2].copy_from_slice(&flags.to_be_bytes());
        assert_eq!(
            try_decode_member(&bytes).unwrap_err().kind(),
            Kind::UnsupportedRecordVariant
        );
    }
    let mut bytes = frame(6);
    let binding = bytes
        .windows(12)
        .position(|window| window == b"WindowPolicy")
        .unwrap();
    bytes[binding] = b' ';
    assert_eq!(
        try_decode_member(&bytes).unwrap_err().kind(),
        Kind::InvalidRecordShape
    );
}

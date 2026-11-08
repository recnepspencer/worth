//! Independent graph-domain wire grammar; legacy single-record bytes stay exact.
use super::*;
use worth_query_declaration::facade::application_history::ApplicationHistoryOwnershipRelation;

fn frame(owns: bool) -> Vec<u8> {
    let mut bytes = navigation::frame(1, false);
    bytes[2..4].copy_from_slice(&4_u16.to_be_bytes());
    bytes[4..6].copy_from_slice(&3_u16.to_be_bytes());
    let mut tail = BinaryOutput::with_capacity(0);
    tail.u16(1); // navigation present; no retention policies
    tail.text("NavigateDocument");
    tail.text("NavigationPermission");
    tail.text("example.document-navigation.capability.v1");
    tail.text("SelectDocumentSnapshot");
    tail.text("example.document-navigation.input.v1");
    tail.u32(u32::from(owns));
    if owns {
        tail.text("HasMember");
        tail.text("Document");
        tail.text("Member");
    }
    bytes.extend(tail.into_bytes());
    bytes
}

#[test]
fn owned_graph_domain_has_a_distinct_versioned_record_even_before_members_are_owned() {
    for owns in [false, true] {
        let mut parts = definition(Domain::OwnedModelGraph, false).parts().clone();
        parts.navigation = Some(navigation::navigation());
        if owns {
            parts.ownership.push(ApplicationHistoryOwnershipRelation {
                relation: "HasMember".into(),
                from: "Document".into(),
                to: "Member".into(),
            });
        }
        let definition = Definition::from_untrusted_parts(parts);
        definition.validate().unwrap();
        let member = ApplicationSchemaMember::ApplicationHistory { definition };
        let bytes = frame(owns);
        assert_eq!(encode_member(&member), bytes);
        assert_eq!(decode_member(&bytes), member);
        for old_version in 1..=3_u16 {
            let mut wrong_version = bytes.clone();
            wrong_version[2..4].copy_from_slice(&old_version.to_be_bytes());
            assert_eq!(
                try_decode_member(&wrong_version).unwrap_err().kind(),
                Kind::UnsupportedRecordVariant
            );
        }
    }
}

#[test]
fn single_record_navigation_cannot_smuggle_an_owned_graph_projection() {
    let mut parts = definition(Domain::ScopeEntityRecord, false).parts().clone();
    parts.navigation = Some(navigation::navigation());
    parts.ownership.push(ApplicationHistoryOwnershipRelation {
        relation: "HasMember".into(),
        from: "Document".into(),
        to: "Member".into(),
    });
    assert!(matches!(Definition::from_untrusted_parts(parts).validate(), Err(worth_query_declaration::facade::application_history::ApplicationHistoryDefinitionDenial::UnsupportedNavigationDomain)));
}

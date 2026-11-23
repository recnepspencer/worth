//! A peer-scope Parent read is archived under its own appended tag: its
//! Parent and governing relation survive exactly, and a truncated relation is
//! refused instead of decoding as an own-scope read.
use super::{decode_member, encode_member, try_decode_member};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationDecisionReadTarget as Target, ApplicationSchemaMember as Member,
};

fn member() -> Member {
    Member::OperationDecisionRead {
        operation: "Contact".into(),
        target: Target::SharedOwnerManagedIndex {
            parent: super::managed_index::definition()
                .parts()
                .source
                .binding
                .clone(),
            relation: "ModelFeature".into(),
            owner: "Model".into(),
            member: "Feature".into(),
        },
    }
}

#[test]
fn shared_owner_read_roundtrips_under_its_own_tag_and_refuses_a_truncated_relation() {
    let member = member();
    let bytes = encode_member(&member);
    assert_eq!(decode_member(&bytes), member);
    let Member::OperationDecisionRead { target, .. } = &member else {
        unreachable!()
    };
    let Target::SharedOwnerManagedIndex { parent, .. } = target else {
        unreachable!()
    };
    let own = encode_member(&Member::OperationDecisionRead {
        operation: "Contact".into(),
        target: Target::ManagedIndex {
            parent: parent.clone(),
        },
    });
    // Same operation prefix, then the target tag: own-scope 4, peer-scope 5.
    let tag = 6 + "Contact".len();
    assert_eq!((own[tag], own[tag + 1]), (0, 4));
    assert_eq!((bytes[tag], bytes[tag + 1]), (0, 5));
    assert_eq!(bytes[..tag], own[..tag]);
    assert_eq!(bytes[tag + 2..own.len()], own[tag + 2..]);
    for cut in [own.len(), bytes.len() - 1] {
        assert!(
            try_decode_member(&bytes[..cut]).is_err(),
            "a peer read without its whole relation is not an archive member"
        );
    }
}

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

/// A shared decision-read scope record whose single target is `target`,
/// written exactly as the scope codec writes one.
fn scope_wire(target: &Target) -> Vec<u8> {
    let mut output = crate::binary_output::BinaryOutput::with_capacity(1024);
    output.u16(55);
    output.u16(1);
    crate::record::sequence::write_sequence(&mut output, std::slice::from_ref(target), |output, target| {
        let bytes = encode_member(&Member::OperationDecisionRead {
            operation: String::new(),
            target: target.clone(),
        });
        output.raw_bytes(&bytes[6..])
    })
    .unwrap();
    crate::record::sequence::write_sequence(&mut output, &[] as &[String], |output, name| {
        output.text(name)
    })
    .unwrap();
    output.into_bytes()
}

#[test]
fn shared_decision_read_scope_carrying_a_peer_read_is_not_an_archive_member() {
    let Member::OperationDecisionRead { target, .. } = member() else {
        unreachable!()
    };
    let Target::SharedOwnerManagedIndex { parent, .. } = &target else {
        unreachable!()
    };
    // The same scope with the own-scope read decodes; the peer read cannot be
    // lent by a shared scope, so its record is refused before installation.
    let own = Target::ManagedIndex {
        parent: parent.clone(),
    };
    assert!(matches!(
        decode_member(&scope_wire(&own)),
        Member::DecisionReadScope { .. }
    ));
    assert_eq!(
        try_decode_member(&scope_wire(&target)).unwrap_err().kind(),
        crate::facade::WorthQueryPackageArchiveDenialKind::InvalidRecordShape
    );
}

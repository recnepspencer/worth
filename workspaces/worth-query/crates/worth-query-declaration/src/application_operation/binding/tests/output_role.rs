use super::{Changed, RenamedAccount, RetiredMapping};
use crate::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputPostureSet,
    ApplicationMutationOutputRoleCardinality, WorthQueryApplicationDeclaredOutputRole,
    WorthQueryApplicationDeclaredOutputRoleFamily, WorthQueryApplicationOutputCardinality,
    WorthQueryApplicationOutputRoleNameDenial, WorthQueryAtMostOneOutput,
    WorthQueryExactlyOneOutput,
};

const _: () = RenamedAccount::DECLARED;
const _: () = RetiredMapping::DECLARED;
const _: () = Changed::DECLARED;

#[test]
fn a_role_marker_derives_its_descriptor() {
    let exact = RenamedAccount::DESCRIPTOR;
    assert_eq!(exact.name(), "renamed-account");
    assert_eq!(exact.entity(), "Account");
    assert_eq!(exact.posture(), ApplicationMutationOutputPosture::Preserve);
    assert_eq!(
        exact.cardinality(),
        ApplicationMutationOutputRoleCardinality::ExactlyOne
    );

    let optional = RetiredMapping::DESCRIPTOR;
    assert_eq!(optional.entity(), "ExternalMapping");
    assert_eq!(optional.posture(), ApplicationMutationOutputPosture::Retire);
    assert_eq!(
        optional.cardinality(),
        ApplicationMutationOutputRoleCardinality::AtMostOne
    );
}

#[test]
fn a_cardinality_shapes_its_read() {
    assert_eq!(WorthQueryExactlyOneOutput::read(Some(7)), Some(7));
    assert_eq!(WorthQueryExactlyOneOutput::read::<u8>(None), None);
    assert_eq!(WorthQueryAtMostOneOutput::read(Some(7)), Some(Some(7)));
    assert_eq!(WorthQueryAtMostOneOutput::read::<u8>(None), Some(None));
}

#[test]
fn a_family_marker_derives_its_descriptor_and_names_its_members() {
    let family = Changed::DESCRIPTOR;
    assert_eq!(family.prefix(), "changed.");
    assert_eq!(family.entity(), "Account");
    assert_eq!(family.postures(), ApplicationMutationOutputPostureSet::ALL);
    assert_eq!(family.minimum(), 1);
    assert_eq!(Changed::member_name("a").as_deref(), Ok("changed.a"));
}

#[test]
fn a_family_member_needs_a_valid_suffix() {
    assert_eq!(
        Changed::member_name(""),
        Err(WorthQueryApplicationOutputRoleNameDenial::Empty)
    );
    assert_eq!(
        Changed::member_name("a "),
        Err(WorthQueryApplicationOutputRoleNameDenial::SurroundingWhitespace)
    );
    assert_eq!(
        Changed::member_name("a\u{7}b"),
        Err(WorthQueryApplicationOutputRoleNameDenial::ControlCharacter)
    );
    assert!(matches!(
        Changed::member_name(&"x".repeat(300)),
        Err(WorthQueryApplicationOutputRoleNameDenial::RepresentationTooLarge { .. })
    ));
}

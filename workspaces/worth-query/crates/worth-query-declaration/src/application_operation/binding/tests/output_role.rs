use super::{CHANGED, RENAMED_ACCOUNT, RETIRED_MAPPING};
use crate::application_operation::{
    ApplicationMutationOutputPosture, ApplicationMutationOutputPostureSet,
    ApplicationMutationOutputRoleCardinality, WorthQueryApplicationOutputRoleNameDenial,
    WorthQueryCreateOutput,
};

#[test]
fn a_token_declares_its_descriptor() {
    let exact = RENAMED_ACCOUNT.descriptor();
    assert_eq!(exact.name(), "renamed-account");
    assert_eq!(exact.entity(), "Account");
    assert_eq!(exact.posture(), ApplicationMutationOutputPosture::Preserve);
    assert_eq!(
        exact.cardinality(),
        ApplicationMutationOutputRoleCardinality::ExactlyOne
    );
    assert_eq!(RENAMED_ACCOUNT.name(), "renamed-account");

    let optional = RETIRED_MAPPING.descriptor();
    assert_eq!(optional.entity(), "ExternalMapping");
    assert_eq!(optional.posture(), ApplicationMutationOutputPosture::Retire);
    assert_eq!(
        optional.cardinality(),
        ApplicationMutationOutputRoleCardinality::AtMostOne
    );
}

#[test]
fn a_family_declares_its_descriptor_and_names_its_members() {
    let family = CHANGED.descriptor();
    assert_eq!(family.prefix(), "changed.");
    assert_eq!(family.entity(), "Account");
    assert_eq!(family.postures(), ApplicationMutationOutputPostureSet::ALL);
    assert_eq!(family.minimum(), 1);

    let member = CHANGED
        .member::<WorthQueryCreateOutput>("a")
        .expect("a non-empty suffix names a member");
    assert_eq!(member.name(), "changed.a");
}

#[test]
fn a_family_member_needs_a_valid_suffix() {
    let member = |suffix: &str| {
        CHANGED
            .member::<WorthQueryCreateOutput>(suffix)
            .map(|role| role.name().to_owned())
    };
    assert_eq!(
        member(""),
        Err(WorthQueryApplicationOutputRoleNameDenial::Empty)
    );
    assert_eq!(
        member("a "),
        Err(WorthQueryApplicationOutputRoleNameDenial::SurroundingWhitespace)
    );
    assert_eq!(
        member("a\u{7}b"),
        Err(WorthQueryApplicationOutputRoleNameDenial::ControlCharacter)
    );
    assert!(matches!(
        member(&"x".repeat(300)),
        Err(WorthQueryApplicationOutputRoleNameDenial::RepresentationTooLarge { .. })
    ));
}

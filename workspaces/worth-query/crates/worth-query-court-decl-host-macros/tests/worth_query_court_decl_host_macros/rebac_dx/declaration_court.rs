//! Malformed ReBAC rules refuse at declaration, in the words their author wrote.
use worth_query_decl::facade::application_schema::{
    ApplicationAuthorizationFieldConditions, ApplicationAuthorizationPathBuilder as Path,
    ApplicationAuthorizationRuleDenial as RuleDenial, ApplicationRoleMatrix,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
};
use worth_query_decl::facade::WorthQueryAbilities;

use super::schema::*;

#[test]
fn an_expansion_past_the_limit_names_the_authored_path_and_its_conditions() {
    let set = Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .when(AssignmentFacts::role().one_of([
            StaffRole::Reviewer,
            StaffRole::Manager,
            StaffRole::Auditor,
        ]))
        .when(AssignmentFacts::level().ne(0))
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .when(CaseFacts::author_id().ne_actor(UserFacts::user_id()))
        .when(CaseFacts::case_id().ne(9))
        .allow(Case::reference());
    match set.into_paths() {
        Err(RuleDenial::ExpansionLimit {
            path,
            expanding,
            lowered,
            limit,
        }) => {
            assert_eq!((lowered, limit), (24, 16));
            assert!(
                path.starts_with("User -UserAssignments-> Assignment [Assignment.Role one of"),
                "{path}"
            );
            assert!(path.ends_with("Case.CaseId != [UInt64(9)]]"), "{path}");
            assert_eq!(expanding.len(), 4, "{expanding:?}");
        }
        other => panic!("expected the expansion limit, got {other:?}"),
    }
}

#[test]
fn an_empty_set_and_a_non_queryable_field_refuse_by_condition() {
    let empty = Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .when(AssignmentFacts::role().one_of([]))
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .allow(Case::reference());
    assert!(matches!(
        empty.into_paths(),
        Err(RuleDenial::EmptyOneOf { condition }) if condition == "Assignment.Role one of"
    ));
    let unindexed = Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .when(CaseFacts::note().eq(1))
        .allow(Case::reference());
    assert!(matches!(
        unindexed.into_paths(),
        Err(RuleDenial::FieldNotQueryable { condition }) if condition == "Case.Note ="
    ));
}

#[test]
fn a_deny_condition_on_an_optional_field_is_refused_because_absence_would_fail_open() {
    let deny = Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .when(CaseFacts::escalated_by().eq(5))
        .deny(Case::reference());
    match deny.into_paths() {
        Err(RuleDenial::DenyConditionOnOptionalField { path, condition }) => {
            assert_eq!(condition, "Case.EscalatedBy = [UInt64(5)]");
            assert!(
                path.ends_with("<-CaseSite- Case [Case.EscalatedBy = [UInt64(5)]]"),
                "{path}"
            );
        }
        other => panic!("expected the optional-field deny refusal, got {other:?}"),
    }
    // The same condition is admissible on an allow path, where absence denies.
    let allow = Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .when(CaseFacts::escalated_by().eq(5))
        .allow(Case::reference());
    assert!(allow.into_paths().is_ok());
}

/// A second ability vocabulary where `Archive` is granted by no role.
#[derive(Clone, Copy, Debug, Eq, PartialEq, WorthQueryAbilities)]
#[worth_query(schema = RebacSchema, scoped_to = Case)]
pub enum BoardAbility {
    Comment,
    Archive,
}

impl ApplicationRoleMatrix<BoardAbility> for StaffRole {
    fn abilities(self) -> &'static [BoardAbility] {
        match self {
            Self::Manager => &[BoardAbility::Comment],
            Self::Reviewer | Self::Auditor => &[],
        }
    }
}

#[test]
fn a_role_less_ability_is_refused_by_name_instead_of_installed_for_nobody() {
    let refused = ApplicationSchemaDeclarationBuilder::<RebacSchema>::for_schema()
        .policy(CaseAccess::reference())
        .role_abilities(
            CaseAccess::reference(),
            |_: BoardAbility, roles: &[StaffRole]| {
                Path::from_principal(User::reference())
                    .forward(UserAssignments::reference())
                    .when(AssignmentFacts::role().one_of(roles.iter().copied()))
                    .forward(AssignmentSite::reference())
                    .reverse(CaseSite::reference())
                    .allow(Case::reference())
                    .into()
            },
        )
        .build()
        .err();
    assert_eq!(
        refused,
        Some(
            ApplicationSchemaDeclarationDenial::InvalidAuthorizationRule(
                RuleDenial::RoleLessAbility {
                    ability: "Archive".to_owned()
                }
            )
        )
    );
}

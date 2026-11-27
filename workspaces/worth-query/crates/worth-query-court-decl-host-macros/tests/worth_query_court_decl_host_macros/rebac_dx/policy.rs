//! The case board's authorization: one rule per ability, over the roles that
//! grant it, in the 9A-1 ReBAC authoring DX.
use worth_query_decl::facade::application_schema::{
    ApplicationAuthorizationFieldConditions, ApplicationAuthorizationPathBuilder as Path,
    ApplicationAuthorizationPathSet, ApplicationAuthorizationRule,
};

use super::schema::*;

type Rule = ApplicationAuthorizationRule<RebacSchema, User, Case>;

/// An assignment holding one of `roles` at the case's site, at `minimum_level` or above.
fn at_case_site(
    roles: &[StaffRole],
    minimum_level: u64,
) -> ApplicationAuthorizationPathSet<RebacSchema, User, Case> {
    Path::from_principal(User::reference())
        .forward(UserAssignments::reference())
        .when(AssignmentFacts::role().one_of(roles.iter().copied()))
        .when(AssignmentFacts::level().ge(minimum_level))
        .forward(AssignmentSite::reference())
        .reverse(CaseSite::reference())
        .allow(Case::reference())
}

pub fn case_rule(ability: CaseAbility, roles: &[StaffRole]) -> Rule {
    match ability {
        CaseAbility::ReadCase => at_case_site(roles, 0).into(),
        // A level-2 closer who is not the case's author (deny on the author edge).
        CaseAbility::Close => Rule::new()
            .path(at_case_site(roles, 2))
            .actor_is_not_target_of(CaseAuthor::reference()),
        // Anyone holding the role except the user the case records as author.
        CaseAbility::Annotate => Rule::new().path(
            Path::from_principal(User::reference())
                .forward(UserAssignments::reference())
                .when(AssignmentFacts::role().one_of(roles.iter().copied()))
                .forward(AssignmentSite::reference())
                .reverse(CaseSite::reference())
                .when(CaseFacts::author_id().ne_actor(UserFacts::user_id()))
                .allow(Case::reference()),
        ),
    }
}

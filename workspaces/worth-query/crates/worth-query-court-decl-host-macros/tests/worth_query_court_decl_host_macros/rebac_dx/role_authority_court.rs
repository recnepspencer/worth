//! W10A `role_authority`: the static half of "explain authority" lists what a
//! role grants from the schema's role matrix, one ability key per ability on
//! its scope entity, in the matrix's order.
use worth_query_decl::facade::application_schema::ApplicationAbilityVocabulary;
use worth_query_host::facade::eligibility::role_authority;

use super::schema::*;

fn granted(role: StaffRole) -> Vec<(&'static str, &'static str)> {
    role_authority::<RebacSchema, _, CaseAbility>(role)
        .iter()
        .map(|key| (key.ability(), key.scope_entity()))
        .collect()
}

#[test]
fn a_role_explains_exactly_the_abilities_its_matrix_row_grants() {
    let case = <CaseAbility as ApplicationAbilityVocabulary<RebacSchema>>::scope_name();
    let name = |ability: CaseAbility| {
        <CaseAbility as ApplicationAbilityVocabulary<RebacSchema>>::ability_name(ability)
    };
    assert_eq!(
        granted(StaffRole::Manager),
        [
            (name(CaseAbility::Annotate), case),
            (name(CaseAbility::Close), case),
            (name(CaseAbility::ReadCase), case),
        ]
    );
    assert_eq!(
        granted(StaffRole::Auditor),
        [(name(CaseAbility::ReadCase), case)]
    );
    assert!(!granted(StaffRole::Reviewer).contains(&(name(CaseAbility::Close), case)));
}

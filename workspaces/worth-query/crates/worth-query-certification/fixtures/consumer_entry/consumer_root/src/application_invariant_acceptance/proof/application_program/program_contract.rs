use worth_query_decl::facade::application_program::{
    ApplicationConnectionDeclaration, ApplicationProgramRuleDeclaration,
};

pub(super) fn assert_installed(
    connections: &[ApplicationConnectionDeclaration],
    rules: &[ApplicationProgramRuleDeclaration],
) {
    assert_eq!(connections.len(), 5);
    assert_eq!(rules.len(), 2);
    assert!(rules.iter().any(|rule| {
        rule.identity() == "PositiveParameterCount"
            && rule.local_owner() == Some("worth.query.certification.parameter-feature.v1")
    }));
    assert!(rules
        .iter()
        .any(|rule| rule.identity() == "PositivePlanarTurn" && rule.local_owner().is_none()));
}

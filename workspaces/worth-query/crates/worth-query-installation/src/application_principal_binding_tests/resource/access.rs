use super::*;
use crate::facade::{
    WorthQueryInstalledGraphAuthorizationRequirement as Authorization,
    WorthQueryInstalledGraphObligationSelectionBasis as Selection,
    WorthQueryResourceAccessPurpose as Purpose,
};

#[test]
fn resource_close_contract_keeps_its_own_ability_without_inspect_or_payload() {
    let schema = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let resource = schema
        .installed_resource::<Assessment<AssessmentResultBinding>>()
        .unwrap();
    let close = resource.access_contract(Purpose::Close);
    assert!(close.disclosure_rules().is_empty());
    assert!(Purpose::Close.facets().is_empty());
    let rows = close.graph_obligations();
    assert_eq!(rows.rows().len(), 2);
    let abilities = rows
        .rows()
        .iter()
        .find_map(|row| match row.authorization_requirement() {
            Some(Authorization::Abilities(requirements)) => Some(requirements),
            _ => None,
        })
        .unwrap();
    assert_eq!(abilities.len(), 1);
    assert_eq!(abilities[0].ability(), "CloseAssessment");
    assert!(rows
        .rows()
        .iter()
        .all(|row| matches!(row.selection_basis(), Selection::AuthenticatedAccessContext)));
    let inspect = resource.access_contract(Purpose::State);
    assert_eq!(inspect.graph_obligations().rows().len(), 1);
    assert_ne!(rows.identity(), inspect.graph_obligations().identity());
    let another_lookup = schema
        .installed_resource::<Assessment<AssessmentResultBinding>>()
        .unwrap();
    assert!(another_lookup.owns_access_contract(close));
    let foreign = installed_index()
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let foreign = foreign
        .installed_resource::<Assessment<AssessmentResultBinding>>()
        .unwrap();
    assert!(!foreign.owns_access_contract(close));
}

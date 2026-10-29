pub use bank_domain::schema::{
    RequestEstateEmergencyAccessCapability, RequestEstateEmergencyAccessOperation,
    ViewEstateAdministrationCapability, ViewRestrictedEstateOperation,
};
pub use worth_query_host::facade::domain::{
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationCapabilityIdentity,
    WorthQueryInstalledGraphAuthorizationRequirement,
    WorthQueryInstalledGraphObligationEffectPosture, WorthQueryInstalledGraphObligationInspection,
    WorthQueryInstalledGraphObligationResourcePosture,
    WorthQueryInstalledGraphObligationSelectionBasis,
    WorthQueryInstalledGraphObligationTerminalRequirement,
};

pub fn assert_capability_only_authority(
    obligations: WorthQueryInstalledGraphObligationInspection<'_>,
    capability_identity: &WorthQueryInstalledApplicationCapabilityIdentity,
) {
    let [authorization] = obligations.rows() else {
        panic!("capability graph authority must contain exactly its authorization observation")
    };

    assert_eq!(
        authorization.effect_posture(),
        WorthQueryInstalledGraphObligationEffectPosture::Policy
    );
    assert!(matches!(
        authorization.selection_basis(),
        WorthQueryInstalledGraphObligationSelectionBasis::AuthenticatedAccessContext
    ));
    assert_eq!(
        authorization.terminal_requirement(),
        WorthQueryInstalledGraphObligationTerminalRequirement::AuthorizationDecisionFact
    );
    assert_eq!(
        authorization.resource_posture(),
        &WorthQueryInstalledGraphObligationResourcePosture::ApplicationQuery {
            maximum_traversal_depth: 0,
            maximum_result_count: 0,
            maximum_authorization_facts: 2,
        }
    );

    let Some(WorthQueryInstalledGraphAuthorizationRequirement::Capabilities(requirements)) =
        authorization.authorization_requirement()
    else {
        panic!("capability graph authority must retain the installed capability requirement")
    };
    let [requirement] = requirements else {
        panic!("capability graph authority must retain exactly one capability requirement")
    };
    assert_eq!(requirement.identity(), capability_identity);
}

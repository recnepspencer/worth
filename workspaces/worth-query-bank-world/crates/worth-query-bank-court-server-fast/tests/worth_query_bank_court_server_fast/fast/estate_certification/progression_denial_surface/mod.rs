pub(crate) use worth_query_bank_worlds::facade::estate_certification::progression_denial_surface::*;

#[test]
fn nested_progression_denial_payloads_are_nameable_by_external_consumers() {
    fn require_public_type<T>() {}

    require_public_type::<BankEstateProgressionDenial>();
    require_public_type::<BankEstateLifecycleProjectionDenial>();
    require_public_type::<BankCapabilityDelegationProjectionDenial>();
    require_public_type::<BankCapabilityRevocationProjectionDenial>();
}

pub use bank_domain::{estate::*, schema::*};
pub use worth_query_host::facade::{
    declaration::{
        application_capability::{
            ApplicationCapabilityFieldDimension, ApplicationCapabilityRelationDimension,
            ApplicationCapabilityRequest,
        },
        application_schema::ApplicationScalarValueBinding,
    },
    domain::WorthQueryInstallationRuntimeIdentity,
};

pub use super::installed_bank;

#[macro_export]
macro_rules! worlds_domain_estate_capability_installation_transition_dimensions_rs_assert_transition_target {
    ($bank:expr, $capability:ident, $operation:ident, $action:expr, $purpose:expr) => {{
        let capability = $bank
            .capability($capability::reference(), $operation::reference())
            .unwrap();
        let contract = capability.contract();
        let target = contract.target();
        assert_eq!(
            target.action().value(),
            &EstateCapabilityOperationBinding::encode(&$action).unwrap()
        );
        assert_eq!(
            target.purpose().value(),
            &EstateCapabilityPurposeBinding::encode(&$purpose).unwrap()
        );
        assert!(matches!(
            target.relation(),
            ApplicationCapabilityRelationDimension::NotApplicable
        ));
        assert!(matches!(
            target.field(),
            ApplicationCapabilityFieldDimension::NotApplicable
        ));
        assert!(matches!(
            contract.constraints().magnitude(),
            ApplicationCapabilityFieldDimension::NotApplicable
        ));
    }};
}
pub use crate::worlds_domain_estate_capability_installation_transition_dimensions_rs_assert_transition_target as assert_transition_target;

pub use super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRetainedCapabilityAuthorization,
};
pub use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
pub mod denial;
pub mod observation;
pub use observation::WorthQueryCapabilityRevalidationObservation;

pub struct WorthQueryCapabilityRefresh<'refresh> {
    pub session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    pub product: &'refresh crate::basis::WorthQueryProductObservationLease,
    pub installed: &'refresh super::capability_registry::WorthQueryInstalledCapabilityPlan,
    pub sample: super::WorthQueryRuntimeTimeSample,
}

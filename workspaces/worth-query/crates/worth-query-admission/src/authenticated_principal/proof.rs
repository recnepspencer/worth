use std::marker::PhantomData;
use std::time::{Instant, SystemTime};

use worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;

use super::{
    WorthQueryAuthenticationAdapterIdentity, WorthQueryAuthenticationAudience,
    WorthQueryAuthenticationMethod, WorthQueryPrincipalAttribute,
};

/// Sealed proof that one admitted adapter authenticated an external identity.
///
/// This proof identifies an external principal only. It is not an application
/// principal, authorization decision, touched-graph authority, or execution
/// authority.
///
/// Direct field construction is not a consumer authority path:
///
/// ```compile_fail
/// use worth_query_admission::facade::authenticated_principal::
///     WorthQueryAuthenticatedExternalPrincipal;
///
/// let _ = WorthQueryAuthenticatedExternalPrincipal::<()> {
///     identity: todo!(),
///     audience: todo!(),
///     method: todo!(),
///     validated_at: todo!(),
///     expires_at: todo!(),
///     valid_until: todo!(),
///     adapter_identity: String::new(),
///     binding_identity: todo!(),
///     attributes: Vec::new(),
///     _schema: std::marker::PhantomData,
/// };
/// ```
///
/// Serialized data cannot mint the proof:
///
/// ```compile_fail
/// use worth_query_admission::facade::authenticated_principal::
///     WorthQueryAuthenticatedExternalPrincipal;
///
/// let _: WorthQueryAuthenticatedExternalPrincipal<()> =
///     serde_json::from_str("{}").unwrap();
/// ```
pub struct WorthQueryAuthenticatedExternalPrincipal<Schema> {
    identity: WorthQueryExternalPrincipalIdentity,
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
    validated_at: SystemTime,
    expires_at: SystemTime,
    valid_until: Instant,
    clock: super::WorthQueryAdmissionClock,
    adapter_identity: WorthQueryAuthenticationAdapterIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    attributes: Vec<WorthQueryPrincipalAttribute>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryAuthenticatedExternalPrincipal<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn mint(
        identity: WorthQueryExternalPrincipalIdentity,
        audience: WorthQueryAuthenticationAudience,
        method: WorthQueryAuthenticationMethod,
        validated_at: SystemTime,
        expires_at: SystemTime,
        valid_until: Instant,
        clock: super::WorthQueryAdmissionClock,
        adapter_identity: WorthQueryAuthenticationAdapterIdentity,
        binding_identity: ApplicationSchemaBindingIdentity,
        attributes: Vec<WorthQueryPrincipalAttribute>,
    ) -> Self {
        Self {
            identity,
            audience,
            method,
            validated_at,
            expires_at,
            valid_until,
            clock,
            adapter_identity,
            binding_identity,
            attributes,
            _schema: PhantomData,
        }
    }

    pub fn identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.identity
    }

    pub fn audience(&self) -> &WorthQueryAuthenticationAudience {
        &self.audience
    }

    pub fn method(&self) -> &WorthQueryAuthenticationMethod {
        &self.method
    }

    pub fn validated_at(&self) -> SystemTime {
        self.validated_at
    }

    pub fn expires_at(&self) -> SystemTime {
        self.expires_at
    }

    pub fn valid_until(&self) -> Instant {
        self.valid_until
    }

    pub const fn adapter_identity(&self) -> &WorthQueryAuthenticationAdapterIdentity {
        &self.adapter_identity
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn attributes(&self) -> &[WorthQueryPrincipalAttribute] {
        &self.attributes
    }

    pub fn clock(&self) -> &super::WorthQueryAdmissionClock {
        &self.clock
    }

    pub fn is_expired(&self) -> bool {
        self.clock.now() >= self.valid_until
    }
}

impl<Schema> std::fmt::Debug for WorthQueryAuthenticatedExternalPrincipal<Schema> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAuthenticatedExternalPrincipal")
            .field("identity", &self.identity)
            .field("audience", &self.audience)
            .field("method", &self.method)
            .field("validated_at", &self.validated_at)
            .field("expires_at", &self.expires_at)
            .field("adapter_identity", &self.adapter_identity)
            .field("binding_identity", &self.binding_identity)
            .field("attribute_count", &self.attributes.len())
            .finish_non_exhaustive()
    }
}

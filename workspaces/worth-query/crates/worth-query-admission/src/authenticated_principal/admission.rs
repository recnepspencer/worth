use std::marker::PhantomData;
use std::time::{Instant, SystemTime};

use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::{
    adapter_identity::adapter_identity, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationAdapterIdentity,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationDenial,
    WorthQueryAuthenticationDenialKind, WorthQueryAuthenticationMethod, WorthQueryRequestScope,
};

const MAX_CONFIGURATION_IDENTITY_BYTES: usize = 512;

pub struct WorthQueryAuthenticationAdapterAdmission {
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
}

impl WorthQueryAuthenticationAdapterAdmission {
    pub fn new(
        audience: WorthQueryAuthenticationAudience,
        method: WorthQueryAuthenticationMethod,
    ) -> Self {
        Self { audience, method }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAuthenticationAdapterAdmissionDenial {
    InvalidConfigurationIdentity,
    CanonicalEntryBudgetExceeded,
    CanonicalEncodedByteBudgetExceeded,
    CanonicalDigestSlotRejected,
}

impl std::fmt::Display for WorthQueryAuthenticationAdapterAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "authentication adapter admission denied: {self:?}"
        )
    }
}

impl std::error::Error for WorthQueryAuthenticationAdapterAdmissionDenial {}

pub fn admit_authentication_adapter<Schema, Adapter>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    admission: WorthQueryAuthenticationAdapterAdmission,
    adapter: Adapter,
) -> Result<
    WorthQueryAdmittedAuthenticationAdapter<Schema, Adapter>,
    WorthQueryAuthenticationAdapterAdmissionDenial,
>
where
    Schema: ApplicationSchema,
    Adapter: WorthQueryAuthenticationAdapter,
{
    let configuration_identity = adapter.configuration_identity();
    if configuration_identity.is_empty()
        || configuration_identity.trim() != configuration_identity
        || configuration_identity.len() > MAX_CONFIGURATION_IDENTITY_BYTES
        || configuration_identity.chars().any(char::is_control)
    {
        return Err(WorthQueryAuthenticationAdapterAdmissionDenial::InvalidConfigurationIdentity);
    }
    let binding_identity = schema.binding_identity();
    let adapter_identity = adapter_identity(
        &binding_identity,
        configuration_identity,
        &admission.audience,
        &admission.method,
    )
    .map_err(|denial| match denial {
        worth_foundational::facade::CanonicalDigestDerivationDenial::EntryLimitExceeded {
            ..
        } => WorthQueryAuthenticationAdapterAdmissionDenial::CanonicalEntryBudgetExceeded,
        worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded {
            ..
        } => WorthQueryAuthenticationAdapterAdmissionDenial::CanonicalEncodedByteBudgetExceeded,
        _ => WorthQueryAuthenticationAdapterAdmissionDenial::CanonicalDigestSlotRejected,
    })?;
    Ok(WorthQueryAdmittedAuthenticationAdapter {
        adapter,
        audience: admission.audience,
        method: admission.method,
        adapter_identity,
        binding_identity,
        _schema: PhantomData,
    })
}

pub struct WorthQueryAdmittedAuthenticationAdapter<Schema, Adapter> {
    adapter: Adapter,
    audience: WorthQueryAuthenticationAudience,
    method: WorthQueryAuthenticationMethod,
    adapter_identity: WorthQueryAuthenticationAdapterIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema, Adapter> WorthQueryAdmittedAuthenticationAdapter<Schema, Adapter>
where
    Adapter: WorthQueryAuthenticationAdapter,
{
    /// Returns the admitted protocol adapter without exposing proof-minting authority.
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub async fn authenticate(
        &self,
        credential: Adapter::Credential,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, WorthQueryAuthenticationDenial>
    {
        if let Some(interruption) = scope.interruption() {
            return Err(WorthQueryAuthenticationDenial::interrupted(interruption));
        }
        let candidate = self
            .adapter
            .validate(credential, scope)
            .await
            .map_err(|failure| {
                WorthQueryAuthenticationDenial::new(
                    WorthQueryAuthenticationDenialKind::AdapterFailed(failure.kind()),
                )
            })?;
        if let Some(interruption) = scope.interruption() {
            return Err(WorthQueryAuthenticationDenial::interrupted(interruption));
        }
        self.admit_candidate(candidate)
    }

    fn admit_candidate(
        &self,
        candidate: super::WorthQueryValidatedExternalPrincipal,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<Schema>, WorthQueryAuthenticationDenial>
    {
        if candidate.audience() != &self.audience {
            return Err(WorthQueryAuthenticationDenial::new(
                WorthQueryAuthenticationDenialKind::AudienceMismatch,
            ));
        }
        if candidate.method() != &self.method {
            return Err(WorthQueryAuthenticationDenial::new(
                WorthQueryAuthenticationDenialKind::MethodMismatch,
            ));
        }
        let wall_now = SystemTime::now();
        let monotonic_now = Instant::now();
        if candidate.validated_at() > wall_now {
            return Err(WorthQueryAuthenticationDenial::new(
                WorthQueryAuthenticationDenialKind::ValidationTimeInFuture,
            ));
        }
        let remaining = candidate
            .expires_at()
            .duration_since(wall_now)
            .map_err(|_| {
                WorthQueryAuthenticationDenial::new(WorthQueryAuthenticationDenialKind::Expired)
            })?;
        let valid_until = monotonic_now.checked_add(remaining).ok_or_else(|| {
            WorthQueryAuthenticationDenial::new(WorthQueryAuthenticationDenialKind::Expired)
        })?;
        let (identity, audience, method, validated_at, expires_at, attributes) =
            candidate.into_parts();
        Ok(WorthQueryAuthenticatedExternalPrincipal::mint(
            identity,
            audience,
            method,
            validated_at,
            expires_at,
            valid_until,
            self.adapter_identity.clone(),
            self.binding_identity.clone(),
            attributes,
        ))
    }

    pub const fn adapter_identity(&self) -> &WorthQueryAuthenticationAdapterIdentity {
        &self.adapter_identity
    }
}

impl<Schema, Adapter> std::fmt::Debug for WorthQueryAdmittedAuthenticationAdapter<Schema, Adapter> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAdmittedAuthenticationAdapter")
            .field("audience", &self.audience)
            .field("method", &self.method)
            .field("adapter_identity", &self.adapter_identity)
            .field("binding_identity", &self.binding_identity)
            .finish_non_exhaustive()
    }
}

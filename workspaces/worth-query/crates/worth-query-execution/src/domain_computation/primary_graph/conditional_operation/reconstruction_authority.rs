use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationIdentityScalarValueBinding,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, EqualityPredicate,
    WorthQueryInstalledPrincipalBinding, WritePosture,
};

use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrincipalResolutionMode,
};

pub(super) struct WorthQueryFreshTemporalOperationAccess<
    Schema,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub(super) principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    pub(super) scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    pub(super) request: WorthQueryRequestScope,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryTemporalPrincipalFailureKind {
    SourceUnavailable,
    AdmissionRejected,
    SourcePanicked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryTemporalPrincipalFailure {
    kind: WorthQueryTemporalPrincipalFailureKind,
    detail: String,
}

impl WorthQueryTemporalPrincipalFailure {
    pub fn new(kind: WorthQueryTemporalPrincipalFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> WorthQueryTemporalPrincipalFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Fresh sealed authentication and request bounds for one temporal contact.
///
/// The source cannot mint `external`; it can only hand Query a proof created
/// by an admitted authentication adapter. Query still resolves that proof
/// through the exact installed principal binding before reading or mutating.
pub struct WorthQueryTemporalPrincipalAdmission<Schema> {
    external: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: WorthQueryRequestScope,
}

impl<Schema> WorthQueryTemporalPrincipalAdmission<Schema> {
    pub fn new(
        external: WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: WorthQueryRequestScope,
    ) -> Self {
        Self { external, request }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryAuthenticatedExternalPrincipal<Schema>,
        WorthQueryRequestScope,
    ) {
        (self.external, self.request)
    }
}

/// Host port for obtaining fresh ordinary-principal authority.
///
/// It supplies authentication evidence, not operation permission. Every
/// returned proof is freshly resolved and authorized by Query.
pub trait WorthQueryTemporalPrincipalSource<Schema>: Send + Sync + 'static {
    const SEMANTIC_IDENTITY: &'static str;

    fn admit(
        &self,
    ) -> Result<WorthQueryTemporalPrincipalAdmission<Schema>, WorthQueryTemporalPrincipalFailure>;
}

/// Move-only authority inputs for the installed reconstruction query.
pub struct WorthQueryTemporalReconstructionAccess<
    Schema,
    Binding,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
    Scope,
    ScopeAspect,
    ScopeField,
    ScopeValue,
    ScopeWrite,
    ScopeUnit,
    PrincipalSource,
    QueryAuthorization = super::WorthQueryPublicTemporalQueryAuthorization,
> {
    pub(super) principal_binding: WorthQueryInstalledPrincipalBinding<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
    pub(super) principal_source: Arc<PrincipalSource>,
    pub(super) scope_field: ApplicationFieldRef<
        Schema,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        EqualityPredicate,
        ScopeUnit,
    >,
    pub(super) scope_value: ScopeValue,
    pub(super) query_authorization: QueryAuthorization,
}

impl<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
    >
    WorthQueryTemporalReconstructionAccess<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
    >
where
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    PrincipalIdentity: 'static,
    ScopeField: DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding: ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
{
    pub fn new(
        principal_binding: WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        principal_source: PrincipalSource,
        scope_field: ApplicationFieldRef<
            Schema,
            Scope,
            ScopeAspect,
            ScopeField,
            ScopeValue,
            ScopeWrite,
            EqualityPredicate,
            ScopeUnit,
        >,
        scope_value: ScopeValue,
    ) -> Result<Self, &'static str> {
        let identity = PrincipalSource::SEMANTIC_IDENTITY;
        if identity.is_empty()
            || identity.trim() != identity
            || identity.chars().any(char::is_whitespace)
        {
            return Err("invalid-temporal-principal-source-identity");
        }
        Ok(Self {
            principal_binding,
            principal_source: Arc::new(principal_source),
            scope_field,
            scope_value,
            query_authorization: super::WorthQueryPublicTemporalQueryAuthorization,
        })
    }
}

impl<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
    >
    WorthQueryTemporalReconstructionAccess<
        Schema,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
        Scope,
        ScopeAspect,
        ScopeField,
        ScopeValue,
        ScopeWrite,
        ScopeUnit,
        PrincipalSource,
        QueryAuthorization,
    >
where
    PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
    PrincipalIdentity: 'static,
    ScopeField: DeclaredApplicationFieldValue<Value = ScopeValue>,
    ScopeField::Binding: ApplicationScalarValueBinding<Value = ScopeValue>,
    ScopeWrite: WritePosture,
    ScopeUnit: ApplicationFieldUnit,
    PrincipalSource: WorthQueryTemporalPrincipalSource<Schema>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn with_query_authorization(
        principal_binding: WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        principal_source: PrincipalSource,
        scope_field: ApplicationFieldRef<
            Schema,
            Scope,
            ScopeAspect,
            ScopeField,
            ScopeValue,
            ScopeWrite,
            EqualityPredicate,
            ScopeUnit,
        >,
        scope_value: ScopeValue,
        query_authorization: QueryAuthorization,
    ) -> Result<Self, &'static str> {
        validate_principal_source_identity(PrincipalSource::SEMANTIC_IDENTITY)?;
        Ok(Self {
            principal_binding,
            principal_source: Arc::new(principal_source),
            scope_field,
            scope_value,
            query_authorization,
        })
    }

    pub fn principal_source_identity(&self) -> &'static str {
        PrincipalSource::SEMANTIC_IDENTITY
    }

    pub(super) fn fresh_admission(
        &self,
    ) -> Result<WorthQueryTemporalPrincipalAdmission<Schema>, WorthQueryTemporalPrincipalFailure>
    {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.principal_source.admit()
        })) {
            Ok(result) => result,
            Err(_) => Err(WorthQueryTemporalPrincipalFailure::new(
                WorthQueryTemporalPrincipalFailureKind::SourcePanicked,
                "temporal principal source panicked",
            )),
        }
    }

    pub(super) fn resolve_fresh_operation_access(
        &self,
        product: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryFreshTemporalOperationAccess<Schema, Principal, PrincipalIdentity, Scope>,
        super::application_operation_reentry::WorthQueryTemporalReentryDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        ScopeValue: Clone,
    {
        let (external, request) = self
            .fresh_admission()
            .map_err(|failure| format!("{:?}: {}", failure.kind(), failure.detail()))?
            .into_parts();
        let principal = product
            .resolve_authenticated_principal(
                &self.principal_binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(super::application_operation_reentry::WorthQueryTemporalReentryDenial::from_principal)?;
        let scope = product
            .resolve_entity(
                self.scope_field,
                self.scope_value.clone(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(
                super::application_operation_reentry::WorthQueryTemporalReentryDenial::from_entity,
            )?;
        Ok(WorthQueryFreshTemporalOperationAccess {
            principal,
            scope,
            request,
        })
    }
}

fn validate_principal_source_identity(identity: &str) -> Result<(), &'static str> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err("invalid-temporal-principal-source-identity")
    } else {
        Ok(())
    }
}

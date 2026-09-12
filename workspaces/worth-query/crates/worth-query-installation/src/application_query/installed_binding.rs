use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryScopeBinding, ApplicationQueryScopeContract,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};

use crate::{
    application_principal_binding::WorthQueryInstalledPrincipalBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
};

use super::{
    binding::WorthQueryInstalledApplicationQueryLimits,
    WorthQueryApplicationQueryInstallationDenial,
    WorthQueryApplicationQueryInstallationDenialKind as DenialKind,
    WorthQueryInstalledApplicationQuery,
};

pub type WorthQueryInstalledBoundQuery<Schema, Binding> = WorthQueryInstalledApplicationQuery<
    Schema,
    <Binding as ApplicationQueryBinding<Schema>>::Query,
    <<Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    <<Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
>;

pub type WorthQueryInstalledBoundPrincipal<Schema, Binding> = WorthQueryInstalledPrincipalBinding<
    Schema,
    <Binding as ApplicationQueryBinding<Schema>>::PrincipalBinding,
    <Binding as ApplicationQueryBinding<Schema>>::Mapping,
    <Binding as ApplicationQueryBinding<Schema>>::Principal,
    <Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
    <Binding as ApplicationQueryBinding<Schema>>::PrincipalIdentityBinding,
>;

/// Installed authority for one exact declaration binding and schema generation.
pub struct WorthQueryInstalledApplicationQueryBinding<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    query: WorthQueryInstalledBoundQuery<Schema, Binding>,
    principal_binding: WorthQueryInstalledBoundPrincipal<Schema, Binding>,
    scope: ApplicationQueryScopeContract,
    limits: WorthQueryInstalledApplicationQueryLimits,
}

impl<Schema, Binding> WorthQueryInstalledApplicationQueryBinding<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationQueryBinding<Schema>,
{
    pub fn query(&self) -> &WorthQueryInstalledBoundQuery<Schema, Binding> {
        &self.query
    }

    pub fn into_query(self) -> WorthQueryInstalledBoundQuery<Schema, Binding> {
        self.query
    }

    pub fn identity(&self) -> &str {
        Binding::IDENTITY
    }

    pub fn principal_binding(&self) -> &WorthQueryInstalledBoundPrincipal<Schema, Binding> {
        &self.principal_binding
    }

    pub fn scope_contract(&self) -> &ApplicationQueryScopeContract {
        &self.scope
    }

    pub const fn limits(&self) -> WorthQueryInstalledApplicationQueryLimits {
        self.limits
    }
}

impl<Schema> WorthQueryInstalledApplicationSchema<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn installed_query_binding<Binding>(
        &self,
    ) -> Result<
        WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        WorthQueryApplicationQueryInstallationDenial,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        let requested = Binding::descriptor();
        let installed = self
            .query_catalog
            .get_binding(Binding::IDENTITY)
            .ok_or_else(|| denial(DenialKind::BindingQueryNotInstalled, Binding::IDENTITY))?;
        if installed.descriptor() != &requested {
            return Err(denial(DenialKind::BindingMeaningChanged, Binding::IDENTITY));
        }
        let principal_binding = self
            .principal_binding(Binding::principal_binding())
            .map_err(|_| {
                denial(
                    DenialKind::BindingPrincipalMeaningChanged,
                    Binding::IDENTITY,
                )
            })?;
        Ok(WorthQueryInstalledApplicationQueryBinding {
            query: WorthQueryInstalledApplicationQuery::from_compiled(installed.query()),
            principal_binding,
            scope: installed.scope().clone(),
            limits: installed.limits(),
        })
    }
}

fn denial(kind: DenialKind, subject: &str) -> WorthQueryApplicationQueryInstallationDenial {
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}

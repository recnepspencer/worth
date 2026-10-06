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

pub enum WorthQueryInstalledQueryBindingAdmissionStop<E> {
    Installation(WorthQueryApplicationQueryInstallationDenial),
    Admission(E),
    AccountingOverflow,
}

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
    pub fn installed_query_identity_by_name(
        &self,
        identifier: &str,
    ) -> Option<&super::WorthQueryInstalledApplicationQueryIdentity> {
        self.query_catalog.query_identity_by_name(identifier)
    }

    pub fn installed_query_binding<Binding>(
        &self,
    ) -> Result<
        WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        WorthQueryApplicationQueryInstallationDenial,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        match self.installed_query_binding_core::<Binding, std::convert::Infallible>(None) {
            Ok(binding) => Ok(binding),
            Err(WorthQueryInstalledQueryBindingAdmissionStop::Installation(denial)) => Err(denial),
            Err(WorthQueryInstalledQueryBindingAdmissionStop::Admission(never)) => match never {},
            Err(WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow) => {
                unreachable!("legacy lookup performs no accounting")
            }
        }
    }

    pub fn installed_query_binding_admitted<Binding, E>(
        &self,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), E>,
    ) -> Result<
        WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        WorthQueryInstalledQueryBindingAdmissionStop<E>,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        self.installed_query_binding_core::<Binding, E>(Some(prepare))
    }

    fn installed_query_binding_core<Binding, E>(
        &self,
        mut prepare: Option<&mut dyn FnMut(u64, u64) -> Result<(), E>>,
    ) -> Result<
        WorthQueryInstalledApplicationQueryBinding<Schema, Binding>,
        WorthQueryInstalledQueryBindingAdmissionStop<E>,
    >
    where
        Binding: ApplicationQueryBinding<Schema>,
    {
        if let Some(admit) = prepare.as_mut() {
            let subject = u64::try_from(Binding::IDENTITY.len())
                .map_err(|_| WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow)?;
            admit(
                subject
                    .checked_add(1)
                    .ok_or(WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow)?,
                subject,
            )
            .map_err(WorthQueryInstalledQueryBindingAdmissionStop::Admission)?;
        }
        // Binding::descriptor is application-authored installation code. The
        // framework's lookup, comparison and owned reissue are admitted below.
        let requested = Binding::descriptor();
        let installed = if let Some(admit) = prepare.as_mut() {
            self.query_catalog
                .get_binding_admitted(Binding::IDENTITY, admit)
                .map_err(|stop| match stop {
                    super::binding::AdmittedQueryCatalogStop::Admission(stop) => {
                        WorthQueryInstalledQueryBindingAdmissionStop::Admission(stop)
                    }
                    super::binding::AdmittedQueryCatalogStop::AccountingOverflow => {
                        WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow
                    }
                })?
        } else {
            self.query_catalog.get_binding(Binding::IDENTITY)
        };
        let installed = installed.ok_or_else(|| {
            WorthQueryInstalledQueryBindingAdmissionStop::Installation(denial(
                DenialKind::BindingQueryNotInstalled,
                Binding::IDENTITY,
            ))
        })?;
        if let Some(admit) = prepare.as_mut() {
            // The bound reads thirty selected widths on each descriptor.
            admit(60, 0).map_err(WorthQueryInstalledQueryBindingAdmissionStop::Admission)?;
            let work = requested
                .comparison_work_bound(installed.descriptor())
                .and_then(|work| u64::try_from(work).ok())
                .ok_or(WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow)?;
            admit(work, 0).map_err(WorthQueryInstalledQueryBindingAdmissionStop::Admission)?;
        }
        if installed.descriptor() != &requested {
            return Err(WorthQueryInstalledQueryBindingAdmissionStop::Installation(
                denial(DenialKind::BindingMeaningChanged, Binding::IDENTITY),
            ));
        }
        let principal_binding =
            if let Some(admit) = prepare.as_mut() {
                self.principal_binding_admitted(Binding::principal_binding(), admit)
                    .map_err(|stop| {
                        match stop {
                    crate::application_schema::AdmittedPrincipalBindingStop::Installation(_) =>
                        WorthQueryInstalledQueryBindingAdmissionStop::Installation(denial(
                            DenialKind::BindingPrincipalMeaningChanged, Binding::IDENTITY)),
                    crate::application_schema::AdmittedPrincipalBindingStop::Admission(stop) =>
                        WorthQueryInstalledQueryBindingAdmissionStop::Admission(stop),
                    crate::application_schema::AdmittedPrincipalBindingStop::AccountingOverflow =>
                        WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow,
                }
                    })?
            } else {
                self.principal_binding(Binding::principal_binding())
                    .map_err(|_| {
                        WorthQueryInstalledQueryBindingAdmissionStop::Installation(denial(
                            DenialKind::BindingPrincipalMeaningChanged,
                            Binding::IDENTITY,
                        ))
                    })?
            };
        if let Some(admit) = prepare.as_mut() {
            let field = installed.scope().field();
            let bytes = field
                .locus()
                .entity()
                .len()
                .checked_add(field.locus().aspect().len())
                .and_then(|n| n.checked_add(field.locus().field().len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow)?;
            admit(
                bytes
                    .checked_add(4)
                    .ok_or(WorthQueryInstalledQueryBindingAdmissionStop::AccountingOverflow)?,
                bytes,
            )
            .map_err(WorthQueryInstalledQueryBindingAdmissionStop::Admission)?;
        }
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

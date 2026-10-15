//! Source-program dispatch into the existing typed capability admission owner.
mod effect;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrincipalResolutionMode,
    WorthQuerySelectedProductOperation,
};
pub(super) use effect::{AdoptionEffect, AdoptionEffectOutcome, AdoptionEffectRefusal};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRef, application_program::*,
    application_schema::ApplicationOperationRef,
};
use worth_query_installation::facade::ApplicationSchema;

/// Why the selected source program refused inspection or adoption.
/// Authorization refusals perform no adoption effects and preserve carried custody.
#[derive(Debug)]
pub enum WorthQueryProgramAdoptionAuthorizationDenial {
    ForeignBranch {
        expected: crate::basis::WorthQueryProductBranch,
        requested: crate::basis::WorthQueryProductBranch,
    },
    ProductSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    CapabilityNotDeclared,
    InstalledMeaningChanged,
    PrincipalInstallation(
        worth_query_installation::facade::WorthQueryPrincipalBindingInstallationDenial,
    ),
    PrincipalResolution(
        crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial,
    ),
    OperationInstallation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    CapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
}

pub(in crate::domain_computation::primary_graph) type ProgramAdoptionAdmission<Schema> =
    for<'effect> fn(
        &WorthQuerySelectedProductOperation<'_, Schema>,
        &WorthQueryAdvancementPhase<'_>,
        &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        &WorthQueryRequestScope,
        &ApplicationProgramRevision,
        AdoptionEffect<'effect>,
    ) -> Result<AdoptionEffectOutcome, AdoptionEffectRefusal<'effect>>;

pub(in crate::domain_computation::primary_graph) fn admit_program<
    'effect,
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &WorthQueryRequestScope,
    target: &ApplicationProgramRevision,
    effect: AdoptionEffect<'effect>,
) -> Result<AdoptionEffectOutcome, AdoptionEffectRefusal<'effect>> {
    let source = match selected.inspect_selected_program() {
        Ok(source) => *source.revision(),
        Err(cause) => {
            return Err(AdoptionEffectRefusal {
                cause: WorthQueryProgramAdoptionAuthorizationDenial::Inspection(cause),
                effect,
            })
        }
    };
    Program::adoption_authorization(AdmissionVisitor {
        selected,
        phase,
        principal,
        scope,
        source,
        target,
        effect,
    })
}

struct AdmissionVisitor<'a, 'runtime, 'phase, 'effect, Schema> {
    selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    phase: &'a WorthQueryAdvancementPhase<'phase>,
    principal: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    scope: &'a WorthQueryRequestScope,
    source: ApplicationProgramRevision,
    target: &'a ApplicationProgramRevision,
    effect: AdoptionEffect<'effect>,
}
impl<'effect, Schema: ApplicationSchema> ApplicationProgramAdoptionAuthorizationVisitor<Schema>
    for AdmissionVisitor<'_, '_, '_, 'effect, Schema>
{
    type Output = Result<AdoptionEffectOutcome, AdoptionEffectRefusal<'effect>>;
    fn unavailable(self) -> Self::Output {
        Err(AdoptionEffectRefusal {
            cause: WorthQueryProgramAdoptionAuthorizationDenial::CapabilityNotDeclared,
            effect: self.effect,
        })
    }
    fn capability<Binding: ApplicationProgramAdoptionBinding<Schema>>(self) -> Self::Output {
        use WorthQueryProgramAdoptionAuthorizationDenial as Denial;
        let admitted = (|| {
            let declaration = ApplicationProgramAdoptionAuthorizationDeclaration::from_binding::<
                Schema,
                Binding,
            >();
            if !self
                .selected
                .application()
                .installed_program_support()
                .is_some_and(|support| {
                    support.adoption_authorization_matches(
                        &self.source,
                        declaration.canonical_meaning(),
                    )
                })
            {
                return Err(Denial::InstalledMeaningChanged);
            }
            let schema = self.selected.application().installed_schema();
            let binding = schema
                .principal_binding(Binding::principal_binding())
                .map_err(Denial::PrincipalInstallation)?;
            let principal = self
                .selected
                .resolve_authenticated_principal(
                    self.phase,
                    &binding,
                    self.principal,
                    self.scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .map_err(Denial::PrincipalResolution)?;
            let capability = schema.capability(ApplicationCapabilityRef::<Schema, Binding::Capability>::from_declaration(), ApplicationOperationRef::<Schema, Binding::Operation, Binding::Input>::from_declaration()).map_err(Denial::CapabilityInstallation)?;
            let input = Binding::input(ApplicationProgramAdoptionContext {
                source: &self.source,
                target: self.target,
            });
            let access = self
                .selected
                .admit_capability_access(self.phase, &principal, &capability, input, self.scope)
                .map_err(Denial::Authorization)?;
            let operation = schema
                .installed_operation(ApplicationOperationRef::<
                    Schema,
                    Binding::Operation,
                    Binding::Input,
                >::from_declaration())
                .map_err(Denial::OperationInstallation)?;
            let admission = self
                .selected
                .application()
                .authorize_capability_operation(self.phase, access, &operation, Default::default())
                .map_err(Denial::Authorization)?;
            Ok(admission)
        })();
        match admitted {
            Ok(admission) => Ok(self.effect.consume(
                self.selected,
                self.phase,
                self.scope,
                self.target,
                admission,
            )),
            Err(cause) => Err(AdoptionEffectRefusal {
                cause,
                effect: self.effect,
            }),
        }
    }
}
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    /// Read admission is consumed here and grants no later effect authority.
    pub fn authorize_program_adoption(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        target: &ApplicationProgramRevision,
    ) -> Result<(), WorthQueryProgramAdoptionAuthorizationDenial> {
        self.with_program_adoption(phase, principal, scope, target, AdoptionEffect::Read)
            .map_err(|refusal| refusal.cause)
            .and_then(|outcome| match outcome {
                AdoptionEffectOutcome::Read(result) => {
                    result.map_err(WorthQueryProgramAdoptionAuthorizationDenial::Authorization)
                }
                _ => unreachable!("read dispatch preserves its requested operation"),
            })
    }

    /// Admission always resolves the program activated on this exact branch.
    pub(in crate::domain_computation::primary_graph) fn with_program_adoption<'effect>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        target: &ApplicationProgramRevision,
        effect: AdoptionEffect<'effect>,
    ) -> Result<AdoptionEffectOutcome, AdoptionEffectRefusal<'effect>> {
        let source = match self.inspect_selected_program() {
            Ok(source) => *source.revision(),
            Err(cause) => {
                return Err(AdoptionEffectRefusal {
                    cause: WorthQueryProgramAdoptionAuthorizationDenial::Inspection(cause),
                    effect,
                })
            }
        };
        let Some(admission) = self
            .application()
            .installed_program_support()
            .and_then(|support| support.adoption_admission(&source))
        else {
            return Err(AdoptionEffectRefusal {
                cause: WorthQueryProgramAdoptionAuthorizationDenial::CapabilityNotDeclared,
                effect,
            });
        };
        admission(self, phase, principal, scope, target, effect)
    }
}

use worth_foundational::facade::CanonicalDigestDerivationDenial;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryAuthorizationRequirement, ApplicationQueryDisclosureContract,
        ErasedApplicationQueryDefinition,
    },
    application_schema::ApplicationSchema,
};

use crate::{
    application_query::{
        authority_seal::derive_installed_query_authority_seal,
        canonical_basis::prepare_installed_query_basis,
        WorthQueryApplicationQueryInstallationDenial,
        WorthQueryApplicationQueryInstallationDenialKind,
        WorthQueryInstalledApplicationContinuationContract,
        WorthQueryInstalledApplicationLiveContract,
        WorthQueryInstalledApplicationQueryAuthorization,
        WorthQueryInstalledApplicationQueryIdentity,
        WorthQueryInstalledApplicationReadFamilyBinding, WorthQueryInstalledGraphReadContract,
    },
    application_schema::WorthQueryInstalledApplicationSchema,
    canonical_work::WorthQueryCanonicalWorkEvidence,
    graph_obligation::{
        bind_query_obligations, WorthQueryGraphObligationInstallationDenial,
        WorthQueryInstalledGraphCapabilityRequirement,
    },
};

use super::WorthQueryCompiledApplicationQuery;

impl WorthQueryCompiledApplicationQuery {
    pub(crate) fn compile<Schema>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        definition: &ErasedApplicationQueryDefinition,
    ) -> Result<Self, WorthQueryApplicationQueryInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let canonical_work_policy =
            crate::application_query::WorthQueryApplicationQueryCanonicalWorkPolicy::for_definition(
                definition,
            );
        let read_graph = WorthQueryInstalledGraphReadContract::compile(
            definition,
            schema.binding_identity().schema_identity(),
            canonical_work_policy.installation(),
        )
        .map_err(|denial| canonical_work_denial(definition.name(), denial))?;
        let binding_identity = schema.binding_identity();
        let canonical = prepare_installed_query_basis(
            binding_identity.package_identity(),
            binding_identity.schema_identity(),
            definition,
            &read_graph,
            canonical_work_policy.installation(),
        )
        .map_err(|denial| canonical_work_denial(definition.name(), denial))?;
        let identity = WorthQueryInstalledApplicationQueryIdentity::from_canonical(&canonical);
        let continuation = WorthQueryInstalledApplicationContinuationContract::compile(
            definition,
            &read_graph,
            canonical_work_policy.installation(),
        )
        .map_err(|denial| canonical_work_denial(definition.name(), denial))?;
        let live = WorthQueryInstalledApplicationLiveContract::compile(
            definition,
            schema.installed_declaration(),
            &read_graph,
            continuation.as_ref(),
        )?;
        let authorization = install_authorization(schema, definition)?;
        let disclosure_capabilities =
            install_disclosure_capabilities(schema, definition.disclosure());
        let obligations = bind_query_obligations(
            &binding_identity,
            definition.name(),
            &identity,
            &read_graph,
            &authorization,
            &disclosure_capabilities,
        )
        .map_err(|denial| graph_obligation_denial(definition.name(), denial))?;
        let installation_canonical_work = schema.installation_canonical_work().combine(
            read_graph
                .canonical_basis()
                .work()
                .combine(read_graph.canonical_planning_basis().work())
                .combine(canonical.work())
                .combine(
                    continuation
                        .as_ref()
                        .map_or(WorthQueryCanonicalWorkEvidence::zero(), |contract| {
                            contract.canonical_basis().work()
                        }),
                )
                .combine(obligations.installation_evidence().canonical_work()),
        );
        let read_family = WorthQueryInstalledApplicationReadFamilyBinding::bind(read_graph);
        let authority_identity = derive_installed_query_authority_seal(
            &schema.package_authority.authority_key,
            &binding_identity,
            &identity,
            obligations.identity(),
        );
        Ok(Self {
            binding_identity,
            canonical,
            canonical_work_policy,
            installation_canonical_work,
            identity,
            authority_identity,
            definition: definition.clone(),
            parameter_type: definition.parameter_identity(),
            result_type: definition.result_identity(),
            read_family,
            continuation,
            live,
            authorization,
            obligations,
        })
    }
}

fn install_disclosure_capabilities<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    disclosure: &ApplicationQueryDisclosureContract,
) -> Vec<WorthQueryInstalledGraphCapabilityRequirement>
where
    Schema: ApplicationSchema,
{
    let (Some(name), Some(capability_type)) =
        (disclosure.capability_name(), disclosure.capability_type())
    else {
        return Vec::new();
    };
    schema
        .capability_registry
        .values()
        .filter(|compiled| {
            compiled.contract().name() == name
                && compiled.contract().capability_type() == capability_type
        })
        .map(|compiled| {
            WorthQueryInstalledGraphCapabilityRequirement::new(
                compiled.identity().clone(),
                compiled.contract().clone(),
            )
        })
        .collect()
}

fn install_authorization<Schema>(
    schema: &WorthQueryInstalledApplicationSchema<Schema>,
    definition: &ErasedApplicationQueryDefinition,
) -> Result<
    WorthQueryInstalledApplicationQueryAuthorization,
    WorthQueryApplicationQueryInstallationDenial,
>
where
    Schema: ApplicationSchema,
{
    match definition.authorization() {
        ApplicationQueryAuthorizationRequirement::Public => {
            Ok(WorthQueryInstalledApplicationQueryAuthorization::Public)
        }
        ApplicationQueryAuthorizationRequirement::Ability {
            ability,
            scope_entity,
        } => schema
            .installed_ability_requirement(ability, scope_entity)
            .cloned()
            .map(WorthQueryInstalledApplicationQueryAuthorization::Ability)
            .ok_or_else(|| {
                WorthQueryApplicationQueryInstallationDenial::new(
                    WorthQueryApplicationQueryInstallationDenialKind::AuthorizationNotInstalled,
                    definition.name(),
                )
            }),
    }
}

fn graph_obligation_denial(
    subject: &str,
    denial: WorthQueryGraphObligationInstallationDenial,
) -> WorthQueryApplicationQueryInstallationDenial {
    let kind = match denial {
        WorthQueryGraphObligationInstallationDenial::InvalidContract => {
            WorthQueryApplicationQueryInstallationDenialKind::InvalidGraphObligationContract
        }
        WorthQueryGraphObligationInstallationDenial::Canonical(denial) => {
            return canonical_work_denial(subject, denial)
        }
    };
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}

fn canonical_work_denial(
    subject: &str,
    denial: CanonicalDigestDerivationDenial,
) -> WorthQueryApplicationQueryInstallationDenial {
    let kind = match denial {
        CanonicalDigestDerivationDenial::EntryLimitExceeded { .. } => {
            WorthQueryApplicationQueryInstallationDenialKind::CanonicalEntryBudgetExceeded
        }
        CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. } => {
            WorthQueryApplicationQueryInstallationDenialKind::CanonicalEncodedByteBudgetExceeded
        }
        _ => WorthQueryApplicationQueryInstallationDenialKind::CanonicalDigestSlotRejected,
    };
    WorthQueryApplicationQueryInstallationDenial::new(kind, subject)
}

//! Contribution owners are admitted once before either memory or Home Native preparation.
use super::super::application_contribution::{
    PreparedApplicationRuntimeContributions, WorthQueryConfiguredApplicationContributions,
};
use super::super::{
    program_occurrence::WorthQueryProgramActivationCell, WorthQueryApplicationInvariantFactories,
};
use super::{
    program_admission::WorthQueryAdmittedProgramSupport,
    WorthQueryApplicationPreparationDenial as Denial,
};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

pub(super) fn prepare<Schema: ApplicationSchema>(
    configured: WorthQueryConfiguredApplicationContributions<Schema>,
    runtime: &WorthQueryExecutionRuntime,
    installed: &WorthQueryInstalledApplicationSchema<Schema>,
    admitted_program_support: &Option<WorthQueryAdmittedProgramSupport<Schema>>,
) -> Result<
    (
        WorthQueryApplicationInvariantFactories<Schema>,
        PreparedApplicationRuntimeContributions<Schema>,
        WorthQueryProgramActivationCell,
    ),
    Denial,
> {
    let (
        mut invariants,
        super::super::application_contribution::PreparedApplicationRuntimeContributions {
            handlers,
            producers,
            conditionals,
            resources,
            managed_indexes,
        },
    ) = configured.into_parts().map_err(Denial::Contributions)?;
    conditionals
        .validate_target_coverage(
            runtime.installed_packages(),
            installed.owner(),
            installed.schema_name(),
        )
        .map_err(Denial::Contributions)?;
    if let Some(support) = &admitted_program_support {
        handlers
            .validate_managed_computations(&support.features)
            .map_err(Denial::Contributions)?;
        handlers
            .validate_derived_collections(&support.features)
            .map_err(Denial::Contributions)?;
    }
    let activation =
        super::super::program_occurrence::WorthQueryProgramActivationCell::unpublished();
    if let Some(support) = &admitted_program_support {
        invariants.select_by_program(
            super::super::invariant_installation::WorthQueryInvariantProgramBasis::admitted(
                std::sync::Arc::clone(&support.roster),
                activation.clone(),
            ),
        );
    }
    Ok((
        invariants,
        PreparedApplicationRuntimeContributions {
            handlers,
            producers,
            conditionals,
            resources,
            managed_indexes,
        },
        activation,
    ))
}

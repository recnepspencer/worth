//! Derives exact invariant obligations; aggregate restrictions belong to the request.

use std::num::NonZeroU32;

use super::compilation::APPLICATION_INVARIANT_SLOT;
use crate::domain_operation::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
    WorthQueryInvariantExecutionContract, WorthQueryOperationEffectContract,
    WorthQueryOperationEffectFamily, WorthQueryOperationInvariantContract,
};

pub(super) fn mutation_contracts(
    graph_mutation_count: usize,
    invariant_invocations: &[crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor],
) -> Result<
    (
        WorthQueryOperationEffectContract,
        WorthQueryOperationInvariantContract,
        WorthQueryInvariantExecutionContract,
    ),
    (),
> {
    if graph_mutation_count == 0 {
        return Ok((
            WorthQueryOperationEffectContract::NotRequired,
            WorthQueryOperationInvariantContract::NotRequired,
            WorthQueryInvariantExecutionContract::NotRequired,
        ));
    }
    let invariant_execution = application_invariant_execution_contract(invariant_invocations)?;
    let invariant_slots = invariant_execution
        .requirements()
        .iter()
        .map(|requirement| requirement.slot().to_owned())
        .collect();
    Ok((
        WorthQueryOperationEffectContract::Declared {
            effect_families: vec![WorthQueryOperationEffectFamily::Mutation],
        },
        WorthQueryOperationInvariantContract::Declared { invariant_slots },
        invariant_execution,
    ))
}

fn application_invariant_execution_contract(
    invariant_invocations: &[crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor],
) -> Result<WorthQueryInvariantExecutionContract, ()> {
    let native = WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
        APPLICATION_INVARIANT_SLOT,
        "application-installed-invariants",
        NonZeroU32::new(1).expect("one is nonzero"),
        WorthQueryInvariantEnforcement::Blocking,
        "primary",
        ["application-proposed-state"],
        None,
        None,
    )
    .map_err(|_| ())?;
    let custom = invariant_invocations.iter().map(|invariant| {
        let point = match invariant.execution_point() {
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::CommitBoundary => "commit-boundary",
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::MutationSensitive => "mutation-sensitive",
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint::SnapshotPublication => "snapshot-publication",
        };
        let version = (u32::from(invariant.major()) << 16) | u32::from(invariant.minor());
        WorthQueryInstalledInvariantExecutionRequirement::with_optional_bounds(
            format!("relational-custom:{}@{point}", invariant.identifier()),
            "relational-custom-invariant",
            NonZeroU32::new(version).ok_or(())?,
            WorthQueryInvariantEnforcement::Blocking,
            "primary",
            ["application-proposed-state"],
            None,
            None,
        ).map_err(|_| ())
            .map(|requirement| requirement.with_application_invariant(invariant.clone()))
    }).collect::<Result<Vec<_>, ()>>()?;
    WorthQueryInvariantExecutionContract::declared(std::iter::once(native).chain(custom))
        .map_err(|_| ())
}

impl WorthQueryInvariantExecutionContract {
    /// Compiles native publication obligations from the installed invariant catalog.
    /// This carries validation duties without granting application mutation scopes.
    pub fn for_installed_application_invariants(
        invariants: &[crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor],
    ) -> Result<Self, &'static str> {
        if invariants.is_empty() {
            return Ok(Self::NotRequired);
        }
        application_invariant_execution_contract(invariants)
            .map_err(|()| "invalid-installed-application-invariant-contract")
    }
}

#[cfg(test)]
mod tests {
    use super::application_invariant_execution_contract;
    use crate::application_schema::WorthQueryInstalledApplicationInvariantDescriptor;
    use std::num::NonZeroU64;
    use worth_query_declaration::facade::application_schema::{
        ApplicationInvariantCostPosture, ApplicationInvariantEnforcement,
        ApplicationInvariantExecutionPoint, ApplicationInvariantGroup,
        ApplicationInvariantScopeTarget,
    };

    #[test]
    fn generated_invariants_keep_algorithm_contract_without_aggregate_proxy() {
        for work in [None, NonZeroU64::new(7)] {
            let descriptor = custom_invariant(work);
            let contract =
                application_invariant_execution_contract(std::slice::from_ref(&descriptor))
                    .unwrap();
            for requirement in contract.requirements() {
                assert_eq!(requirement.max_state_facts(), None);
                assert_eq!(requirement.max_work_units(), None);
            }
            let custom = contract
                .requirements()
                .iter()
                .find_map(|r| r.application_invariant())
                .unwrap();
            assert_eq!(custom.maximum_work_units(), work);
            assert_eq!(custom, &descriptor);
        }
    }

    #[test]
    fn native_reference_contract_compiles_exact_invariants_without_application_mutation() {
        let descriptor = custom_invariant(NonZeroU64::new(7));
        let native =
            super::WorthQueryInvariantExecutionContract::for_installed_application_invariants(
                std::slice::from_ref(&descriptor),
            )
            .unwrap();
        let required = native
            .requirements()
            .iter()
            .find_map(|r| r.application_invariant())
            .unwrap();
        assert_eq!(required, &descriptor);
        let (effects, _, control) = super::mutation_contracts(0, &[descriptor]).unwrap();
        assert!(matches!(
            effects,
            super::WorthQueryOperationEffectContract::NotRequired
        ));
        assert!(
            control.requirements().is_empty(),
            "read-only control retains its original authority"
        );
        assert!(
            super::WorthQueryInvariantExecutionContract::for_installed_application_invariants(&[])
                .unwrap()
                .requirements()
                .is_empty()
        );
    }

    fn custom_invariant(
        work: Option<NonZeroU64>,
    ) -> WorthQueryInstalledApplicationInvariantDescriptor {
        WorthQueryInstalledApplicationInvariantDescriptor::from_installed_parts(
            "budget-regression".to_owned(),
            1,
            0,
            ApplicationInvariantExecutionPoint::CommitBoundary,
            work,
            ApplicationInvariantEnforcement::BlockCommit,
            vec![ApplicationInvariantGroup::SchemaCompliance],
            vec![ApplicationInvariantScopeTarget::Entity("item".to_owned())],
            vec![ApplicationInvariantScopeTarget::Entity("item".to_owned())],
            "primary".to_owned(),
            ApplicationInvariantCostPosture::Touched,
        )
    }
}

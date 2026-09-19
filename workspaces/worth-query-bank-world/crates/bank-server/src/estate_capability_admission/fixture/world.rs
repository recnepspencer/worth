use bank_domain::{
    estate::{BankEstateWorld, CapabilityGrantId, EstateWorkflowStage},
    model::BankPrincipalId,
};
use worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity;

use super::{
    base_estate, extra_principal, grant, AuthorizationTimeController, GrantSpec,
    ALTERNATE_EMERGENCY_BOUND_GRANT, EXECUTOR, GRANT, SPECIALIST,
};
use crate::BankIdentityRuntime;

#[path = "world/delegation.rs"]
mod delegation;
#[path = "world/disclosure.rs"]
mod disclosure;
#[path = "world/foreign_estate.rs"]
mod foreign_estate;
#[path = "world/governance_projection.rs"]
mod governance_projection;
#[path = "world/lifecycle.rs"]
mod lifecycle;
#[path = "world/product_projection.rs"]
mod product_projection;
#[path = "world/seed.rs"]
mod seed;
#[path = "world/snapshot.rs"]
mod snapshot_fixture;
#[path = "world/warm_locality.rs"]
mod warm_locality;

pub(crate) use foreign_estate::{foreign_estate_revocation_world, FOREIGN_ESTATE, FOREIGN_GRANT};

pub(super) struct FixtureWorldSpec<'a> {
    pub(super) scenario: &'a str,
    pub(super) spec: GrantSpec,
    pub(super) case_stage: EstateWorkflowStage,
    pub(super) specialist_holds_authority: bool,
    pub(super) unrelated_grants: usize,
    pub(super) composition: FixtureWorldComposition,
    pub(super) alternate_emergency_bound: Option<GrantSpec>,
}

#[derive(Clone, Copy)]
pub(super) enum FixtureWorldComposition {
    Admission,
    Disclosure {
        beneficiary: BankPrincipalId,
    },
    Lifecycle,
    GovernanceProjection,
    ProductProjection,
    WarmLocality {
        axis: WarmLocalityAxis,
        count: usize,
    },
    Delegation {
        command_authority: bool,
        parent_context: DelegationParentContext,
    },
    ForeignEstateRevocation,
}

#[derive(Clone, Copy)]
pub(crate) enum WarmLocalityAxis {
    Grants,
    Relationships,
    Fields,
    Cases,
    ResultRows,
}

#[derive(Clone, Copy)]
pub(super) enum DelegationParentContext {
    Exact,
    Branch,
    Institution,
}

pub(super) struct InstalledFixtureWorld {
    pub(super) runtime: BankIdentityRuntime,
    pub(super) estate_world: BankEstateWorld,
    pub(super) identities: [WorthQueryExternalPrincipalIdentity; 5],
}

pub(super) fn install_fixture_world(spec: FixtureWorldSpec<'_>) -> InstalledFixtureWorld {
    install_fixture_world_with_source(spec, None)
}

pub(super) fn install_fixture_world_with_authorization_time(
    spec: FixtureWorldSpec<'_>,
    authorization_time: AuthorizationTimeController,
) -> InstalledFixtureWorld {
    install_fixture_world_with_source(spec, Some(authorization_time))
}

fn install_fixture_world_with_source(
    spec: FixtureWorldSpec<'_>,
    authorization_time: Option<AuthorizationTimeController>,
) -> InstalledFixtureWorld {
    let identities = seed::identities(spec.scenario);
    let snapshot = snapshot_fixture::snapshot(additional_principal_count(&spec));
    let estate = compose_estate(&spec);
    let estate_world = estate.clone();
    let world_seed = seed::assemble(snapshot, estate, &identities, &spec);
    let runtime = install_runtime(world_seed, authorization_time);
    InstalledFixtureWorld {
        runtime,
        estate_world,
        identities,
    }
}

fn install_runtime(
    seed: crate::BankWorldSeed,
    authorization_time: Option<AuthorizationTimeController>,
) -> BankIdentityRuntime {
    match authorization_time {
        Some(source) => {
            BankIdentityRuntime::install_world_with_authorization_time_source(seed, source)
        }
        None => BankIdentityRuntime::install_world(seed),
    }
    .expect("capability fixture runtime should install")
}

fn compose_estate(spec: &FixtureWorldSpec<'_>) -> BankEstateWorld {
    let authority_holder = if spec.specialist_holds_authority {
        SPECIALIST
    } else {
        EXECUTOR
    };
    let estate = base_estate(spec.case_stage, authority_holder)
        .with_grant(grant(GRANT, SPECIALIST, spec.spec));
    let estate = install_scenario_truth(estate, spec);
    install_scenario_scale(estate, spec)
}

fn install_scenario_truth(estate: BankEstateWorld, spec: &FixtureWorldSpec<'_>) -> BankEstateWorld {
    match spec.composition {
        FixtureWorldComposition::Admission => estate,
        FixtureWorldComposition::Disclosure { beneficiary } => {
            disclosure::install_present_beneficiary(estate, super::ESTATE, beneficiary)
        }
        FixtureWorldComposition::Lifecycle => lifecycle::install_grants(estate),
        FixtureWorldComposition::GovernanceProjection => {
            governance_projection::install_truth(estate)
        }
        FixtureWorldComposition::ProductProjection => {
            product_projection::install_truth(estate, spec.spec)
        }
        FixtureWorldComposition::WarmLocality { axis, count } => {
            let estate = product_projection::install_truth(estate, spec.spec);
            warm_locality::install_growth(estate, axis, count)
        }
        FixtureWorldComposition::Delegation {
            command_authority,
            parent_context,
        } => delegation::install_grants(estate, command_authority, parent_context, spec.spec),
        FixtureWorldComposition::ForeignEstateRevocation => {
            foreign_estate::install_foreign_estate_revocation(estate)
        }
    }
}

fn additional_principal_count(spec: &FixtureWorldSpec<'_>) -> usize {
    match spec.composition {
        FixtureWorldComposition::WarmLocality {
            axis: WarmLocalityAxis::Relationships,
            count,
        } => spec.unrelated_grants.max(count),
        _ => spec.unrelated_grants,
    }
}

fn install_scenario_scale(
    mut estate: BankEstateWorld,
    spec: &FixtureWorldSpec<'_>,
) -> BankEstateWorld {
    if let Some(alternate) = spec.alternate_emergency_bound {
        estate = estate.with_grant(grant(
            ALTERNATE_EMERGENCY_BOUND_GRANT,
            SPECIALIST,
            alternate,
        ));
    }
    for ordinal in 0..spec.unrelated_grants {
        estate = estate.with_grant(grant(
            CapabilityGrantId::new(2_000 + ordinal as u64).unwrap(),
            extra_principal(ordinal),
            GrantSpec::view(),
        ));
    }
    estate
}

use super::authorization_world_installation::{
    install_authorization_world, AuthorizationWorld, AuthorizationWorldSpec,
    CapabilityGrantPopulation,
};
use super::WorthQueryApplicationQueryResourceProfile;

pub(in crate::domain_computation::primary_graph) fn installed_capability_authorization_world(
) -> AuthorizationWorld {
    capability_world(1, "primary", CapabilityGrantPopulation::Current)
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_world_with_label(
    label: &str,
) -> AuthorizationWorld {
    capability_world(1, label, CapabilityGrantPopulation::Current)
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_live_world(
) -> AuthorizationWorld {
    installed_capability_live_world_with_label("primary")
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_live_world_with_label(
    label: &str,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: &[("principal-1", "account-1")],
        ..capability_spec(2, label, CapabilityGrantPopulation::Current)
    })
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_replacement_world(
) -> AuthorizationWorld {
    capability_world(
        1,
        "primary",
        CapabilityGrantPopulation::CurrentAndFutureReplacement,
    )
}

pub(in crate::domain_computation) fn installed_composed_capability_world(
    scenario: super::capability_seed::CapabilityCompositionScenario,
) -> AuthorizationWorld {
    capability_world(1, "primary", CapabilityGrantPopulation::Composed(scenario))
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_world_with_same_resource_unrelated(
    unrelated: usize,
) -> AuthorizationWorld {
    capability_world(
        2,
        "primary",
        CapabilityGrantPopulation::CurrentWithSameResourceUnrelated(unrelated),
    )
}

pub(in crate::domain_computation::primary_graph) fn installed_capability_world_with_exact_pair_population(
    count: usize,
) -> AuthorizationWorld {
    capability_world(
        1,
        "primary",
        CapabilityGrantPopulation::ExactPairPopulation(count),
    )
}

pub(in crate::domain_computation::primary_graph) fn installed_delegated_capability_world(
) -> AuthorizationWorld {
    installed_delegated_capability_world_at_depth(2)
}

pub(in crate::domain_computation::primary_graph) fn installed_delegated_capability_world_at_depth(
    links: usize,
) -> AuthorizationWorld {
    capability_world(
        2,
        "primary",
        CapabilityGrantPopulation::Delegated {
            links,
            unrelated: 0,
        },
    )
}

pub(in crate::domain_computation::primary_graph) fn installed_delegated_capability_world_with_unrelated(
    unrelated: usize,
) -> AuthorizationWorld {
    capability_world(
        2,
        "primary",
        CapabilityGrantPopulation::Delegated {
            links: 2,
            unrelated,
        },
    )
}

pub(in crate::domain_computation::primary_graph) fn installed_elevated_capability_world(
    scenario: super::capability_elevation_seed::CapabilityElevationScenario,
) -> AuthorizationWorld {
    capability_world(3, "primary", CapabilityGrantPopulation::Elevated(scenario))
}

pub(in crate::domain_computation::primary_graph) fn installed_elevated_capability_live_world(
    scenario: super::capability_elevation_seed::CapabilityElevationScenario,
) -> AuthorizationWorld {
    install_authorization_world(AuthorizationWorldSpec {
        owner_bindings: &[("principal-2", "account-1")],
        ..capability_spec(3, "primary", CapabilityGrantPopulation::Elevated(scenario))
    })
}

fn capability_world(
    principal_count: usize,
    label: &str,
    grants: CapabilityGrantPopulation,
) -> AuthorizationWorld {
    install_authorization_world(capability_spec(principal_count, label, grants))
}

fn capability_spec(
    principal_count: usize,
    primary_label: &str,
    capability_grants: CapabilityGrantPopulation,
) -> AuthorizationWorldSpec<'_> {
    AuthorizationWorldSpec {
        owner_bindings: &[],
        blocked: false,
        principal_count,
        primary_label,
        resources: WorthQueryApplicationQueryResourceProfile::default(),
        capability_grants,
    }
}

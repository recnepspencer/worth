use std::num::NonZeroUsize;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope, primary_graph, runtime,
};

use super::adapters::block_on;
use super::schema::*;
use super::world::{self, CourtroomWorld};

pub(super) fn principal(
    world: &CourtroomWorld,
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryAuthenticatedPrincipal<TemporalHostSchema, Principal, u64> {
    let schema = world.application.installed_schema();
    let binding = schema
        .principal_binding(TemporalPrincipalBinding::reference())
        .unwrap();
    let authentication = world::admit_identity_adapter(schema);
    let external = block_on(authentication.authenticate((), request)).unwrap();
    world
        .application
        .select_product_branch(world.application.product_runtime().default_branch())
        .expect("the selected product branch remains admitted")
        .resolve_authenticated_principal(
            &binding,
            external,
            request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(super) fn fork_relational_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQueryProductBranchLease,
    name: &str,
    relational_name: &str,
) -> runtime::ProductBranchIdentity {
    let intent = runtime::ProductBranchCreationIntent::from_source(
        name,
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ForkExact {
                target: runtime::BranchId(relational_name.to_owned()),
            },
            runtime::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let outcome = world
        .application
        .product_runtime()
        .create_product_branch(
            source,
            intent,
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("fork must publish an exact sibling product: {outcome:?}")
    };
    observation.branch_identity().clone()
}

pub(super) fn fork_independent_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQueryProductBranchLease,
    name: &str,
    relational_name: &str,
    signal_name: &str,
) -> runtime::ProductBranchIdentity {
    let intent = runtime::ProductBranchCreationIntent::from_source(
        name,
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ForkExact {
                target: runtime::BranchId(relational_name.to_owned()),
            },
            runtime::SignalBranchCreationPlan::ForkExact {
                target: runtime::validate_signal_branch_name(signal_name)
                    .expect("the independent Signal branch name validates"),
            },
        ),
    )
    .unwrap();
    let outcome = world
        .application
        .product_runtime()
        .create_product_branch(
            source,
            intent,
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("independent fork must publish both product components: {outcome:?}")
    };
    observation.branch_identity().clone()
}

pub(super) fn reuse_exact_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQueryProductBranchLease,
    name: &str,
) -> runtime::ProductBranchIdentity {
    let intent = runtime::ProductBranchCreationIntent::from_source(
        name,
        runtime::ProductBranchCreationPlans::new(
            runtime::RelationalBranchCreationPlan::ReuseExact,
            runtime::SignalBranchCreationPlan::ReuseExact,
        ),
    )
    .unwrap();
    let outcome = world
        .application
        .product_runtime()
        .create_product_branch(
            source,
            intent,
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let runtime::RuntimeWorldBranchCreationOutcome::Performed(observation) = outcome else {
        panic!("exact reuse must publish a product sibling: {outcome:?}")
    };
    observation.branch_identity().clone()
}

pub(super) fn controls(
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryProductQueryControls<'_> {
    primary_graph::WorthQueryProductQueryControls::new(
        NonZeroUsize::new(8).unwrap(),
        NonZeroUsize::new(64).unwrap(),
        request,
    )
}

pub(super) fn product_identity(
    receipt: &primary_graph::WorthQueryApplicationQueryAccessReceipt,
) -> &primary_graph::WorthQueryProductBranchReadIdentity {
    let primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(identity) =
        receipt.basis_identity().selection()
    else {
        panic!("selected product custody must survive into the normal query receipt")
    };
    identity
}

pub(super) fn assert_security_work(
    receipt: &primary_graph::WorthQueryApplicationQueryAccessReceipt,
) {
    assert_eq!(
        receipt
            .authorization_work()
            .admission_security_product_resolutions(),
        1
    );
    assert_eq!(
        receipt
            .authorization_work()
            .execution_security_product_resolutions(),
        1
    );
}

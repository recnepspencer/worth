use std::num::NonZeroUsize;

use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope, primary_graph, product,
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
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .expect("the selected product branch remains admitted")
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens")
}

pub(super) fn fork_relational_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    _name: &str,
    _relational_name: &str,
) -> product::WorthQueryProductBranch {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source.product().product_branch())
                .components(|components| components.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .expect("fork must publish an exact sibling product")
        })
        .expect("installed host call opens")
}

pub(super) fn fork_independent_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    _name: &str,
    _relational_name: &str,
    _signal_name: &str,
) -> product::WorthQueryProductBranch {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source.product().product_branch())
                .components(|components| components.fork_relational().fork_signal())
                .create(&phase)
                .expect("independent fork must publish both product components")
        })
        .expect("installed host call opens")
}

pub(super) fn reuse_exact_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    _name: &str,
) -> product::WorthQueryProductBranch {
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .branches()
                .fork(source.product().product_branch())
                .components(|components| {
                    components
                        .reuse_exact_relational_basis()
                        .reuse_exact_signal_basis()
                })
                .create(&phase)
                .expect("exact reuse must publish a product sibling")
        })
        .expect("installed host call opens")
}

pub(super) fn controls(
    request: &WorthQueryRequestScope,
) -> primary_graph::WorthQueryProductQueryControls<'_> {
    primary_graph::WorthQueryProductQueryControls::new(NonZeroUsize::new(8).unwrap(), None, request)
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

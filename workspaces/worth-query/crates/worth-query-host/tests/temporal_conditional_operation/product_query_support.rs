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
        .on_branch(world.application.current_world())
        .select()
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
    source: &primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    _name: &str,
    _relational_name: &str,
) -> product::WorthQueryProductBranch {
    world
        .application
        .branches()
        .fork(source.product().product_branch())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("fork must publish an exact sibling product")
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
        .branches()
        .fork(source.product().product_branch())
        .components(|components| components.fork_relational().fork_signal())
        .create()
        .expect("independent fork must publish both product components")
}

pub(super) fn reuse_exact_product(
    world: &CourtroomWorld,
    source: &primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
    _name: &str,
) -> product::WorthQueryProductBranch {
    world
        .application
        .branches()
        .fork(source.product().product_branch())
        .components(|components| {
            components
                .reuse_exact_relational_basis()
                .reuse_exact_signal_basis()
        })
        .create()
        .expect("exact reuse must publish a product sibling")
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

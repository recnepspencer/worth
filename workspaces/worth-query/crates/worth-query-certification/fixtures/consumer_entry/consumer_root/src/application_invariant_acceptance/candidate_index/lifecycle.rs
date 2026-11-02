//! The same physical lifecycle harness, with this composition's actual principal.
use super::super::authentication;
pub(super) use super::super::managed_index::lifecycle::{isolated_allocation_case, run, ALLOCATOR};
use super::{host, schema::Catalog};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
pub(super) fn authenticate(
    world: &host::Host,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}

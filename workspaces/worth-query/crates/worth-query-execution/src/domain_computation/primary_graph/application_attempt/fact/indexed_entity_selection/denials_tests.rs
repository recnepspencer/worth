use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, AccountIdentity, AccountNote,
};

#[test]
fn indexed_reobservation_preserves_native_request_and_lookup_denials() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountNote::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let kind = graph
        .layout
        .entity_kind(AccountIdentity::reference().entity())
        .unwrap();
    let value = StringApplicationValueBinding::encode(&"reviewed".to_owned()).unwrap();
    graph.integration_handle().with_runtime(|runtime| {
        for (limit, expected) in [
            (0, BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit),
            (2, BoundedEntityFieldLookupDenialKind::IndexNotInstalled),
        ] {
            let result = observe_checked(
                runtime,
                selected.application_basis().snapshot_handle(),
                DerivedIndexId(u64::MAX),
                kind,
                locator.clone(),
                value.clone(),
                limit,
            );
            assert_eq!(
                result.unwrap_err(),
                IndexedSelectionReobserveDenial::Lookup(expected),
                "the native denial must survive Query's fact observation boundary"
            );
        }
    });
}

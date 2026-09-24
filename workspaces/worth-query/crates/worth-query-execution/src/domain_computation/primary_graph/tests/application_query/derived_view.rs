use std::collections::BTreeMap;
use std::time::Duration;

use worth_query_declaration::facade::application_query::{
    ApplicationDerivedViewDefinition, ApplicationDerivedViewLimits, ApplicationQueryParameterSet,
};
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::ApplicationScalarValueBinding;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::{
    current_controls, installed_authorization_world, installed_ordered_query, installed_query,
    live_scope, status_parameter, AccountStatus, AccountSummaryQuery, OrderedAccountSummaryQuery,
};
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial, WorthQueryPrincipalResolutionMode,
};

struct SceneLabel(String);

impl WorthQueryManagedDerivedValue for SceneLabel {
    fn retained_bytes(&self) -> usize {
        self.0.capacity()
    }
}

#[test]
fn governed_query_cannot_open_a_persistent_view_without_read_reauthorization() {
    let world = installed_authorization_world(true);
    let query = installed_query(&world);
    let definition = ApplicationDerivedViewDefinition::new(
        "governed-account",
        AccountSummaryQuery::reference(),
        ApplicationDerivedViewLimits::bounded(8, 4096),
    );
    let selected = world.selected_product();
    let denial = world
        .application
        .open_managed_derived_view::<_, _, _, _, SceneLabel>(
            &definition,
            &query,
            selected.product(),
        )
        .err();
    assert_eq!(
        denial,
        Some(WorthQueryManagedDerivedViewDenial::AuthorizationRequired)
    );
}

#[test]
fn public_query_result_reconstructs_disposable_managed_view_from_owner_observations() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = installed_ordered_query(&world);
    let definition = ApplicationDerivedViewDefinition::new(
        "scene-labels",
        OrderedAccountSummaryQuery::reference(),
        ApplicationDerivedViewLimits::bounded(8, 4096),
    );
    let view = world
        .application
        .open_managed_derived_view(&definition, &query, selected.product())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let result = world
        .application
        .execute_application_query_one_shot(
            selected
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    let retained_product = world.selected_product();
    let foreign = installed_authorization_world(true);
    assert_eq!(
        world
            .application
            .reconstruct_managed_derived_view(
                &view,
                foreign.selected_product().product(),
                &result,
                |row| SceneLabel(row.label().to_string()),
            )
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::ForeignBranch)
    );
    let tiny = world
        .application
        .open_managed_derived_view(
            &ApplicationDerivedViewDefinition::new(
                "tiny-scene",
                OrderedAccountSummaryQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 1),
            ),
            &query,
            retained_product.product(),
        )
        .unwrap();
    assert_eq!(
        world
            .application
            .reconstruct_managed_derived_view(&tiny, retained_product.product(), &result, |row| {
                SceneLabel(row.label().to_string())
            })
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
    );
    drop(tiny);
    world
        .application
        .reconstruct_managed_derived_view(&view, retained_product.product(), &result, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    let observed = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    let key = result.observed_sources()[0].managed_derived_view_key();
    assert_eq!(observed.get(&key).unwrap().unwrap().0, "primary");
    view.discard();
    assert!(matches!(
        world.application.observe_managed_derived_view(&view),
        Ok(_)
    ));
    assert_eq!(
        observed.get(&key).err(),
        Some(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
    );
    world
        .application
        .reconstruct_managed_derived_view(&view, retained_product.product(), &result, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    drop(retained_product);
    let graph = world.application.runtime.primary_graph().unwrap();
    let field = AccountLabel::reference();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap()
        .clone();
    let fields = AspectFieldPatch::from(BTreeMap::from([(
        locator,
        StringApplicationValueBinding::encode(&"changed".to_string()).unwrap(),
    )]));
    super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("direct-world-change").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: account.entity_id(),
                fields,
            }),
        )),
    );
    assert!(matches!(
        world.application.observe_managed_derived_view(&view),
        Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired)
    ));
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let account = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_string(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &account);
    let fresh = world
        .application
        .execute_application_query_one_shot(
            selected
                .admit_application_query(
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(status_parameter(), "open".to_string())
                        .unwrap(),
                    current_controls(&request),
                )
                .unwrap(),
        )
        .unwrap();
    let fresh_product = world.selected_product();
    world
        .application
        .reconstruct_managed_derived_view(&view, fresh_product.product(), &fresh, |row| {
            SceneLabel(row.label().to_string())
        })
        .unwrap();
    let fresh_key = fresh.observed_sources()[0].managed_derived_view_key();
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(&fresh_key)
            .unwrap()
            .unwrap()
            .0,
        "changed"
    );
}

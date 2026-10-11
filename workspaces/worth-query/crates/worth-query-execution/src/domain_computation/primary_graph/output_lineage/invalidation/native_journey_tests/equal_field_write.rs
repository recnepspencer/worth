//! Equal field writes preserve native stamps and the installed actor's marks.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use super::*;

#[test]
fn equal_native_field_write_keeps_revision_and_downstream_clean_until_a_real_change() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let source = |output_binding| {
        SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding,
    }
    };
    let output = RecordedSettlementIdentity::retain(
        &source(OutputBindingIdentity::declared("StatusOutput")),
        coordinate,
        0,
    );
    let downstream = RecordedSettlementIdentity::retain(
        &source(OutputBindingIdentity::declared("DownstreamOutput")),
        coordinate,
        0,
    );

    handle.with_runtime_mut(|runtime| {
        write_field(runtime, entity, label, "prime");
        let (before_handle, before) = snapshot(runtime);
        let before_fact = field_fact(runtime, &before_handle, entity, status.clone());
        register(
            owner,
            Arc::clone(&output),
            Arc::from([before_fact.clone()]),
            &before,
            OrdSet::new(),
        );
        register(
            owner,
            Arc::clone(&downstream),
            Arc::from([]),
            &before,
            OrdSet::unit(Arc::clone(&output)),
        );

        write_field(runtime, entity, status.clone(), "open");
        let (equal_handle, equal) = snapshot(runtime);
        let equal_fact = field_fact(runtime, &equal_handle, entity, status.clone());
        assert_eq!(revision(&equal_fact), revision(&before_fact));
        assert!(matches!(
            currentness(owner, &equal, &output),
            SourceSettlementCurrentness::Clean
        ));
        assert!(matches!(
            currentness(owner, &equal, &downstream),
            SourceSettlementCurrentness::Clean
        ));

        write_field(runtime, entity, status.clone(), "closed");
        let (changed_handle, changed) = snapshot(runtime);
        let changed_fact = field_fact(runtime, &changed_handle, entity, status);
        assert_ne!(revision(&changed_fact), revision(&equal_fact));
        assert!(matches!(
            currentness(owner, &changed, &output),
            SourceSettlementCurrentness::Dirty(_)
        ));
        match currentness(owner, &changed, &downstream) {
            SourceSettlementCurrentness::PendingUpstream(upstream) => {
                assert_eq!(upstream, OrdSet::unit(output))
            }
            _ => panic!("a real native field change must mark the actual downstream"),
        }
        for snapshot in [before_handle, equal_handle, changed_handle] {
            runtime.snapshots().release_snapshot(&snapshot).unwrap();
        }
    });
}

fn revision(
    fact: &WorthQueryApplicationObservedFact,
) -> worth_relational::facade::runtime::RelationalFieldRevision {
    match fact {
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision, ..
        } => native_revision.expect("the actual native snapshot must issue a field revision"),
        _ => panic!("the real field reader must issue a field revision"),
    }
}

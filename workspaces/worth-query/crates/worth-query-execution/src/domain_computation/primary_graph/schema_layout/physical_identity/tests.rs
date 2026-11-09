//! Allocation laws are value tests; actual Home service is a separate court.
use super::value_layout::{lower, schema};
use super::*;
use worth_relational::facade::schema::{SchemaId, SchemaVersionId};

#[test]
fn earlier_sorting_application_roles_preserve_every_retained_physical_role() {
    let a = lower(&schema(false, false, false), None);
    let b = lower(&schema(true, true, false), Some(&a.roles));
    for (name, id) in &a.roles.entities {
        assert_eq!(b.roles.entities[name], *id);
    }
    for (name, id) in &a.roles.relations {
        assert_eq!(b.roles.relations[name], *id);
    }
    assert_eq!(b.roles.platform_first, a.roles.platform_first);
    assert_eq!(b.roles.platform_aspects, a.roles.platform_aspects);
    assert_eq!(b.roles.bootstrap, a.roles.bootstrap);
    assert!(b.roles.entities["Earlier"] > a.roles.maximum_kind.unwrap());
    assert!(b.roles.relations["EarlierEdge"] > a.roles.maximum_kind.unwrap());
    // Check every actual fixed platform role, including appended workflow IDs.
    let previous = a.registry.authority_snapshot();
    for old in &previous.entity_kinds {
        if !a.roles.entity_ids.contains(&old.kind_id) {
            assert_eq!(
                b.registry
                    .entity_registration(old.kind_id)
                    .unwrap()
                    .kind_name,
                old.kind_name
            );
        }
    }
    for old in &previous.relation_kinds {
        if !a.roles.relation_ids.contains(&old.kind_id) {
            assert_eq!(
                b.registry
                    .relation_registration(old.kind_id)
                    .unwrap()
                    .kind_name,
                old.kind_name
            );
        }
    }
}

#[test]
fn exact_authored_contracts_distinguish_an_application_with_a_platform_name() {
    let declaration = schema(false, false, false);
    let a = lower(&declaration, None);
    let (entities, relations) = source_application::resolve(
        &declaration,
        &value_layout::contracts(),
        &a.registry,
        &a.registry.authority_snapshot(),
    )
    .unwrap();
    assert_eq!(entities, a.roles.entities);
    assert_eq!(relations, a.roles.relations);
    assert_ne!(
        entities["worth-query-provider-idempotency"],
        a.roles.platform_first
    );
    let readmitted = PhysicalIdentityBindings::from_compiled(
        &a.registry,
        entities,
        relations,
        a.roles.maximum_kind,
        a.roles.maximum_aspect,
    )
    .unwrap();
    let receiver = lower(&declaration, Some(&readmitted));
    readmitted
        .require_original(&receiver.registry, &a.registry)
        .unwrap();
}

#[test]
fn two_exact_original_application_candidates_refuse_instead_of_choosing_an_id() {
    let declaration = schema(false, false, false);
    let a = lower(&declaration, None);
    let duplicate = super::super::registry_lowering::register_entity(
        a.registry,
        &SchemaId("pure-role-values".into()),
        SchemaVersionId(1),
        "Zulu",
        KindId(a.roles.maximum_kind.unwrap().0 + 1),
        Vec::new(),
    )
    .unwrap();
    let refused = source_application::resolve(
        &declaration,
        &value_layout::contracts(),
        &duplicate,
        &duplicate.authority_snapshot(),
    )
    .unwrap_err();
    assert_eq!(
        refused.kind(),
        super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
    );
    assert_eq!(
        refused.subject(),
        "ambiguous original application entity role"
    );
}

#[test]
fn inserting_an_aggregate_definition_keeps_its_existing_membership_and_storage() {
    let a = lower(&schema(false, false, true), None);
    let b = lower(&schema(false, true, true), Some(&a.roles));
    assert_eq!(b.roles.aggregate_storage, a.roles.aggregate_storage);
    for (definition, kind) in &a.roles.memberships {
        assert_eq!(b.roles.memberships[definition], *kind);
    }
    assert_eq!(a.roles.memberships.len(), 1);
    assert_eq!(b.roles.memberships.len(), 2);
    assert!(b
        .roles
        .memberships
        .values()
        .any(|id| *id > a.roles.maximum_kind.unwrap()));
    assert_eq!(b.roles.bootstrap, a.roles.bootstrap);
}

#[test]
fn the_first_aggregate_is_allocated_after_the_retained_bootstrap_role() {
    let a = lower(&schema(false, false, false), None);
    let b = lower(&schema(false, false, true), Some(&a.roles));
    let storage = b.roles.aggregate_storage.unwrap();
    assert!(storage.aggregate_kind > a.roles.maximum_kind.unwrap());
    assert!(storage.lineage_kind > a.roles.maximum_kind.unwrap());
    assert!(storage.aggregate_aspect > a.roles.maximum_aspect.unwrap());
    assert!(storage.lineage_aspect > a.roles.maximum_aspect.unwrap());
    assert_eq!(b.roles.bootstrap, a.roles.bootstrap);
}

#[test]
fn authored_aspects_refuse_retained_platform_identities_without_relabelling_them() {
    let a = lower(&schema(false, false, true), None);
    let old = a.roles.platform_aspects;
    for identity in old.into_iter().chain([
        a.roles.bootstrap.1,
        a.roles.aggregate_storage.unwrap().aggregate_aspect,
    ]) {
        let refused = PhysicalIdentityAllocation::for_identities(
            Some(&a.roles),
            &RelationalSchemaRegistry::new(),
            Some(identity),
            [identity].into_iter(),
        )
        .err()
        .expect("collision must refuse");
        assert_eq!(
            refused.kind(),
            super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
        );
    }
    assert_eq!(a.roles.platform_aspects, old);
}

#[test]
fn historical_identity_reservations_survive_a_removed_role() {
    let mut a = lower(&schema(false, false, false), None);
    let historical = value_layout::identity_reservation(KindId(1000), AspectIdentity(2000));
    a.roles
        .reserve_snapshot(&historical, &historical.authority_snapshot());
    let b = lower(&schema(false, false, false), Some(&a.roles));
    assert_eq!(b.roles.maximum_kind, a.roles.maximum_kind);
    assert_eq!(b.roles.maximum_aspect, a.roles.maximum_aspect);
    let c = lower(&schema(true, false, true), Some(&b.roles));
    assert!(c.roles.entities["Earlier"] > KindId(1000));
    assert!(c.roles.aggregate_storage.unwrap().aggregate_aspect > AspectIdentity(2000));
}

#[test]
fn exhausted_free_space_still_allows_retained_roles_but_refuses_new_roles() {
    let mut a = lower(&schema(false, false, true), None);
    let historical = value_layout::identity_reservation(KindId(u32::MAX), AspectIdentity(u64::MAX));
    a.roles
        .reserve_snapshot(&historical, &historical.authority_snapshot());
    let raw = RelationalSchemaRegistry::new();
    let mut allocation =
        PhysicalIdentityAllocation::for_identities(Some(&a.roles), &raw, None, std::iter::empty())
            .unwrap();
    assert_eq!(allocation.entity("Zulu").unwrap(), a.roles.entities["Zulu"]);
    assert_eq!(
        allocation.bootstrap(|| unreachable!()).unwrap(),
        a.roles.bootstrap
    );
    assert_eq!(
        allocation.entity("New").unwrap_err().subject(),
        "application schema exhausts Relational kind identity space"
    );
    assert_eq!(
        allocation
            .aggregate_storage(|| unreachable!(), true)
            .unwrap_err()
            .subject(),
        "application schema exhausts Relational aspect-contract identity space"
    );
}

#[test]
fn identical_entity_and_relation_names_retain_their_distinct_physical_classes() {
    let declaration = value_layout::same_named_classes();
    let a = lower(&declaration, None);
    let (entities, relations) = source_application::resolve(
        &declaration,
        &value_layout::contracts(),
        &a.registry,
        &a.registry.authority_snapshot(),
    )
    .unwrap();
    assert_eq!(entities["Zulu"], a.roles.entities["Zulu"]);
    assert_eq!(relations["Zulu"], a.roles.relations["Zulu"]);
    assert_ne!(entities["Zulu"], relations["Zulu"]);
}

#[test]
fn an_absent_original_application_role_refuses_before_allocation() {
    let a = lower(&schema(false, false, false), None);
    let refused = source_application::resolve(
        &schema(true, false, false),
        &value_layout::contracts(),
        &a.registry,
        &a.registry.authority_snapshot(),
    )
    .unwrap_err();
    assert_eq!(
        refused.kind(),
        super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::InvalidSchemaMember
    );
    assert_eq!(
        refused.subject(),
        "original application entity role is missing"
    );
}

#[test]
fn projected_aggregate_expansion_keeps_existing_storage_and_reserves_new_aspects() {
    let a = lower(&schema(false, false, true), None);
    let mut allocation = PhysicalIdentityAllocation::for_identities(
        Some(&a.roles),
        &RelationalSchemaRegistry::new(),
        None,
        std::iter::empty(),
    )
    .unwrap();
    let expanded = allocation
        .aggregate_storage(|| unreachable!(), true)
        .unwrap();
    let old = a.roles.aggregate_storage.unwrap();
    assert_eq!(expanded.aggregate_kind, old.aggregate_kind);
    assert_eq!(expanded.lineage_kind, old.lineage_kind);
    assert_eq!(expanded.aggregate_aspect, old.aggregate_aspect);
    assert_eq!(expanded.lineage_aspect, old.lineage_aspect);
    let projected = expanded.projected.unwrap();
    assert!(projected.coordinate_aspect > a.roles.maximum_aspect.unwrap());
    assert!(projected.membership_aspect > projected.coordinate_aspect);
    assert!(old.projected.is_none());
}

#[test]
fn new_roles_reserve_raw_receiver_kind_and_aspect_identities_together() {
    let a = lower(&schema(false, false, false), None);
    let receiver = value_layout::identity_reservation(KindId(9000), AspectIdentity(10000));
    let mut allocation = PhysicalIdentityAllocation::for_identities(
        Some(&a.roles),
        &receiver,
        None,
        std::iter::empty(),
    )
    .unwrap();
    assert!(allocation.entity("New").unwrap() > KindId(9000));
    let storage = allocation
        .aggregate_storage(|| unreachable!(), false)
        .unwrap();
    assert!(storage.aggregate_kind > KindId(9000));
    assert!(storage.lineage_kind > storage.aggregate_kind);
    assert!(storage.aggregate_aspect > AspectIdentity(10000));
    assert!(storage.lineage_aspect > storage.aggregate_aspect);
}

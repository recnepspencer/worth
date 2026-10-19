//! Observe the real primary payload and complete roster's native incidence.
use super::*;
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};

pub(super) fn observe<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    settled: &WorthQueryOutputDemandSettlement,
    roles: &[RequiredOutputRoleTarget],
) -> RelationId {
    let id_for = |name| {
        let mut matches = roles.iter().filter(|role| role.entity_name == name);
        let EntityReference::Existing(id) = matches.next().unwrap().target else {
            panic!("settled correspondence must contain actual native IDs")
        };
        assert!(matches.next().is_none(), "one exact member per entity kind");
        id
    };
    let present = id_for("PresentNode");
    let keeping = roles
        .iter()
        .all(|role| role.posture == RequiredOutputRolePosture::Preserve);
    let primary = if keeping {
        settled.output_correspondence.entity_for_binding_role::<Binding<Schema, true>>(
            <<Preserve<Schema> as WorthQueryApplicationProducerBinding<Schema>>::OutputRole
                as WorthQueryApplicationOutputRole>::NAME,
        )
    } else {
        settled.output_correspondence.entity_for_binding_role::<Binding<Schema, false>>(
            <<Initial<Schema> as WorthQueryApplicationProducerBinding<Schema>>::OutputRole
                as WorthQueryApplicationOutputRole>::NAME,
        )
    }
    .unwrap();
    assert_eq!(
        primary, present,
        "the actual producer primary is the present member"
    );
    observe_roles(runtime, phase, primary, roles)
}
pub(super) fn observe_roles<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    primary: EntityId,
    roles: &[RequiredOutputRoleTarget],
) -> RelationId {
    let id_for = |name| {
        let mut found = roles.iter().filter(|role| role.entity_name == name);
        let EntityReference::Existing(id) = found.next().unwrap().target else {
            unreachable!()
        };
        assert!(found.next().is_none());
        id
    };
    let execution = phase
        .execution_request_for(&runtime.product_runtime)
        .unwrap();
    let selected = runtime
        .on_branch(runtime.current_world())
        .select(phase)
        .unwrap();
    let graph = &runtime.primary_provider.graph;
    let field = Value::reference::<Schema>();
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap();
    let aspect = locator.aspect().aspect_key();
    let key = &locator.field_path().fields()[0];
    graph.with_runtime(&execution, |native| {
        let view = native
            .read_truth()
            .project_snapshot(selected.application_basis().snapshot_handle())
            .unwrap();
        let value = view
            .entity_record_with_projection_scope(
                &execution,
                primary,
                ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                    aspect.clone(),
                    [key.clone()],
                )]),
                |record| {
                    assert_eq!(record.kind_name(), field.entity());
                    record.aspect_field_value(aspect, key).cloned()
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(
            value,
            StringApplicationValueBinding::encode(&"native-present".to_owned()).unwrap(),
            "readiness observes the real existing payload, never a wake-only write"
        );
        // Every role retains its exact native identity and kind, including the
        // wholly absent Optional member and the genuinely fieldless member.
        for role in roles {
            let EntityReference::Existing(id) = role.target else {
                unreachable!()
            };
            assert!(view
                .entity_record_with_projection_scope(
                    &execution,
                    id,
                    ProjectionAspectScope::empty(),
                    |record| {
                        assert_eq!(record.kind_id(), role.kind);
                        assert_eq!(record.kind_name(), role.entity_name);
                        Some(())
                    },
                )
                .unwrap()
                .is_some());
        }
        let node = id_for("OutputNode");
        let fieldless = id_for("FieldlessNode");
        let incidence = view
            .outgoing_relations_of_kind(
                &execution,
                node,
                graph.layout.relation("Incidence").unwrap().kind,
            )
            .unwrap();
        assert_eq!(
            incidence.len(),
            1,
            "the complete real incidence remains present"
        );
        assert_eq!(incidence[0].source, node);
        assert_eq!(incidence[0].target, fieldless);
        incidence[0].relation_id
    })
}

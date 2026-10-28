use worth_query_consumer_values::{PlanarOperation, PlanarVertex};
use worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestQueryDenial,
    },
    declaration::application_operation::WorthQueryCreateOutput,
    primary_graph::WorthQueryEntityResolutionDenialKind,
};
use worth_query_topology_entry::{
    PlanarAnchorOutput, PlanarCreatedOutputs, PlanarMutation, PlanarOutputs, PlanarRead,
};

use super::{
    length, mutate, read_y, require_planar_violation, source_version, ProgramApplication, Request,
};

pub(super) fn create_and_reject_cycles(request: &Request<'_>, application: &ProgramApplication) {
    declared_minimum_is_program_meaning();
    short_cycles_reach_the_platform(request, application);
    for (ordinal, coordinates) in [
        vec![(41, 1), (50, 1), (41, 10)],
        vec![(61, 1), (70, 1), (70, 10), (61, 10)],
        vec![
            (130, 100),
            (126, 115),
            (115, 126),
            (100, 130),
            (85, 126),
            (74, 115),
            (70, 100),
            (74, 85),
            (85, 74),
            (100, 70),
            (115, 74),
            (126, 85),
        ],
    ]
    .into_iter()
    .enumerate()
    {
        let vertices = cycle_vertices(&format!("created-{ordinal}"), &coordinates);
        let input = PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::CreateCycle(vertices.clone()),
        };
        let outcome = mutate(request, application, input, 100 + ordinal as u64);
        let WorthQueryApplicationMutationOutcome::Committed {
            mut receipt,
            result,
        } = outcome
        else {
            panic!("the complete positive-turn ring must publish: {outcome:?}")
        };
        assert_eq!(result.changed_vertices, vertices.len());
        let outputs = receipt
            .outputs_of::<PlanarOutputs>()
            .expect("the cycle commits under the planar contract");
        outputs
            .entity::<PlanarAnchorOutput<crate::ConsumerSchema>>()
            .expect("the committed group preserves its declared anchor through owner identity correspondence");
        let created = vertices
            .iter()
            .map(|vertex| {
                outputs
                    .member::<PlanarCreatedOutputs<crate::ConsumerSchema>, WorthQueryCreateOutput>(
                        &vertex.body_key,
                    )
                    .expect("every created cycle member is sealed under its source-derived role")
                    .entity_id()
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(created.len(), vertices.len());
        let work = receipt
            .mutation_work()
            .expect("the real primary mutation carries work evidence");
        assert!(work.invariant_work_units() > 0);
        assert_eq!(work.relational_invariant_execution_count(), 3);
        assert!(
            receipt.take_performed_relational_product_change().is_some(),
            "a real committed World publication must carry one consumable change"
        );
        assert!(
            receipt.take_performed_relational_product_change().is_none(),
            "one commit cannot manufacture a second performed change"
        );
        for vertex in &vertices {
            assert_eq!(
                read_y(request, &vertex.body_key),
                worth_query_consumer_values::PositiveLength::get(&vertex.y)
            );
        }

        // Same vertex/relation/field cardinality, opposite winding.
        let mut malformed = cycle_vertices(&format!("rejected-{ordinal}"), &coordinates);
        malformed.reverse();
        let before = source_version(request);
        require_planar_violation(mutate(
            request,
            application,
            PlanarMutation {
                scope_key: "anchor-a".to_owned(),
                operation: PlanarOperation::CreateCycle(malformed.clone()),
            },
            200 + ordinal as u64,
        ));
        assert_eq!(source_version(request), before);
        require_absent_vertices(request, malformed);
    }
}

fn cycle_vertices(prefix: &str, coordinates: &[(u64, u64)]) -> Vec<PlanarVertex> {
    coordinates
        .iter()
        .enumerate()
        .map(|(index, &(x, y))| PlanarVertex {
            body_key: format!("{prefix}-{index}"),
            x: length(x),
            y: length(y),
        })
        .collect()
}

fn declared_minimum_is_program_meaning() {
    use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
    use worth_query_topology_entry::{Body, PlanarSuccessor};
    let integrity = PlanarSuccessor::reference::<crate::ConsumerSchema>().integrity();
    let minimum = integrity
        .reaches_at_least
        .expect("the real ring declares its minimum");
    assert_eq!(minimum.minimum_reachable_targets, 2);
    assert_eq!(
        minimum.target_kinds,
        &[Body::reference::<crate::ConsumerSchema>().name()]
    );
    let bound = crate::application_program::validated_program().unwrap();
    let unbound =
        ApplicationProgramAuthoring::<crate::ConsumerSchema, crate::ConsumerProgram>::begin()
            .validated_program()
            .unwrap();
    assert_ne!(
        bound.revision(),
        unbound.revision(),
        "the installed program revision includes its actual schema's graph laws"
    );
    let declaration = crate::ConsumerSchema::declaration().unwrap();
    let explicit =
        ApplicationProgramAuthoring::<crate::ConsumerSchema, crate::ConsumerProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap();
    assert_eq!(bound.revision(), explicit.revision());
}

fn short_cycles_reach_the_platform(request: &Request<'_>, application: &ProgramApplication) {
    for count in [1, 2] {
        let vertices = cycle_vertices(&format!("short-{count}"), &[(71, 1), (80, 1)][..count]);
        let before = source_version(request);
        let outcome = mutate(
            request,
            application,
            PlanarMutation {
                scope_key: "anchor-a".to_owned(),
                operation: PlanarOperation::CreateCycle(vertices.clone()),
            },
            700 + count as u64,
        );
        if count == 1 {
            require_original_self_edge(outcome);
        } else {
            require_planar_violation(outcome);
        }
        assert_eq!(source_version(request), before);
        require_absent_vertices(request, vertices);
    }
}

fn require_absent_vertices(request: &Request<'_>, vertices: Vec<PlanarVertex>) {
    for vertex in vertices {
        let absent = request
            .query(PlanarRead {
                body_key: vertex.body_key,
            })
            .execute();
        match absent {
            Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial)) => {
                assert_eq!(
                    denial.kind(),
                    WorthQueryEntityResolutionDenialKind::Refused(ResolutionRefusal::UnknownEntity)
                );
            }
            Err(denial) => panic!("another denial cannot prove vertex absence: {denial:?}"),
            Ok(_) => panic!("a rejected candidate vertex must not be selectable"),
        }
    }
}

fn require_original_self_edge(outcome: super::MutationOutcome) {
    use worth_query_host::facade::{
        application_invariants::EntityReference,
        primary_graph::{
            WorthQueryApplicationCommitDenialKind as Kind,
            WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted,
            WorthQueryRelationLawDenial,
        },
    };
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the self edge must reach platform validation: {outcome:?}");
    };
    assert_eq!(denial.kind(), Kind::ProviderRejected);
    assert_eq!(denial.stage(), Stage::InvariantExecution);
    let Some(WorthQueryRelationLawDenial::SelfEdge {
        contract_id,
        source,
        target,
        self_edge,
        ..
    }) = denial.relation_law()
    else {
        panic!("the original endpoint cause must survive: {denial:?}");
    };
    assert!(self_edge);
    assert_eq!(contract_id.as_str(), "PlanarSuccessor:endpoints");
    assert_eq!(source, target);
    let EntityReference::Created(_) = source else {
        panic!("the unpublished endpoint must keep its creation identity: {source:?}");
    };
}

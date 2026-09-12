use worth_query_consumer_values::{PlanarOperation, PlanarVertex};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestQueryDenial,
    },
    primary_graph::{
        WorthQueryApplicationOutputRole, WorthQueryEntityResolutionDenialKind,
        WorthQueryPreserveOutput,
    },
};
use worth_query_topology_entry::{Body, PlanarMutation, PlanarMutationBinding, PlanarRead};

use super::{length, mutate, read_y, require_planar_violation, source_version, Request};

pub(super) fn create_and_reject_cycles(request: &Request<'_>) {
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
            validator_work: 4096,
        };
        let outcome = mutate(request, input, 100 + ordinal as u64);
        let WorthQueryApplicationMutationOutcome::Committed {
            mut receipt,
            result,
        } = outcome
        else {
            panic!("the complete positive-turn ring must publish: {outcome:?}")
        };
        assert_eq!(result.changed_vertices, vertices.len());
        receipt.output_correspondence().entity(
            WorthQueryApplicationOutputRole::<PlanarMutationBinding<crate::ConsumerSchema>, Body, WorthQueryPreserveOutput>::new("anchor"),
        ).expect("the committed group preserves its declared anchor through owner identity correspondence");
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
            PlanarMutation {
                scope_key: "anchor-a".to_owned(),
                operation: PlanarOperation::CreateCycle(malformed.clone()),
                validator_work: 4096,
            },
            200 + ordinal as u64,
        ));
        assert_eq!(source_version(request), before);
        for vertex in malformed {
            let absent = request
                .query(PlanarRead {
                    body_key: vertex.body_key,
                })
                .execute();
            match absent {
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial)) => {
                    assert_eq!(
                        denial.kind(),
                        WorthQueryEntityResolutionDenialKind::UnknownEntity
                    )
                }
                Err(denial) => panic!("another denial cannot prove vertex absence: {denial:?}"),
                Ok(_) => panic!("a rejected candidate vertex must not be selectable"),
            }
        }
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

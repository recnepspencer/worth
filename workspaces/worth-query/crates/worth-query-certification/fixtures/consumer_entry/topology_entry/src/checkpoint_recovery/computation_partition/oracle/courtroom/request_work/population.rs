//! Identical Native truth; only registry population differs between lanes.
use super::*;
pub(super) type ScaleProgram = OracleProgram<false, TOTALS_WORK, 1, 4>;
type ScaleApplication =
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, ScaleProgram>;
fn profile() -> worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile {
    worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
        .with_registry_obligation_retained_bytes(std::num::NonZeroUsize::new(1 << 40).unwrap())
        .with_lineage_retained_bytes(std::num::NonZeroUsize::new(1 << 40).unwrap())
        .with_registry_record_retained_bytes(std::num::NonZeroUsize::new(1 << 40).unwrap())
        .with_registry_required_retained_bytes(std::num::NonZeroUsize::new(1 << 40).unwrap())
}
pub(super) fn install_population() -> ScaleApplication {
    author_population()
}

fn author_population() -> ScaleApplication {
    let model = Model::new(&mut Lcg(SEEDS[0]));
    let app = installation::install_variant_with_observations::<false, TOTALS_WORK, 1, 4>(
        None,
        profile(),
        1_000,
        |graph| model.seed(graph),
    );
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    // Each public edit creates one valid triangle within its declared
    // invariant budget. Both lanes restore the same publicly authored truth.
    for triangle in 0..1_008_usize.div_ceil(3) {
        use worth_query_consumer_values::{PlanarOperation, PlanarVertex};
        let source = request
            .query(PlanarRead {
                body_key: SCOPE.to_owned(),
            })
            .execute()
            .unwrap();
        let vertices = [(1, 1), (10, 1), (1, 10)]
            .into_iter()
            .enumerate()
            .map(|(corner, (x, y))| PlanarVertex {
                body_key: format!("unrelated-{}", triangle * 3 + corner),
                x: length(x),
                y: length(y),
            })
            .collect();
        let outcome = request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: SCOPE.to_owned(),
                operation: PlanarOperation::CreateCycle(vertices),
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&(0x612_6000 + triangle as u64))
            .execute_in_program::<ScaleProgram>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    for number in 0..1_008_u64 {
        let name = format!("unrelated-{number}");
        let source = request
            .query(PlanarRead {
                body_key: name.clone(),
            })
            .execute()
            .unwrap();
        let value = worth_query_consumer_values::PositiveLength::get(&source.rows()[0].y) + 1;
        let committed = request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: name.clone(),
                operation: worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(
                    worth_query_consumer_values::PlanarDerivedOutput {
                        body_key: name,
                        value: length(value),
                    },
                ),
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&(0x612_6800 + number))
            .execute_in_program::<ScaleProgram>(
                &app,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    app
}

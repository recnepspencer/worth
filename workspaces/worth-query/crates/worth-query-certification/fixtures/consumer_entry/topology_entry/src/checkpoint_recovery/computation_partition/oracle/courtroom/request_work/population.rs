//! Identical Native truth; only registry population differs between lanes.
use super::*;
pub(super) type ScaleProgram = OracleProgram<false, TOTALS_WORK, 1, 4>;
type ScaleApplication =
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, ScaleProgram>;
#[cfg(target_pointer_width = "64")]
const REGISTRY_OBLIGATION_RETAINED_BYTES: usize = 1 << 40;
#[cfg(target_pointer_width = "32")]
const REGISTRY_OBLIGATION_RETAINED_BYTES: usize = usize::MAX;
#[cfg(target_pointer_width = "64")]
const LINEAGE_RETAINED_BYTES: usize = 1 << 40;
#[cfg(target_pointer_width = "32")]
const LINEAGE_RETAINED_BYTES: usize = usize::MAX;
#[cfg(target_pointer_width = "64")]
const REGISTRY_RECORD_RETAINED_BYTES: usize = 1 << 40;
#[cfg(target_pointer_width = "32")]
const REGISTRY_RECORD_RETAINED_BYTES: usize = usize::MAX;
#[cfg(target_pointer_width = "64")]
const REGISTRY_REQUIRED_RETAINED_BYTES: usize = 1 << 40;
#[cfg(target_pointer_width = "32")]
const REGISTRY_REQUIRED_RETAINED_BYTES: usize = usize::MAX;

fn profile() -> worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile {
    worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
        .with_registry_obligation_retained_bytes(
            std::num::NonZeroUsize::new(REGISTRY_OBLIGATION_RETAINED_BYTES).unwrap(),
        )
        .with_lineage_retained_bytes(std::num::NonZeroUsize::new(LINEAGE_RETAINED_BYTES).unwrap())
        .with_registry_record_retained_bytes(
            std::num::NonZeroUsize::new(REGISTRY_RECORD_RETAINED_BYTES).unwrap(),
        )
        .with_registry_required_retained_bytes(
            std::num::NonZeroUsize::new(REGISTRY_REQUIRED_RETAINED_BYTES).unwrap(),
        )
}
pub(super) fn install_population() -> ScaleApplication {
    author_population(1_008)
}

pub(super) fn install_settled_population() -> ScaleApplication {
    // Twenty-five cycle edits and seventy-five publications prime one hundred
    // real admissions. Both measured lanes then render three-digit operation
    // identities, so label-width charges cannot counterfeit population work.
    author_population(75)
}

fn author_population(vertices: usize) -> ScaleApplication {
    let model = Model::new(&mut Lcg(SEEDS[0]));
    let app = installation::install_variant_with_observations::<false, TOTALS_WORK, 1, 4>(
        None,
        profile(),
        1_000,
        |graph| model.seed(graph),
    );
    // Each public edit creates one valid triangle within its declared
    // invariant budget. Both lanes restore the same publicly authored truth.
    for triangle in 0..vertices.div_ceil(3) {
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
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
    for number in 0..vertices as u64 {
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
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

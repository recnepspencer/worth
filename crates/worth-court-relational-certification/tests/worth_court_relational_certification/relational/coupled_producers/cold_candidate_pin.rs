//! Native applied pins retain the dictionary owner that admitted their root.
use super::*;
use std::cell::Cell;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::runtime::ProjectionAspectScope;

struct ProducerRead<'a> {
    world: &'a World,
    reads: Cell<usize>,
}
impl RelationalAuthoredCoupledProducerAssembly for ProducerRead<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        let read = candidate.pin_source(execution)?;
        let value = name(&read.project_candidate(), execution, self.world.source);
        assert_eq!(value, "edited-source");
        self.reads.set(self.reads.get() + 1);
        Ok(ControlFlow::Continue(()))
    }
}
fn name(
    view: &worth_relational::facade::runtime::VisibilityProjectionView,
    execution: &ExecutionRequest<'_, '_>,
    id: EntityId,
) -> String {
    view.entity_record_with_projection_scope(
        execution,
        id,
        ProjectionAspectScope::whole_aspects([aspect_key("name")]),
        |record| match record.aspect_value(&aspect_key("name"))? {
            AspectValue::String(InternedString::Raw(text)) => Some(text.clone()),
            _ => panic!("the real name contract retains a plain string"),
        },
    )
    .expect("valid pin keeps its admitted symbol predecessor")
    .expect("the named row exists")
}
#[test]
fn cold_applied_candidate_pin_retains_admitted_symbol_predecessor() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, false, false);
    let cold = world.runtime.place_symbol_pages_cold_for_test(execution);
    let before = world.basis();
    let view = world
        .runtime
        .read_truth()
        .project_observation(&before.observation())
        .unwrap();
    assert_eq!(
        name(&view, execution, world.source),
        "source",
        "initial C admission succeeds"
    );
    let producer = ProducerRead {
        world: &world,
        reads: Cell::new(0),
    };
    let proposal = world
        .prepare(execution, &producer)
        .expect("the producer's D read succeeds");
    assert!(producer.reads.get() > 0, "the genuine callback read D");
    // Placement installs the genuine cold directory; this scalar-row fixture
    // need not fault another ID page after its initial resident C admission.
    eprintln!(
        "Native cold placement page reads: {}",
        cold.page_reads_for_test()
    );
    assert_eq!(world.basis(), before, "the candidate remains unpublished");
    assert_eq!(
        name(&view, execution, world.source),
        "source",
        "C stays pinned"
    );
    drop(proposal);
}

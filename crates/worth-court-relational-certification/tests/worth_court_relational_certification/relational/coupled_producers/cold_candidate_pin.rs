//! Native applied pins retain the dictionary owner that admitted their root.
use super::*;
use std::cell::Cell;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::runtime::ProjectionAspectScope;

struct ProducerRead<'a> {
    world: &'a World,
    reads: Cell<usize>,
    second_revision: bool,
}
impl RelationalAuthoredCoupledProducerAssembly for ProducerRead<'_> {
    fn assemble(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<ControlFlow<()>, RelationalCoupledAssemblyFailure> {
        // Capture D without an incidental head read re-admitting its lost
        // owned mark. The real producer consumes that pin as its predecessor.
        let read = candidate.pin_source(execution)?;
        assert!(assembly(self.world)
            .assemble(execution, candidate)?
            .is_continue());
        let value = name(&read.project_candidate(), execution, self.world.source);
        assert_eq!(value, "edited-source");
        self.reads.set(self.reads.get() + 1);
        if self.second_revision {
            candidate.append_publication(
                execution,
                RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                    admission: self.world.owner.admit_required_output("second", [3; 32]),
                    absence: Some(self.world.absence(execution, "second")),
                    meaning: self.world.meaning("second"),
                    roles: vec![created("second-output")],
                }),
                &RelationalContentAttachments::default(),
                vec![create("second-output")],
            )?;
        }
        let next = candidate.pin_source(execution)?;
        assert_eq!(
            name(&next.project_candidate(), execution, self.world.carrier),
            "calculated-carrier"
        );
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
    exercise(false);
}
#[test]
fn cold_later_applied_revision_carries_previous_dictionary_admission() {
    exercise(true);
}
fn exercise(second_revision: bool) {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, false);
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
        second_revision,
    };
    let owners = world
        .runtime
        .readmit_edit_coupled_owners(execution, &world.installed, &before, world.source)
        .unwrap();
    let mut tx = begin_main_transaction(&world.runtime);
    // A real graph edit selects Native's sparse graph overlay. A flat scalar
    // copy retains enough old rows to reconstruct a lost mark and masks the bug.
    tx.push_batch(
        update(world.source, "edited-source").push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: ClientKey::raw("prefix-edge"),
                source: EntityReference::Existing(world.source),
                target: EntityReference::Existing(world.carrier),
                fields: string_aspect_field_patch([(
                    aspect_key("label"),
                    field_key("label"),
                    "prefix-edge",
                )]),
            },
        ))),
        execution.allocation_policy(),
    )
    .unwrap();
    let tx = tx
        .bind_authored_coupled_producers(&world.installed, owners, None)
        .unwrap();
    let proposal = world
        .runtime
        .preparation_port()
        .prepare_authored_coupled_transaction(execution, tx, &producer)
        .expect("the producer's sparse graph D read succeeds")
        .expect("the producer read completes");
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

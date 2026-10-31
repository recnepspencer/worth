//! A current Preserve supplier does not become the creator of an unchanged field.
use super::*;
use primary_graph::{
    admitted_capture::run_retained_stage, application_output_demand::OriginalSettledProducerSource,
};
use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};
use worth_relational::facade::{
    publication::PublishedAuthoritativeFieldEffect as Effect, runtime::RelationalFieldRevision,
};

pub(in super::super) fn created<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> (EntityId, RelationalFieldRevision) {
    let (entity, revision, effect) = observe(runtime, phase, receipt);
    assert_eq!(
        effect,
        Effect::Present,
        "the original actually wrote the field"
    );
    assert_eq!(revision.version(), receipt.commit_reference().version_id);
    (entity, revision)
}

pub(in super::super) fn preserved<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    settled: &WorthQueryOutputDemandSettlement,
    original: (EntityId, RelationalFieldRevision),
) {
    let receipt = settled.application_commit_receipt().unwrap();
    let (entity, revision, effect) = observe(runtime, phase, receipt);
    assert_eq!(
        entity, original.0,
        "Preserve keeps the full original generation"
    );
    assert_eq!(
        revision, original.1,
        "unchanged output keeps its Native origin"
    );
    assert_eq!(
        effect,
        Effect::Unchanged,
        "currentness does not author a field"
    );
    assert_ne!(revision.version(), receipt.commit_reference().version_id);

    let readers = runtime.application_query_basis_observer();
    let baseline = readers.observe().active();
    let read = settled.retained_read();
    let selected = runtime
        .select_application_read_observation(phase, &read)
        .unwrap();
    let supplier = selected.original_settled_producer().unwrap();
    let OriginalSettledProducerSource::Committed {
        changes,
        original_publication,
        ..
    } = supplier.source()
    else {
        panic!("the executed Preserve retains its real committed supplier")
    };
    assert!(
        changes.shares_original(receipt.committed_changes()),
        "retention must not substitute the field creator for the current supplier"
    );
    assert_ne!(changes.commit_reference().version_id, revision.version());
    let producing = original_publication
        .as_ref()
        .expect("the original own-publication continuation retains its actual producing evidence");
    assert_eq!(
        producing.changes.commit_reference().version_id,
        original.1.version()
    );
    assert!(!producing.changes.shares_original(changes));
    assert!(producing
        .correspondence
        .native_witness_roles()
        .any(|(_, _, _, output)| output == original.0));
    drop(selected);
    drop(read);
    assert_eq!(readers.observe().active(), baseline);
}

fn observe<Schema: BirthSchema>(
    runtime: &installation::Application<Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> (EntityId, RelationalFieldRevision, Effect) {
    let field = Value::reference::<Schema>();
    let mut roles = receipt
        .output_correspondence()
        .native_witness_roles()
        .filter(|(_, _, entity, _)| *entity == field.entity());
    let (_, _, _, entity) = roles.next().unwrap();
    assert!(
        roles.next().is_none(),
        "one actual output of this installed kind"
    );
    let graph = &runtime.primary_provider.graph;
    let locator = graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap();
    let execution = phase
        .execution_request_for(&runtime.product_runtime)
        .unwrap();
    assert_eq!(locator.aspect().authority(), LocatorAuthority::Planned);
    let (effect, native_locator) = run_retained_stage(&execution, &mut None, |context, _| {
        let coordinate = graph
            .layout
            .resolve_declared_field_coordinate(
                runtime.installed_schema().native_contracts(),
                field.entity(),
                locator.aspect().aspect_key(),
                &locator.field_path().fields()[0],
                |work| primary_graph::admitted_capture::checkpoint(context, work),
            )
            .map_err(worth_execution::MapKernelFailure::Stop)?
            .unwrap();
        let effect = receipt
            .committed_changes()
            .field_effect(context, entity, coordinate)
            .map_err(worth_execution::MapKernelFailure::<()>::Stop)?;
        let native_locator = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            coordinate.native().contract().key().clone(),
            CanonicalFieldPath::single(coordinate.field().clone()),
        );
        Ok((effect, native_locator))
    })
    .unwrap_or_else(|_| panic!("actual original canonical effect must be available"));
    let selected = runtime
        .on_branch(receipt.product_branch())
        .select(phase)
        .unwrap();
    assert!(
        receipt
            .committed_product_publication()
            .is_selected_at(selected.product().observation()),
        "observe at the actual publication, never a later substituted head"
    );
    let revision = graph.with_runtime(&execution, |native| {
        native
            .read_truth()
            .project_snapshot(selected.application_basis().snapshot_handle())
            .unwrap()
            .entity_field_revision(&execution, entity, &native_locator)
            .unwrap()
            .expect("the installed published field has genuine Native provenance")
    });
    assert_eq!(
        revision.presence(),
        worth_relational::facade::runtime::RelationalFieldPresence::Present
    );
    (entity, revision, effect)
}

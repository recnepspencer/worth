//! One mixed original through World's definition path: a Native write and
//! the next generation of the court's authored Bridge definition.
use super::super::artifact_source::{bridge_owner, Fixture};
use super::*;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_runtime_bridge::facade::BridgePreparedAuthoredDefinitionPublication;

pub(in super::super) struct MixedOriginal {
    pub prepared: WorldPreparedPublicationWithSignal,
    pub definition: BridgePreparedAuthoredDefinitionPublication,
    /// Attempt, reserved commit and custody ordinals of the durable open.
    pub identities: [u64; 3],
}

/// Open, bind Native's validated write and prepare the Bridge successor, as
/// Query does after the open commits. No owner effect has happened yet.
pub(in super::super) fn prepare(
    fixture: &Fixture,
    request: &ExecutionRequest<'_, '_>,
    head: &ProductBranchObservation,
    label: &str,
) -> MixedOriginal {
    let port = fixture.world.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let opened = match port
        .prepare_query_publication_with_signal(
            request,
            head.clone(),
            fixture.recorder.program_source(head),
            Some(RelationalTransactionIntent::ordinary()),
            &cancellation.token(),
            None,
        )
        .unwrap()
        .advance(request)
    {
        WorldOpenAttemptOutcome::Opened(opened) => opened,
        other => panic!("actual mixed open: {other:?}"),
    };
    let mut transaction =
        worth_testkit_relational::aspect_storage::publications::begin_main_transaction(
            &fixture.native,
        );
    transaction
        .push_batch(
            worth_testkit_relational::durable_publication::batch_create(label),
            request.allocation_policy(),
        )
        .unwrap();
    let proposal = transaction.validate(request, &fixture.native).unwrap();
    let identities = [
        opened.attempt_identity().ordinal(),
        opened.reserved_commit_identity().ordinal(),
        opened.unpublished_recovery_handle().identity().ordinal(),
    ];
    let prepared = opened
        .prepare_signal(&fixture.native, request, Some(proposal))
        .unwrap();
    let assembly = fixture.bridge();
    let predecessor = bridge_owner::installed(assembly);
    let generation = generation(&predecessor) + 1;
    let branch = DurableSignalBranchIdentity::from_recorded(
        assembly.signal_owner().binding().owner(),
        0,
    );
    let definition = assembly
        .prepare_conditional_definition_publication(
            request,
            branch,
            &predecessor,
            bridge_owner::installation(generation),
        )
        .unwrap();
    MixedOriginal {
        prepared,
        definition,
        identities,
    }
}

/// Execute the same original: World records the planned Signal leg, then
/// Native's effect, Signal's effect and the occurrence.
pub(in super::super) fn publish(
    port: &RuntimeWorldPublicationPort<(), (), (), (), ()>,
    request: &ExecutionRequest<'_, '_>,
    original: MixedOriginal,
    cancellation: &RuntimeWorldCancellationToken,
) -> WorldDurablePublicationOutcome {
    port.execute_durable_authored_conditional_definition(
        *request,
        original.prepared,
        original.definition,
        cancellation,
    )
}

/// The authored generation the Bridge owner currently serves.
pub(in super::super) fn served_generation(fixture: &Fixture) -> u64 {
    generation(&bridge_owner::installed(fixture.bridge()))
}

fn generation(lowering: &worth_runtime_bridge::facade::BridgeInstalledConditionalLowering) -> u64 {
    lowering.signal_definition_generation()
}

//! A generated output restored into its branch is reused, preserved or
//! refused. The producer that created it never runs over it a second time.

use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_entry::WorthQueryOutputSettlementPosture as Posture;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::primary_graph::{
    WorthQueryGeneratedEntity, WorthQueryReconstructedOutputEntity,
};

#[cfg(feature = "test-output-delivery-faults")]
mod native_writer;

const ROOT: &str = "anchor-a";

type Request<'application, 'principal, 'scope> =
    worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >;
type Final = PlanarFinalOutputProducer<CheckpointSchema>;

/// One vertex of the generated ring: its key, abscissa, ordinate and length.
type Vertex = (String, PositiveLength, PositiveLength, PositiveLength);

#[test]
fn an_output_restored_on_the_runtime_that_performed_it_is_reused() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let ring = ring(&request);

    suspend_and_restore(&application, &request, &scope, &ring);

    // The restoration continued the performed record: nothing is contacted.
    assert_eq!(
        redemanded(&request, &application),
        (Posture::StableReused, 0)
    );
    assert_eq!(self::ring(&request), ring);
}

#[test]
fn an_output_restored_after_a_checkpoint_is_never_created_a_second_time() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the ready outputs are checkpointable");
    drop(application);

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let ring = ring(&request);

    // The restored row holds no performed proof, so the restoration has no
    // performed record to continue.
    suspend_and_restore(&restored, &request, &scope, &ring);

    assert_preserved_once(&request, &restored, &ring);
}

#[test]
fn an_output_readmitted_after_a_checkpoint_then_restored_is_never_created_a_second_time() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the ready outputs are checkpointable");
    drop(application);

    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let ring = ring(&request);
    assert_eq!(
        redemanded(&request, &restored),
        (Posture::RecoveredPerformed, 0)
    );

    // The restored row holds no performed proof, so the restoration has no
    // performed record to continue.
    suspend_and_restore(&restored, &request, &scope, &ring);

    assert_preserved_once(&request, &restored, &ring);
}

/// Recording a restoration is an invalidation edit, bounded by the installed
/// invalidation work. Under this allowance the first demand performs, and the
/// restored row is recorded with no settlement registered for it.
const RECORDING_STOPS: u64 = 1_024;

#[test]
fn an_output_whose_restoration_could_not_be_recorded_is_preserved_not_created_again() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program_with_seed::<CheckpointProgram>(
        None,
        Default::default(),
        32,
        128 * 1_024 * 1_024,
        RECORDING_STOPS,
        support::seed_cycle,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    let ring = ring(&request);

    suspend_and_restore(&application, &request, &scope, &ring);

    // The restored row has no settlement to cut off against, so its creating
    // producer is not selected for it.
    assert_preserved_once(&request, &application, &ring);
}

/// Demands the program's outputs and reports how its final output settled.
/// A second creation over the live ring would leave two bodies under one key,
/// which `ring` refuses to read.
fn redemanded(
    request: &Request<'_, '_, '_>,
    application: &support::Application,
) -> (Posture, usize) {
    let settlement = settle(request, application);
    let mut finals = settlement.outputs_for::<CheckpointSchema, PlanarOutputToFinalConnection>();
    let (_, output) = finals.next().expect("the program settles its final output");
    assert!(finals.next().is_none());
    (output.posture(), output.producer_contacts_in_this_demand())
}

/// One execution under the Preserve posture, then reuse. The creating
/// producer over the live ring would leave two bodies under each key.
fn assert_preserved_once(
    request: &Request<'_, '_, '_>,
    application: &support::Application,
    ring: &[Vertex],
) {
    assert_eq!(redemanded(request, application), (Posture::Performed, 1));
    assert_eq!(self::ring(request), ring);
    assert_eq!(redemanded(request, application), (Posture::Performed, 0));
    assert_eq!(self::ring(request), ring);
}

fn ring(request: &Request<'_, '_, '_>) -> Vec<Vertex> {
    ["", ":b", ":c"]
        .into_iter()
        .zip([1, 2, 1])
        .map(|(suffix, x)| {
            let key = format!("final:{ROOT}{suffix}");
            let read = request
                .query(PlanarRead {
                    body_key: key.clone(),
                })
                .execute()
                .unwrap_or_else(|denial| panic!("{key} is readable: {denial:?}"));
            assert_eq!(read.rows().len(), 1, "exactly one body is keyed {key}");
            let output = request
                .query(PlanarOutputRead {
                    body_key: key.clone(),
                })
                .execute()
                .unwrap_or_else(|denial| panic!("{key} reads as an output: {denial:?}"));
            assert_eq!(output.rows().len(), 1, "exactly one output is keyed {key}");
            (key, length(x), read.rows()[0].y, output.rows()[0].value)
        })
        .collect()
}

fn suspend_and_restore(
    application: &support::Application,
    request: &Request<'_, '_, '_>,
    scope: &authentication::WorthQueryRequestScope,
    ring: &[Vertex],
) {
    let source = request
        .query(PlanarRead {
            body_key: ROOT.to_owned(),
        })
        .execute()
        .expect("the producer source is readable")
        .observed_sources()[0]
        .clone();
    let suspended = application
        .on_branch(application.current_world())
        .select()
        .expect("the current branch is selectable")
        .suspend_current_generated_output::<Final>(scope, source)
        .unwrap_or_else(|_| panic!("the generated final output suspends"));
    let completed = reconstruct_ring(application, suspended, ring);
    application
        .restore_generated_output(completed, scope)
        .unwrap_or_else(|_| panic!("the reconstructed output is restored"));
}

fn generated<Denial: std::fmt::Debug>(
    entity: Result<WorthQueryReconstructedOutputEntity<CheckpointSchema, Body>, Denial>,
) -> WorthQueryGeneratedEntity<CheckpointSchema, Body> {
    match entity.expect("the typed role resolves its retained identity") {
        WorthQueryReconstructedOutputEntity::Generated(entity) => entity,
        WorthQueryReconstructedOutputEntity::Retained(_) => {
            panic!("a created ring vertex is in generated custody")
        }
    }
}

fn reconstruct_ring(
    application: &support::Application,
    suspended: worth_query_host::facade::primary_graph::WorthQuerySuspendedGeneratedOutput,
    ring: &[Vertex],
) -> worth_query_host::facade::primary_graph::WorthQueryCompletedGeneratedOutputReconstruction<
    CheckpointSchema,
    Final,
> {
    let mut reconstruction = application
        .reconstruct_generated_output::<Final>(suspended)
        .unwrap_or_else(|_| panic!("the creating producer reconstructs its output"));
    let mut entities = Vec::with_capacity(ring.len());
    for (index, (key, x, y, length)) in ring.iter().enumerate() {
        let entity = generated(match index {
            0 => reconstruction.output::<FinalAnchorOutput<CheckpointSchema>>(),
            1 => reconstruction.output_member::<FinalCreatedOutputs<CheckpointSchema>>(key),
            _ => reconstruction
                .output::<FinalClosingOutput<CheckpointSchema>>()
                .map(|closing| closing.expect("the suspended output bound its closing vertex")),
        });
        reconstruction
            .field(&entity, BodyKey::reference(), key.clone())
            .unwrap();
        reconstruction
            .field(&entity, PositionX::reference(), *x)
            .unwrap();
        reconstruction
            .field(&entity, PositionY::reference(), *y)
            .unwrap();
        reconstruction
            .field(&entity, Length::reference(), *length)
            .unwrap();
        entities.push(entity);
    }
    for index in 0..entities.len() {
        reconstruction
            .relation(
                PlanarSuccessor::reference(),
                &entities[index],
                &entities[(index + 1) % entities.len()],
            )
            .expect("custody retains the exact relation endpoints");
    }
    reconstruction
        .finish()
        .unwrap_or_else(|_| panic!("every suspended record is reconstructed once"))
}

#[cfg(feature = "test-query-execution-observer")]
mod advancement_custody;

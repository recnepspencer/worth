//! Shared media-sweep target/outcome types, outcome classifiers, and
//! allowlisted test-boundary Store helpers.

use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::transactions::{CommitResult, TransactionCommitError};
use worth_store_application::{ApplicationHomeCheckpointDenial, ReadmittedApplicationHome};

use crate::crash_oracle::InFlight;
use crate::media_faults::FaultedMedia;

/// The label of the create that probes an unresolved runtime.
pub(crate) const LATER_CREATE: usize = 2_000_000;

/// The step a sweep interrupts, by its index in the script.
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Commit(usize),
    BranchTransition(usize),
    Checkpoint(usize),
    RegionLoad(usize),
}

impl Target {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Commit(index)
            | Self::BranchTransition(index)
            | Self::Checkpoint(index)
            | Self::RegionLoad(index) => index,
        }
    }
}

/// How the faulted target ended.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Ended {
    /// The schedule refused the directive for its role; nothing ran.
    Skipped,
    Acknowledged,
    Refused,
    Unresolved {
        recovered: bool,
    },
}

/// Open a closed World-composed home with an admitted certification media fault
/// schedule so checkpoint sweeps execute through the genuine World owner.
pub(crate) fn open_world_with_media(
    location: &std::path::Path,
    schema: worth_relational::facade::schema::SchemaVersionId,
    media: FaultedMedia,
    execution: &ExecutionRequest<'_, '_>,
) -> (
    worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture,
    Vec<worth_store::physical_runtime::certification::CertificationMediaFaultActivation>,
) {
    let request = worth_store_testkit_application_oracle::facade::home_opening::request(
        location,
        crate::workload::runtime_builder(schema),
    )
    .with_media_for_test(media.admission);
    let fixture =
        worth_store_testkit_application_oracle::facade::world_home::open(request, execution)
            .unwrap_or_else(|refusal| panic!("a closed World home opens with faults: {refusal:?}"));
    (fixture, media.activations)
}

/// Content-verified physical recovery & checkpoint member digest captured after
/// readmission completes. Returns `None` if Store requires inspection or if
/// reading `home.checkpoint` or any member payload from disk fails.
pub(crate) fn verified_recovered_physical(
    home: &ReadmittedApplicationHome,
) -> Option<crate::media_sweep::equivalence::VerifiedPhysicalState> {
    crate::media_sweep::equivalence::verify_serving_physical(home.physical_serving_for_test())
}

#[cfg(test)]
pub(crate) fn with_serving_for_test<R>(
    home: &ReadmittedApplicationHome,
    f: impl FnOnce(&worth_store::physical_runtime::ServingPhysicalRuntime) -> R,
) -> R {
    f(home.physical_serving_for_test())
}

pub(crate) fn world_checkpoint_generation(
    fixture: &worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture,
) -> u64 {
    crate::selected_checkpoint::generation_from_serving(fixture.home.physical_serving_for_test())
}

/// Drain Store's unpinned clean physical frames on a live World home so the
/// armed checkpoint target runs against a cold physical frame cache.
pub(crate) fn drain_world_clean_frames(
    fixture: &worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture,
) {
    fixture
        .home
        .physical_serving_for_test()
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
}

#[cfg(test)]
pub(crate) fn first_checkpoint_member_bytes(home: &ReadmittedApplicationHome) -> Vec<u8> {
    let records = home.physical_serving_for_test().anchored_records().unwrap();
    let anchor =
        worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR)
            .unwrap();
    let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
    records
        .read_member(anchor, selected.members()[0], 1 << 30)
        .unwrap()
}

/// Classify a faulted commit and require the runtime to agree with it: an
/// acknowledged commit is installed, a refused or unresolved one is not, and
/// an unresolved fate refuses every later commit the same way.
pub(crate) fn commit_outcome(
    home: &ReadmittedApplicationHome,
    before: Vec<Vec<u8>>,
    outcome: Result<CommitResult, TransactionCommitError>,
    execution: &ExecutionRequest<'_, '_>,
    case: &str,
) -> (Ended, Vec<Vec<u8>>, InFlight) {
    let now =
        crate::workload::append_live_artifacts(execution, home.relational().runtime(), &before);
    match outcome {
        Ok(_) => {
            assert_eq!(
                now.len(),
                before.len() + 1,
                "{case}: acknowledged, not installed"
            );
            (crate::media_case::Ended::Acknowledged, now, crate::crash_oracle::InFlight::Nothing)
        }
        Err(worth_relational::facade::transactions::TransactionCommitError::PublicationDenied {
            denial: worth_relational::facade::mvcc::RelationalPublicationDenial::DurablePublicationRefused(_),
            ..
        }) => {
            assert!(now == before, "{case}: a refused commit was installed");
            (crate::media_case::Ended::Refused, before, crate::crash_oracle::InFlight::Refused)
        }
        Err(worth_relational::facade::transactions::TransactionCommitError::Preparation { error, .. })
            if matches!(
                error.reason(),
                worth_relational::facade::transactions::CommitPreparationReason::StorageUnavailable(
                    _
                )
            ) =>
        {
            // A media fault can now refuse a declared cold demand before the
            // durable step. Recovery must prove there was no journal append.
            assert!(
                now == before,
                "{case}: unavailable cold storage published a commit"
            );
            (crate::media_case::Ended::Refused, before, crate::crash_oracle::InFlight::NotDurable)
        }
        Err(error) => {
            let fate = error
                .durability_unresolved_fate()
                .unwrap_or_else(|| panic!("{case}: unexpected commit error {error:?}"))
                .clone();
            assert!(now == before, "{case}: an unresolved commit was installed");
            assert_eq!(
                crate::crash_oracle::commit_subject(&fate).0 .0,
                before.len() as u64 + 1,
                "{case}"
            );
            // A later commit that would otherwise publish is refused with the
            // same unresolved fate.
            let later = crate::workload::commit(
                home.relational().runtime(),
                execution,
                &BranchId("main".to_owned()),
                crate::batches::create(LATER_CREATE),
            );
            let later_fate = later
                .as_ref()
                .err()
                .and_then(|error| error.durability_unresolved_fate());
            assert!(
                later_fate == Some(&fate),
                "{case}: a later commit was not refused as unresolved: {:?}",
                later.map(|_| ())
            );
            (
                crate::media_case::Ended::Unresolved { recovered: false },
                before,
                crate::crash_oracle::InFlight::Unresolved(fate),
            )
        }
    }
}

/// Classify a faulted checkpoint and the generations recovery may select:
/// the new one once its record is published, the old one when the
/// publication was proven to have had no effect, otherwise either.
pub(crate) fn checkpoint_outcome(
    outcome: Result<
        worth_store_application::ApplicationHomeCheckpoint,
        ApplicationHomeCheckpointDenial,
    >,
    old: u64,
    case: &str,
) -> (Ended, Vec<u64>) {
    let new = old + 1;
    match outcome {
        Ok(checkpoint) => {
            assert_eq!(checkpoint.generation(), new, "{case}");
            (crate::media_case::Ended::Acknowledged, vec![new])
        }
        // The record is selected; only Store's own checkpoint did not finish.
        Err(
            worth_store_application::ApplicationHomeCheckpointDenial::PhysicalCheckpoint { .. }
            | worth_store_application::ApplicationHomeCheckpointDenial::PhysicalCheckpointOutcome(_),
        ) => (crate::media_case::Ended::Acknowledged, vec![new]),
        Err(worth_store_application::ApplicationHomeCheckpointDenial::Publication(
            worth_store::physical_runtime::AnchoredRecordPublicationFailure::ProvenNoEffect(_),
        )) => (crate::media_case::Ended::Refused, vec![old]),
        Err(_) => (
            crate::media_case::Ended::Unresolved { recovered: false },
            vec![old, new],
        ),
    }
}

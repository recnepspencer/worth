//! Construction refusal reaches the caller without retaining the physical home.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::commit_strategies::{
    CommitStrategyDescriptor, CommitStrategyFamilyName, CommitStrategyId,
    CommitStrategyRegistration, CommitStrategySemanticName, CommitStrategyVersion,
    DuplicateCommitStrategyRegistration, PersistentArtifactName, StrategyInputSchemaName,
    StrategyInputSchemaVersion, StrategyIntentName, StrategyOutputSchemaName,
    StrategyPacketContract, StrategyReadContract, StrategyReadCostClass, StrategyReadLocalityClass,
    StrategyReadScopeClass, StrategyTraversalBasis,
};
use worth_relational::facade::runtime::{RelationalRuntimeApi, RelationalRuntimeConstructionDenial};

use crate::{
    RelationalStoreHome, RelationalStoreHomeClosed, RelationalStoreHomeOpenRefusal,
    RelationalStoreHomeRequest,
};

#[test]
fn duplicate_strategy_refuses_open_with_its_cause_and_releases_the_home() {
    let registration = CommitStrategyRegistration::new(CommitStrategyDescriptor::new(
        CommitStrategyId(41),
        CommitStrategySemanticName::new("store.construction.refusal"),
        CommitStrategyFamilyName::new("store.construction"),
        CommitStrategyVersion::new(1, 0),
        StrategyIntentName::new("construction.refusal"),
        StrategyInputSchemaName::new("construction.input.v1"),
        StrategyInputSchemaVersion(1),
        StrategyOutputSchemaName::new("construction.output.v1"),
        StrategyReadContract {
            scope_class: StrategyReadScopeClass::ExplicitTargetsOnly,
            locality_class: StrategyReadLocalityClass::SinglePartition,
            traversal_basis: StrategyTraversalBasis::NoTraversal,
            packet_contract: StrategyPacketContract::ProjectionOnly,
            cost_class: StrategyReadCostClass::ORequestedSurface,
        },
        PersistentArtifactName::new("store.construction.refusal"),
    ))
    .expect("valid strategy declaration");
    let builder = RelationalRuntimeApi::builder()
        .commit_strategy(registration.clone())
        .commit_strategy(registration);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let refused = RelationalStoreHome::open(
        RelationalStoreHomeRequest::new(&location, builder),
        &execution,
    );
    assert!(matches!(
        refused,
        Err(RelationalStoreHomeOpenRefusal::RuntimeConstruction(
            RelationalRuntimeConstructionDenial::CommitStrategy(
                DuplicateCommitStrategyRegistration::Id {
                    id: CommitStrategyId(41),
                }
            )
        ))
    ));

    // A real production reopen independently proves the refusal released its
    // Store custody rather than leaving the physical location locked.
    let home = RelationalStoreHome::open(
        RelationalStoreHomeRequest::new(&location, RelationalRuntimeApi::builder()),
        &execution,
    )
    .expect("the refused location remains usable by a valid runtime");
    assert!(matches!(home.close(), RelationalStoreHomeClosed::Closed(_)));
}

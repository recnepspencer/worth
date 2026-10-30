#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::scheduler::locality::*;
use tempfile::tempdir;
use worth_foundational::aspects;

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::validated_value,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
};

#[test]
fn multi_artifact_store_scope_reaches_scheduler_with_exact_structural_relations() {
    let root = tempdir().unwrap();
    let (contract, identity, admission, witness) = admitted_contract(41);
    let state = match aspects()
        .authoritative_state()
        .admit([validated_value(&contract, "multi-artifact-locality")])
    {
        worth_proof::TransitionOutcome::Success(state) => state,
        outcome => panic!("state should admit: {outcome:?}"),
    };
    let fact = worth_store::aspect_native::StoreAspectBoundaryFact::from_admitted_state(
        identity,
        worth_store::aspect_native::StoreAspectAuthorityInput::new(state, witness),
    )
    .unwrap();
    let basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        fact,
        admission.clone(),
    )
    .unwrap();
    let security = security_scope(witness);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::new(security, [admission])
            .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let before = serving.media_counters();

    let batch = scheduler_admit(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            worth_store::physical_runtime::PhysicalWorkScope::batch([
                coordinate(
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                    0,
                    8,
                ),
                coordinate(
                    worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
                    32,
                    8,
                ),
            ])
            .unwrap(),
            basis.clone(),
            security,
        )
        .unwrap(),
    );
    let overlapping = scheduler_admit(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            worth_store::physical_runtime::PhysicalWorkScope::one(coordinate(
                worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
                36,
                8,
            )),
            basis.clone(),
            security,
        )
        .unwrap(),
    );
    let adjacent = scheduler_admit(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            worth_store::physical_runtime::PhysicalWorkScope::batch([
                coordinate(
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                    8,
                    8,
                ),
                coordinate(
                    worth_store_physical_format::RecordArtifactFile::Segment {
                        segment: 3,
                        generation: 1,
                    },
                    0,
                    8,
                ),
            ])
            .unwrap(),
            basis,
            security,
        )
        .unwrap(),
    );

    let batch_locality = batch.queue_plan().grouping_basis().locality().unwrap();
    let overlapping_locality = overlapping
        .queue_plan()
        .grouping_basis()
        .locality()
        .unwrap();
    let adjacent_locality = adjacent.queue_plan().grouping_basis().locality().unwrap();
    assert_eq!(
        batch_locality.relation(overlapping_locality),
        worth_store_io_scheduler::QueueLocalityRelation::OverlappingOrInterleaved
    );
    assert_eq!(
        batch_locality.relation(adjacent_locality),
        worth_store_io_scheduler::QueueLocalityRelation::Adjacent
    );
    assert_eq!(batch.intent().scope().coordinates().len(), 2);
    assert_eq!(serving.media_counters(), before);
    drop((batch, overlapping, adjacent));
    serving.close();
}

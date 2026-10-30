#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::locality::route_backpressure::*;
use tempfile::tempdir;
use worth_store_physical_format::RecordArtifactFile;
use {
    worth_store::physical_runtime::PhysicalSignalAspectBindingSet,
    worth_store::physical_runtime::PhysicalWorkCapacity,
    worth_store::physical_runtime::PhysicalWorkProfileDeclaration,
    worth_store::physical_runtime::PhysicalWorkReadiness,
};

use {
    crate::physical_record_journeys::physical_work::locality::admit_read,
    crate::physical_record_journeys::physical_work::locality::projection_fact,
    crate::physical_record_journeys::physical_work::locality::read_dependency,
    crate::physical_record_journeys::physical_work::locality::request,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
};

#[test]
fn saturated_signal_route_does_not_capture_an_independent_route() {
    let root = tempdir().unwrap();
    let (left_contract, left_identity, left_admission, witness) =
        admitted_named_contract("store.physical.work.saturated-left", 917, 1);
    let (right_contract, right_identity, right_admission, _) =
        admitted_named_contract("store.physical.work.independent-right", 918, 1);
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(
        16,
        1,
        16,
        1024 * 1024,
        16 * 1024 * 1024,
    )
    .unwrap();
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [
                read_dependency(left_admission.clone()),
                read_dependency(right_admission.clone()),
            ],
        )
        .unwrap()
        .with_capacity(capacity);
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        profile.clone(),
    );
    let left_route = bindings
        .binding_for_identity(&left_identity)
        .unwrap()
        .digest();
    let right_route = bindings
        .binding_for_identity(&right_identity)
        .unwrap()
        .digest();
    let left_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        projection_fact(&left_contract, left_identity, witness, "left"),
        left_admission,
    )
    .unwrap();
    let right_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        projection_fact(&right_contract, right_identity, witness, "right"),
        right_admission,
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    assert_eq!(
        serving.certification_physical_signal_route_depth(left_route),
        Some(0),
        "left binding digest must name an installed empty route"
    );
    assert_eq!(
        serving.certification_physical_signal_route_depth(right_route),
        Some(0),
        "right binding digest must name an installed empty route"
    );
    let before = serving.media_counters();
    let mut left = (0..9)
        .map(|offset| {
            admit_read(
                &serving,
                request(
                    left_basis.clone(),
                    witness,
                    worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                    offset * 8,
                ),
            )
        })
        .collect::<Vec<_>>();
    let right = admit_read(
        &serving,
        request(
            right_basis,
            witness,
            worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
            0,
        ),
    );
    let gate = serving.certification_pause_physical_signal_after_dequeue();

    let readiness = std::thread::scope(|scope| {
        let runtime = &serving;
        let first_work = left.remove(0);
        let first = scope.spawn(move || runtime.request_physical_work(first_work));
        if !gate.await_arrivals(1) {
            gate.release();
            panic!("Signal worker did not reach the named dequeue yieldpoint");
        }
        let left = left
            .into_iter()
            .map(|work| scope.spawn(move || runtime.request_physical_work(work)))
            .collect::<Vec<_>>();
        if !await_route_depth(&serving, left_route, 8) {
            gate.release();
            panic!(
                "left Signal route did not reach its bounded capacity: left={:?}, right={:?}",
                serving.certification_physical_signal_route_depth(left_route),
                serving.certification_physical_signal_route_depth(right_route),
            );
        }
        let right = scope.spawn(move || runtime.request_physical_work(right));
        if !await_route_depth(&serving, right_route, 1) {
            gate.release();
            panic!("independent Signal route could not enqueue under left-route pressure");
        }
        let right_progress = serving.certification_pause_physical_signal_after_dequeue();
        gate.release();
        if !right_progress.await_arrivals(1) {
            right_progress.release();
            panic!("independent Signal route was not selected ahead of saturated route A");
        }
        assert_eq!(
            serving.certification_physical_signal_route_depth(left_route),
            Some(8),
            "route B must progress while route A remains saturated"
        );
        assert_eq!(
            serving.certification_physical_signal_route_depth(right_route),
            Some(0),
            "route B must leave its own mailbox when independently selected"
        );
        right_progress.release();

        std::iter::once(first)
            .chain(left)
            .chain(std::iter::once(right))
            .map(|join| join.join().unwrap().unwrap())
            .collect::<Vec<_>>()
    });

    assert!(readiness.iter().all(|result| matches!(
        result,
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
    )));
    drop(readiness);
    assert!(await_signal_cleanup(&serving));
    assert_eq!(serving.media_counters(), before);
    serving.close();
}

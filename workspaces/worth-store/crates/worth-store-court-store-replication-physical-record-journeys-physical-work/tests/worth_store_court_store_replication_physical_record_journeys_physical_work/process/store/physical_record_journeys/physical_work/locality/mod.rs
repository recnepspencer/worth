#[allow(unused_imports)]
use crate::physical_record_journeys::physical_work::locality::*;
use tempfile::tempdir;

use {
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::admitted_named_contract,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::authority::security_scope,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile,
    worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::EXPECTED_NATIVE_RECORD_BINDING_COUNT,
};

#[test]
fn disjoint_read_bindings_progress_through_one_graph_and_independent_bounded_routes() {
    let root = tempdir().unwrap();
    let (left_contract, left_identity, left_admission, witness) =
        admitted_named_contract("store.physical.work.left-read", 911, 1);
    let (right_contract, right_identity, right_admission, _) =
        admitted_named_contract("store.physical.work.right-read", 912, 1);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [
                read_dependency(left_admission.clone()),
                read_dependency(right_admission.clone()),
            ],
        )
        .unwrap();
    let caller_binding_count = u16::try_from(profile.contract_count()).unwrap();
    let left_fact = projection_fact(&left_contract, left_identity.clone(), witness, "left");
    let right_fact = projection_fact(&right_contract, right_identity.clone(), witness, "right");
    let left_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        left_fact.clone(),
        left_admission,
    )
    .unwrap();
    let right_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        right_fact,
        right_admission,
    )
    .unwrap();
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
    assert_ne!(left_route, right_route);
    let delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        bindings.binding_for_identity(&left_identity).unwrap(),
        &left_fact,
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                0,
                8,
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let topology = serving.physical_signal_observation().unwrap();
    assert_eq!(
        topology.graph_owner_count(),
        1,
        "physical dependency truth must remain in one derived Signal graph"
    );
    let installed_binding_count = caller_binding_count + EXPECTED_NATIVE_RECORD_BINDING_COUNT;
    assert_eq!(
        topology.aspect_binding_count(),
        installed_binding_count,
        "caller bindings and all twelve native record/durability bindings must share one graph"
    );
    assert_eq!(
        topology.locality_owner_count(),
        installed_binding_count,
        "each installed binding must own a distinct bounded Signal route"
    );
    assert_eq!(
        serving.certification_physical_signal_route_depth(left_route),
        Some(0),
        "the caller's left binding must retain its route identity after native extension"
    );
    assert_eq!(
        serving.certification_physical_signal_route_depth(right_route),
        Some(0),
        "the caller's right binding must retain its route identity after native extension"
    );
    serving
        .certification_apply_physical_aspect_delta(delta)
        .unwrap();
    let left = admit_read(
        &serving,
        request(
            left_basis,
            witness,
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
        ),
    );
    let right = admit_read(
        &serving,
        request(
            right_basis,
            witness,
            worth_store_physical_format::RecordArtifactFile::RootManifest { generation: 1 },
            0,
        ),
    );
    let before = serving.media_counters();

    std::thread::scope(|scope| {
        let left = scope.spawn(|| serving.request_physical_work(left));
        let right = scope.spawn(|| serving.request_physical_work(right));
        assert!(matches!(
            left.join().unwrap().unwrap(),
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
        ));
        assert!(matches!(
            right.join().unwrap().unwrap(),
            worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_)
        ));
    });

    assert_eq!(serving.media_counters(), before);
    serving.close();
}

#[test]
fn same_binding_revalidation_changes_only_the_intersecting_physical_scope() {
    let root = tempdir().unwrap();
    let (contract, identity, admission, witness) =
        admitted_named_contract("store.physical.work.scope-local-read", 913, 1);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [read_dependency(admission.clone())],
        )
        .unwrap();
    let fact = projection_fact(&contract, identity.clone(), witness, "available");
    let basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        fact.clone(),
        admission,
    )
    .unwrap();
    let changed_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    );
    let untouched_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            16,
            8,
        )
        .unwrap(),
    );
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        profile.clone(),
    );
    let delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        bindings.binding_for_identity(&identity).unwrap(),
        &fact,
        changed_scope.clone(),
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let changed = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            changed_scope,
            basis.clone(),
            security_scope(witness),
        )
        .unwrap(),
    );
    let untouched = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            untouched_scope,
            basis,
            security_scope(witness),
        )
        .unwrap(),
    );
    let untouched = ready(serving.request_physical_work(untouched).unwrap());
    let changed = ready(serving.request_physical_work(changed).unwrap());
    let changed_lineage = changed.signal_request();
    let untouched_lineage = untouched.signal_request();

    serving
        .certification_apply_physical_aspect_delta(delta)
        .unwrap();

    let changed = ready(serving.revalidate_physical_work(changed).unwrap());
    let untouched = ready(serving.revalidate_physical_work(untouched).unwrap());
    assert_ne!(changed.signal_request(), changed_lineage);
    assert_eq!(untouched.signal_request(), untouched_lineage);
    assert_eq!(untouched.revalidated_from_signal_request(), None);
    serving.close();
}

#[test]
fn blocked_same_binding_revalidation_retains_the_exact_active_lineage() {
    let root = tempdir().unwrap();
    let (contract, identity, admission, witness) =
        admitted_named_contract("store.physical.work.blocked-lineage", 914, 1);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [read_dependency(admission.clone())],
        )
        .unwrap();
    let fact = projection_fact(&contract, identity.clone(), witness, "available");
    let basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        fact.clone(),
        admission,
    )
    .unwrap();
    let changed_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            0,
            8,
        )
        .unwrap(),
    );
    let newer_scope = worth_store::physical_runtime::PhysicalWorkScope::one(
        worth_store_physical_format::RecordFrameCoordinate::new(
            worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
            16,
            8,
        )
        .unwrap(),
    );
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        profile.clone(),
    );
    let delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        bindings.binding_for_identity(&identity).unwrap(),
        &fact,
        changed_scope.clone(),
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let older = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            changed_scope,
            basis.clone(),
            security_scope(witness),
        )
        .unwrap(),
    );
    let newer = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            newer_scope,
            basis,
            security_scope(witness),
        )
        .unwrap(),
    );
    let older = ready(serving.request_physical_work(older).unwrap());
    let older_lineage = older.signal_request();
    let _newer = ready(serving.request_physical_work(newer).unwrap());
    serving
        .certification_apply_physical_aspect_delta(delta)
        .unwrap();

    let blocked = match serving.revalidate_physical_work(older).unwrap() {
        worth_store::physical_runtime::PhysicalWorkReadiness::Blocked(blocked) => blocked,
        worth_store::physical_runtime::PhysicalWorkReadiness::Ready(_) => {
            panic!("superseded active lineage must remain blocked")
        }
    };
    assert_eq!(blocked.active_request(), Some(older_lineage));
    drop(blocked);
    serving.close();
}

#[test]
fn same_scope_delta_does_not_cross_an_independent_signal_binding() {
    let root = tempdir().unwrap();
    let (left_contract, left_identity, left_admission, witness) =
        admitted_named_contract("store.physical.work.binding-left", 915, 1);
    let (right_contract, right_identity, right_admission, _) =
        admitted_named_contract("store.physical.work.binding-right", 916, 1);
    let profile =
        worth_store::physical_runtime::PhysicalWorkProfileDeclaration::from_signal_aspects(
            security_scope(witness),
            [
                read_dependency(left_admission.clone()),
                read_dependency(right_admission.clone()),
            ],
        )
        .unwrap();
    let left_fact = projection_fact(&left_contract, left_identity.clone(), witness, "left");
    let right_fact = projection_fact(&right_contract, right_identity, witness, "right");
    let left_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        left_fact.clone(),
        left_admission,
    )
    .unwrap();
    let right_basis = worth_store::physical_runtime::PhysicalWorkSemanticBasis::projection(
        right_fact,
        right_admission,
    )
    .unwrap();
    let exact_scope = || {
        worth_store::physical_runtime::PhysicalWorkScope::one(
            worth_store_physical_format::RecordFrameCoordinate::new(
                worth_store_physical_format::RecordArtifactFile::BootstrapCatalog,
                0,
                8,
            )
            .unwrap(),
        )
    };
    let bindings = worth_store::physical_runtime::PhysicalSignalAspectBindingSet::from_profile(
        profile.clone(),
    );
    let delta = worth_store::physical_runtime::PhysicalWorkAspectDelta::from_boundary_fact(
        bindings.binding_for_identity(&left_identity).unwrap(),
        &left_fact,
        exact_scope(),
    )
    .unwrap();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let right = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            exact_scope(),
            right_basis,
            security_scope(witness),
        )
        .unwrap(),
    );
    let left = admit_read(
        &serving,
        worth_store::physical_runtime::PhysicalReadWorkRequest::new(
            exact_scope(),
            left_basis,
            security_scope(witness),
        )
        .unwrap(),
    );
    let right = ready(serving.request_physical_work(right).unwrap());
    let left = ready(serving.request_physical_work(left).unwrap());
    let right_lineage = right.signal_request();
    let left_lineage = left.signal_request();

    serving
        .certification_apply_physical_aspect_delta(delta)
        .unwrap();

    let right = ready(serving.revalidate_physical_work(right).unwrap());
    let left = ready(serving.revalidate_physical_work(left).unwrap());
    assert_eq!(right.signal_request(), right_lineage);
    assert_ne!(left.signal_request(), left_lineage);
    serving.close();
}
#[path = "route_backpressure/mod.rs"]
pub(crate) mod route_backpressure;

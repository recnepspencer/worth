//! A real durable Open cut, before Native, preserves the no-effect exception.
use super::*;
use host_observation::{projection::StateProjection, requests};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::{
    primary_graph::WorthQueryApplicationIdempotencyResolution as Resolution,
    WorthQueryApplicationHomeLocation,
};
const LOCATION: &str = "WORTH_HOST_NO_EFFECT_HOME";
const MARKERS: &str = "WORTH_HOST_NO_EFFECT_MARKERS";
const CHILD: &str =
    "process::application::host_oracle::no_effect_release::durable_open_before_native_child";
const KEY: u64 = 0x9922;
const VALUE: u64 = 90_992;

#[test]
fn recovered_no_effect_release_permits_one_fresh_execution_of_the_original_key() {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[(LOCATION, &location), (MARKERS, root.path())],
            &root.path().join("ready"),
            &root.path().join("finished")
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let before: StateProjection =
        serde_json::from_slice(&std::fs::read(root.path().join("before.json")).unwrap()).unwrap();
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::Directory(location),
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
        None,
        Default::default(),
        &execution,
    );
    let recovered = host_observation::projection::observe(home.runtime(), &execution);
    assert_eq!(recovered.head, before.head);
    assert_eq!(
        serde_json::to_value(&recovered.entities).unwrap(),
        serde_json::to_value(&before.entities).unwrap(),
        "complete Native discovery found no owner effect"
    );
    let page = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    let [row] = page.rows() else {
        panic!("the actual durable Open is recovered");
    };
    let original = home
        .runtime()
        .runtime()
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    assert!(original
        .inspect()
        .unwrap()
        .native_semantic_result()
        .is_none());
    home.runtime()
        .runtime()
        .release_product_publication_recovery(original, 0)
        .unwrap();
    assert!(matches!(
        requests::resolve_original(home.runtime(), KEY, VALUE).unwrap(),
        Resolution::Unseen
    ));
    let first = requests::set(home.runtime(), KEY, VALUE);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    let after = host_observation::projection::observe(home.runtime(), &execution);
    assert_eq!(after.occurrences.len(), before.occurrences.len() + 1);
    let again = requests::set(home.runtime(), KEY, VALUE);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(
        host_observation::projection::observe(home.runtime(), &execution).head,
        after.head
    );
    assert_eq!(
        host_observation::receipts::observe(&first, &before.head),
        host_observation::receipts::observe(&again, &before.head)
    );
    drop((first, again, page));
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no-effect release closes: {cause:?}"));
}

#[test]
fn durable_open_before_native_child() {
    let Some(location) = std::env::var_os(LOCATION) else {
        return;
    };
    let markers = std::path::PathBuf::from(std::env::var_os(MARKERS).unwrap());
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let control = host_interruption::opening::control(|_| {}, None, None);
    let physical = control.physical_control_for_test();
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::Directory(location.into()),
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        Some(control),
        Default::default(),
        &execution,
    );
    let before = host_observation::projection::observe(home.runtime(), &execution);
    std::fs::write(
        markers.join("before.json"),
        serde_json::to_vec(&before).unwrap(),
    )
    .unwrap();
    // The first post-open durable publication is World's actual Open. This
    // physical cut precedes its acknowledgement and therefore Native publish.
    let gate = physical.pause_mutation_for_test(worth_store::physical_runtime::certification::CertificationPhysicalMutationCheckpoint::AfterRootReplacement).unwrap();
    std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let _original = requests::set(home.runtime(), KEY, VALUE);
            Ok(())
        });
        assert!(crate::process_kill::await_mutation_seam(
            &gate,
            &worker,
            std::time::Instant::now() + crate::killed_child::PARK_DEADLINE
        ));
        crate::killed_child::announce(
            &markers.join("ready"),
            b"durable World Open before Native acknowledgement",
        );
        crate::killed_child::park();
    });
}

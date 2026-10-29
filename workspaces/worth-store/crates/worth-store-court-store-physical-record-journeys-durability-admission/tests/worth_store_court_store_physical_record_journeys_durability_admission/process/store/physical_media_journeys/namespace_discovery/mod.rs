#[allow(unused_imports)] use crate::physical_media_journeys::namespace_discovery::*;
use {std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::MediaOwnedPhysicalRuntime, worth_store::physical_runtime::MediaShutdownOutcome, worth_store::physical_runtime::ObservationError};
use {worth_store_physical_backend::MediaCounterSnapshot, worth_store_physical_backend::MediaOperationRole};

use {crate::child_dispatch::emit, crate::child_dispatch::run_role, crate::child_dispatch::ChildReport};
use {crate::physical_media_journeys::admit_runtime, crate::physical_media_journeys::media_admission};



use {crate::namespace_discovery::canonical_equivalence::canonical_mismatch_loci};
use {crate::namespace_discovery::observer_report::assert_namespace_shape, crate::namespace_discovery::observer_report::observe_namespace};

#[test]
fn namespace_creation_and_fresh_process_discovery() {
    let parent = tempfile::tempdir().unwrap();
    let absent = parent.path().join("absent-store");
    let existing_empty = parent.path().join("existing-empty-store");
    std::fs::create_dir(&existing_empty).unwrap();

    let absent_writer = run_role("namespace-writer", &absent, &[]);
    let empty_writer = run_role("namespace-writer", &existing_empty, &[]);
    assert_initialization_counters(&absent_writer);
    assert_initialization_counters(&empty_writer);

    let relocated = parent.path().join("relocated-store");
    std::fs::rename(&absent, &relocated).unwrap();
    let before_reopen = observe_namespace(&relocated);
    assert_namespace_shape(&before_reopen);
    assert_eq!(
        before_reopen.value("identity"),
        absent_writer.value("store")
    );

    let observer_environment = [
        (
            "WORTH_STORE_C4_OBSERVED_NAMESPACE_VERSION",
            before_reopen.value("namespace_version"),
        ),
        (
            "WORTH_STORE_C4_OBSERVED_ENCODING_VERSION",
            before_reopen.value("encoding_version"),
        ),
        (
            "WORTH_STORE_C4_OBSERVED_IDENTITY",
            before_reopen.value("identity"),
        ),
    ];
    let first_reopener = run_role("namespace-reopener", &relocated, &observer_environment);
    let second_reopener = run_role("namespace-reopener", &relocated, &observer_environment);
    assert_reopener_identity_independence(&absent_writer, &first_reopener);
    assert_reopener_identity_independence(&first_reopener, &second_reopener);
    for reopener in [&first_reopener, &second_reopener] {
        assert_eq!(reopener.value("store"), absent_writer.value("store"));
        assert_eq!(reopener.value("canonical"), "equivalent");
        assert_eq!(
            reopener.value("mutant_loci"),
            "namespace.version,encoding.version,identity,publication.posture"
        );
        assert_reopen_has_no_initialization_effects(reopener);
    }

    let after_reopen = observe_namespace(&relocated);
    assert_namespace_shape(&after_reopen);
    assert_eq!(
        before_reopen.value("identity_record_sha256"),
        after_reopen.value("identity_record_sha256")
    );
    assert_eq!(before_reopen.path_kinds(), after_reopen.path_kinds());

    let empty_observation = observe_namespace(&existing_empty);
    assert_namespace_shape(&empty_observation);
    assert_eq!(
        empty_observation.value("identity"),
        empty_writer.value("store")
    );
    assert_ne!(empty_writer.value("store"), absent_writer.value("store"));
    assert_eq!(
        sorted_child_paths(parent.path()),
        vec![
            "existing-empty-store".to_owned(),
            "relocated-store".to_owned()
        ]
    );
}

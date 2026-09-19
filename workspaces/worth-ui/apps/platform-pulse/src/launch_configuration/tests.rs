use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::admit;
use worth_ui_platform_pulse::observation_contract::PlatformPulseLaunchConfigurationDenial;

static NEXT_INSTALLATION: AtomicU64 = AtomicU64::new(1);

struct IsolatedInstallation {
    root: PathBuf,
}

impl IsolatedInstallation {
    fn empty() -> Self {
        let ordinal = NEXT_INSTALLATION.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "worth-ui-platform-pulse-phase5-{}-{ordinal}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("create isolated installation");
        Self { root }
    }

    fn with_entry() -> Self {
        let installation = Self::empty();
        let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app");
        for source in std::fs::read_dir(source_root).expect("read canonical sources") {
            let source = source.expect("canonical source entry");
            if source
                .path()
                .extension()
                .is_some_and(|extension| extension == "wui")
            {
                std::fs::copy(source.path(), installation.root.join(source.file_name()))
                    .expect("copy canonical source");
            }
        }
        installation
    }
}

impl Drop for IsolatedInstallation {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).expect("remove isolated installation");
    }
}

fn canonical_query_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("query_samples")
}

fn canonical_intent_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("intent_samples")
}

fn admit_test(
    arguments: impl IntoIterator<Item = std::ffi::OsString>,
    default_source_root: PathBuf,
) -> Result<super::AdmittedPlatformPulseLaunchConfiguration, PlatformPulseLaunchConfigurationDenial>
{
    admit(
        arguments,
        default_source_root,
        canonical_query_root(),
        canonical_intent_root(),
    )
}

#[test]
fn no_arguments_admit_all_checked_in_canonical_installations() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let admitted = admit_test(Vec::new(), manifest.join("app")).expect("canonical launch");
    assert_eq!(admitted.source_root(), manifest.join("app"));
    assert_eq!(
        admitted.intent_source_root(),
        manifest.join("intent_samples")
    );
}

#[test]
fn explicit_absolute_installation_is_admitted_without_test_authority() {
    let installation = IsolatedInstallation::with_entry();
    let admitted = admit_test(
        [
            "--source-root".into(),
            installation.root.clone().into_os_string(),
        ],
        PathBuf::from("unused"),
    )
    .expect("explicit launch");
    assert_eq!(admitted.source_root(), installation.root);
}

#[test]
fn explicit_absolute_installation_reaches_real_application_preparation() {
    let installation = IsolatedInstallation::with_entry();
    let admitted = admit_test(
        [
            "--source-root".into(),
            installation.root.clone().into_os_string(),
        ],
        PathBuf::from("unused"),
    )
    .expect("explicit launch");
    let prepared = crate::application::prepare_composition(&admitted)
        .expect("isolated source reaches real filesystem and application preparation");
    let crate::application::PreparedPlatformPulseComposition {
        builder,
        watcher,
        query_lifecycle,
        query_watcher,
        intent_watcher,
        intent_action_owner,
        ..
    } = prepared;
    let application = builder.freeze().expect("prepared application freezes");
    let definitions = application.capabilities().intent_definitions();
    assert_eq!(definitions.len(), 3);
    assert!(definitions
        .get(&worth_ui::facade::intent::UiIntentId::stable(
            worth_ui_platform_pulse::intent::PLATFORM_PULSE_ACTION_DEFINITION,
        ))
        .is_some());
    assert!(definitions
        .get(&worth_ui::facade::intent::UiIntentId::stable(
            worth_ui_platform_pulse::intent::PLATFORM_PULSE_OPEN_PORTAL_DEFINITION,
        ))
        .is_some());
    assert!(definitions
        .get(&worth_ui::facade::intent::UiIntentId::stable(
            worth_ui_platform_pulse::intent::PLATFORM_PULSE_CLOSE_PORTAL_DEFINITION,
        ))
        .is_some());
    let action_view = worth_ui::facade::query_binding::WorthUiQueryViewIdentity::new(
        worth_ui_platform_pulse::intent::PLATFORM_PULSE_ACTION_QUERY_VIEW,
    )
    .expect("static Pulse action view identity");
    assert_ne!(
        action_view.as_str(),
        worth_ui_platform_pulse::PLATFORM_PULSE_STATUS_QUERY_VIEW,
        "Query action evidence and the visible scalar consequence remain distinct identities"
    );
    assert_ne!(
        application
            .generation_identity()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        0,
        "effect-free preparation retains one concrete host-neutral generation"
    );
    drop(application);
    assert!(matches!(
        query_lifecycle.close(),
        Err(crate::query_source::PlatformPulseQueryLifecycleDenial::OwnerNotLive)
    ));
    query_watcher
        .shutdown()
        .expect("isolated Query watcher shuts down");
    let intent_shutdown = intent_watcher
        .shutdown()
        .expect("isolated intent watcher shuts down");
    assert!(intent_shutdown.worker_joined());
    assert_eq!(intent_shutdown.pending_event_count(), 0);
    let intent_census = intent_action_owner.census();
    assert_eq!(intent_census.submitted(), 0);
    assert_eq!(intent_census.received(), 0);
    assert_eq!(intent_census.settled(), 0);
    assert_eq!(intent_census.retained(), 0);
    watcher
        .shutdown()
        .expect("isolated source watcher shuts down");
}

#[test]
fn intent_source_root_requires_the_exact_versioned_product_input() {
    let installation = IsolatedInstallation::with_entry();
    let empty_intent = IsolatedInstallation::empty();
    let denial = match admit(
        [
            "--source-root".into(),
            installation.root.clone().into_os_string(),
            "--intent-source-root".into(),
            empty_intent.root.clone().into_os_string(),
        ],
        PathBuf::from("unused"),
        canonical_query_root(),
        PathBuf::from("unused"),
    ) {
        Err(denial) => denial,
        Ok(_) => panic!("missing typed intent input must stop launch"),
    };
    assert!(matches!(
        denial,
        PlatformPulseLaunchConfigurationDenial::MissingIntentSource(_)
    ));
}

#[test]
fn invalid_launch_shapes_are_distinct_typed_denials() {
    let empty = IsolatedInstallation::empty();
    let missing = empty.root.join("missing");
    let file = empty.root.join("not-a-directory");
    std::fs::write(&file, b"not a directory").expect("write file");

    assert!(matches!(
        admit_test(["--source-root".into()], PathBuf::from("unused")),
        Err(PlatformPulseLaunchConfigurationDenial::MissingSourceRootValue)
    ));
    assert!(matches!(
        admit_test(
            ["--source-root".into(), "relative".into()],
            PathBuf::from("unused")
        ),
        Err(PlatformPulseLaunchConfigurationDenial::RelativeSourceRoot(
            _
        ))
    ));
    assert!(matches!(
        admit_test(
            ["--source-root".into(), missing.into_os_string()],
            PathBuf::from("unused")
        ),
        Err(PlatformPulseLaunchConfigurationDenial::MissingSourceRoot(_))
    ));
    assert!(matches!(
        admit_test(
            ["--source-root".into(), file.into_os_string()],
            PathBuf::from("unused")
        ),
        Err(PlatformPulseLaunchConfigurationDenial::SourceRootNotDirectory(_))
    ));
    assert!(matches!(
        admit_test(
            ["--source-root".into(), empty.root.clone().into_os_string()],
            PathBuf::from("unused")
        ),
        Err(PlatformPulseLaunchConfigurationDenial::MissingEntrySource(
            _
        ))
    ));
    assert!(matches!(
        admit_test(["--unknown".into()], PathBuf::from("unused")),
        Err(PlatformPulseLaunchConfigurationDenial::UnexpectedArgument)
    ));
}

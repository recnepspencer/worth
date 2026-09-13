use super::*;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn watched_preference_settles_writes_rejects_bad_revisions_and_joins() {
    let root = std::env::temp_dir().join(format!(
        "pulse-theme-watch-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("platform-pulse-theme.json");
    let mut watch = PlatformPulseThemePreferenceWatch::open(&root).unwrap();
    let (sender, receiver) = std::sync::mpsc::channel();
    watch.install_readiness(
        worth_ui_platform_pulse::PlatformPulseApplicationReadinessSignal::from_callback(
            move || {
                let _ = sender.send(());
            },
        ),
    );

    std::fs::write(&path, "{").unwrap();
    // Truncate and finish within the watcher quiet interval: only the finished
    // record may reach application preference admission.
    std::thread::sleep(Duration::from_millis(10));
    replace(&path, 1, "alternate");
    let preference = next(&mut watch, &receiver).unwrap();
    assert_eq!(preference.revision, 1);
    assert_eq!(preference.theme, PlatformPulseThemeChoice::Alternate);

    replace(&path, 1, "default");
    assert_eq!(
        next(&mut watch, &receiver),
        Err(PlatformPulseThemePreferenceDenial::StaleRevision)
    );
    std::fs::write(&path, "invalid").unwrap();
    assert_eq!(
        next(&mut watch, &receiver),
        Err(PlatformPulseThemePreferenceDenial::InvalidRecord)
    );
    for revision in 2..=8 {
        replace(&path, revision, "default");
    }
    let preference = next(&mut watch, &receiver).unwrap();
    assert_eq!(preference.revision, 8);
    assert_eq!(preference.theme, PlatformPulseThemeChoice::Default);
    watch.shutdown().unwrap();
    std::fs::remove_dir_all(root).unwrap();
}

fn replace(path: &std::path::Path, revision: u64, theme: &str) {
    let temporary = path.with_extension("pending");
    std::fs::write(
        &temporary,
        format!(r#"{{"schema_version":1,"revision":{revision},"theme":"{theme}"}}"#),
    )
    .unwrap();
    std::fs::rename(temporary, path).unwrap();
}

fn next(
    watch: &mut PlatformPulseThemePreferenceWatch,
    receiver: &std::sync::mpsc::Receiver<()>,
) -> Result<PlatformPulseThemePreference, PlatformPulseThemePreferenceDenial> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        receiver
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("watcher must wake the actual application input consumer");
        match watch.take_changed() {
            Ok(Some(preference)) => return Ok(preference),
            Ok(None) => {}
            Err(denial) => return Err(denial),
        }
    }
}

//! Exact failing coordinates, never permission for another metric or axis to grow.
use super::{
    growth::{self, Axis, GrowthFailure},
    observation::{Metric, Phase, Sample},
};

struct Exception {
    specimen: &'static str,
    metric: Metric,
    axis: Axis,
    lane: &'static str,
    issue: u32,
}

const CREATED_UTC_DAY: u64 = 20763; // 2026-11-06; expires at 2026-11-20T00:00Z.
const BASE: &str = "5f454efe0341d3527c43471ce219aa5005abb194";
const EXCEPTIONS: &[Exception] = &[];

pub fn verify(specimen: &str, samples: &[Sample; 4]) {
    let failures = growth::compare(samples);
    let unknown = unregistered(specimen, &failures);
    assert!(
        unknown.is_empty(),
        "new scale failures:\n{}",
        unknown.join("\n")
    );
    for exception in EXCEPTIONS.iter().filter(|entry| entry.specimen == specimen) {
        assert!(failures.iter().any(|failure| failure.metric == exception.metric && failure.axis == exception.axis),
            "remove healed ratchet entry: {:?}/{} {:?}, {} #{}", exception.metric.phase, exception.metric.name, exception.axis, exception.lane, exception.issue);
        let day = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 86400;
        assert!(
            day < CREATED_UTC_DAY + 14,
            "expired scale exception: {} #{}",
            exception.lane,
            exception.issue
        );
        assert_blocker_not_landed(exception.issue);
    }
    for failure in failures {
        let entry = EXCEPTIONS
            .iter()
            .find(|entry| matches(entry, specimen, &failure))
            .unwrap();
        eprintln!(
            "RATCHET {failure}; owner={} issue=#{} expires=2026-11-20T00:00Z",
            entry.lane, entry.issue
        );
    }
}

pub fn unregistered(specimen: &str, failures: &[GrowthFailure]) -> Vec<String> {
    failures
        .iter()
        .filter(|failure| {
            !EXCEPTIONS
                .iter()
                .any(|entry| matches(entry, specimen, failure))
        })
        .map(ToString::to_string)
        .collect()
}

fn matches(entry: &Exception, specimen: &str, failure: &GrowthFailure) -> bool {
    entry.specimen == specimen && entry.metric == failure.metric && entry.axis == failure.axis
}

fn assert_blocker_not_landed(issue: u32) {
    // Every lane's handoff uses "Fixes #issue". A fetched master containing
    // that closeout retires the exception even before its UTC expiry.
    let output = std::process::Command::new("git")
        .args(["log", &format!("{BASE}..origin/master"), "--format=%B"])
        .output()
        .expect("git resolves the ratchet source fence");
    assert!(
        output.status.success(),
        "cannot resolve origin/master source fence"
    );
    let marker = format!("fixes #{issue}");
    assert!(
        !String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .any(|line| line.trim().to_ascii_lowercase() == marker),
        "{} reached origin/master; remove its scale exception",
        marker
    );
}

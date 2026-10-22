//! Every seeded reuse edit is evaluated across the prior and current width.
use super::*;
const CHANGES: [Change; 14] = [
    Change::Gather,
    Change::Shared,
    Change::Key,
    Change::Input,
    Change::Fault,
    Change::Join,
    Change::Leave,
    Change::DigestSwap,
    Change::Replacement,
    Change::CarriedCeiling,
    Change::EmptyBefore,
    Change::EmptyAfter,
    Change::EmptyStable,
    Change::CodeIdentity,
];
#[test]
fn every_seeded_reuse_input_crosses_prior_and_current_worker_counts() {
    if let Ok(encoded) = std::env::var("WORTH_REUSE_WIDTH_CELL") {
        let v = encoded
            .split(',')
            .map(|s| s.parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        case_at(CHANGES[v[0]], v[1], v[2], v[3] != 0);
        return;
    }
    let mut count = 0;
    let mut anchors = std::collections::BTreeMap::new();
    let mut comparison_anchors = std::collections::BTreeMap::new();
    let mut red = Vec::new();
    for change in 0..CHANGES.len() {
        for prior in [1, 2, 5] {
            for current in [1, 2, 5] {
                for reuse in [false, true] {
                    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","checkpoint_recovery::computation_partition::oracle::parallel_history_reuse::worker_matrix::every_seeded_reuse_input_crosses_prior_and_current_worker_counts","--nocapture"]).env("WORTH_REUSE_WIDTH_CELL",format!("{change},{prior},{current},{}",u8::from(reuse))).output().unwrap();
                    count += 1;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if !output.status.success() {
                        red.push(format!("change {change} prior {prior} current {current} reuse {reuse}: {stdout}{}",String::from_utf8_lossy(&output.stderr)));
                    } else {
                        let total = history_bills(&stdout);
                        // The comparison history has no retained prior (or is
                        // explicitly reuse-off), so this half is mode-independent.
                        let comparison = (total.2, total.3);
                        if let Some(expected) = comparison_anchors.get(&change) {
                            if &comparison != expected {
                                red.push(format!("comparison history: change {change} prior {prior} current {current} reuse {reuse}: {expected:?} versus {comparison:?}"));
                            }
                        } else {
                            comparison_anchors.insert(change, comparison);
                        }
                        // Whole advancement includes native preparation and
                        // publication work. Compare widths at fixed reuse mode;
                        // case_at separately requires identical computation charges.
                        let history = (change, reuse);
                        if let Some(expected) = anchors.get(&history) {
                            if &total != expected {
                                red.push(format!("charge by posture: change {change} prior {prior} current {current} reuse {reuse}: {expected:?} versus {total:?}"));
                            }
                        } else {
                            println!("REUSE_ANCHOR:{change}:reuse={reuse}:{total:?}");
                            anchors.insert(history, total);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(
        count,
        CHANGES.len() * 3 * 3 * 2,
        "complete seeded reuse product"
    );
    for finding in &red {
        println!("REUSE_NAMED_RED:{finding}");
    }
    assert!(
        red.is_empty(),
        "every reuse law and whole-history charge is unchanged across prior/current worker widths at fixed reuse mode"
    );
}

fn history_bills(stdout: &str) -> (u64, u64, u64, u64) {
    let (line, names) = match stdout
        .lines()
        .find(|line| line.starts_with("HISTORY_BILLS:"))
    {
        Some(line) => (
            line,
            [
                "edited_work",
                "edited_span",
                "new_installation_work",
                "new_installation_span",
            ],
        ),
        None => (
            stdout
                .lines()
                .find(|line| line.starts_with("SIBLING_BRANCH_BILLS:"))
                .unwrap(),
            [
                "edited_retained_work",
                "edited_retained_span",
                "edited_reuse_off_work",
                "edited_reuse_off_span",
            ],
        ),
    };
    let field = |name| {
        line.split(':')
            .filter_map(|field| field.split_once('='))
            .find(|(key, _)| *key == name)
            .unwrap()
            .1
            .parse::<u64>()
            .unwrap()
    };
    (
        field(names[0]),
        field(names[1]),
        field(names[2]),
        field(names[3]),
    )
}

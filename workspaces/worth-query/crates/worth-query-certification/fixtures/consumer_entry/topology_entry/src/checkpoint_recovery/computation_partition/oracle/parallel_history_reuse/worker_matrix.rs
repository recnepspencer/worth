//! Every seeded reuse edit is evaluated across the prior and current width.
use super::*;
const CHANGES: [Change; 10] = [
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
                        let total = stdout
                            .lines()
                            .find(|s| s.starts_with("REUSE_TOTAL:"))
                            .unwrap()
                            .to_owned();
                        if let Some(expected) = anchors.get(&change) {
                            if &total != expected {
                                red.push(format!("charge by posture: change {change} prior {prior} current {current} reuse {reuse}: {expected} versus {total}"));
                            }
                        } else {
                            println!("REUSE_ANCHOR:{change}:{total}");
                            anchors.insert(change, total);
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
        "every reuse law and charge is unchanged across prior/current posture"
    );
}

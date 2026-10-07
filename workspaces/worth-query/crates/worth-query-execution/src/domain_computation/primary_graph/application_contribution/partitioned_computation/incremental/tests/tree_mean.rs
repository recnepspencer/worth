//! A predetermined random-identity workload, with a threshold derived before
//! observing reduction work. Topology does not change during these updates.

use super::*;
use sha2::{Digest, Sha256};

const SIZES: [usize; 3] = [1024, 2048, 4096];
const SETS: usize = 128;
const SEED: &[u8] = b"worth-query-tree-mean-law-v1";

/// Under uniform independent identities/priorities, the Cartesian tree is
/// a random BST. Its internal path length I_n obeys
/// I_n = n-1 + I_L + I_(n-1-L), L uniform on 0..n.
/// E[I_n]/n+1 = 2(n+1)H_n/n-3 < 2 ln n-1.8 for n>=1024.
/// Var[I_n]/n² < 0.5 there. Independent sets reduce mean variance by SETS;
/// Chebyshev and the three-size union bound give rejection probability
/// <= 3*0.5/(128*1.8²) < 0.0037 at the unfitted ceiling 2 ln n.
/// The recurrence below derives both moments before any tree runs. SHA-256
/// counter samples represent the declared uniform-identity model; all seeds,
/// sizes and thresholds are fixed here, rather than selected by measured work.
fn moments(max: usize) -> (Vec<f64>, Vec<f64>) {
    let mut means = vec![0.0; max + 1];
    let mut variances = vec![0.0; max + 1];
    for n in 1..=max {
        let nf = f64::from(u32::try_from(n).unwrap());
        means[n] = nf - 1.0 + (0..n).map(|l| means[l] + means[n - 1 - l]).sum::<f64>() / nf;
        variances[n] = (0..n)
            .map(|l| {
                let conditional = nf - 1.0 + means[l] + means[n - 1 - l];
                variances[l] + variances[n - 1 - l] + (conditional - means[n]).powi(2)
            })
            .sum::<f64>()
            / nf;
    }
    (means, variances)
}

#[test]
fn mean_reported_update_nodes_obey_the_random_treap_depth_law() {
    let (means, variances) = moments(*SIZES.last().unwrap());
    let mut union_risk = 0.0;
    for n in SIZES {
        let nf = f64::from(u32::try_from(n).unwrap());
        let expected = means[n] / nf + 1.0;
        let threshold = 2.0 * nf.ln();
        assert!(threshold - expected > 1.8);
        assert!(variances[n] / (nf * nf) < 0.5);
        union_risk += variances[n]
            / (nf * nf)
            / f64::from(u32::try_from(SETS).unwrap())
            / (threshold - expected).powi(2);
    }
    assert!(
        union_risk < 0.0037,
        "derive threshold before execution: {union_risk}"
    );
    let template = template();
    for n in SIZES {
        let mut total = 0_u128;
        for sample in 0..SETS {
            let mut keys: Vec<_> = (0..n)
                .map(|leaf| {
                    let mut hash = Sha256::new();
                    hash.update(SEED);
                    hash.update(u64::try_from(n).unwrap().to_le_bytes());
                    hash.update(u64::try_from(sample).unwrap().to_le_bytes());
                    hash.update(u64::try_from(leaf).unwrap().to_le_bytes());
                    let digest = hash.finalize();
                    Id::new(u64::from_le_bytes(digest[..8].try_into().unwrap()))
                })
                .collect();
            keys.sort();
            assert!(
                keys.windows(2).all(|pair| pair[0] != pair[1]),
                "unique sampled identities"
            );
            let retained = retained(&template, &keys);
            CALLS.set(0);
            let next = run(
                &retained,
                &keys,
                keys.iter().map(|id| (*id, 2)).collect(),
                u64::MAX,
            );
            reconcile(next.report);
            let TreeRun::Edited(metrics) = next.report else {
                panic!("mean workload must edit")
            };
            assert_eq!(
                *next.outcome.unwrap().result(),
                u64::try_from(2 * n).unwrap()
            );
            total += metrics.recombined_nodes;
        }
        let mean =
            f64::from(u32::try_from(total).unwrap()) / f64::from(u32::try_from(SETS * n).unwrap());
        let ceiling = 2.0 * f64::from(u32::try_from(n).unwrap()).ln();
        assert!(
            mean <= ceiling,
            "P={n}, mean={mean}, analytically derived ceiling={ceiling}"
        );
    }
}

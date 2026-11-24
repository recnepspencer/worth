//! Law-based property suite for `worth-store-branch-deltas`:
//! arbitrary branch lineage witnesses, delta layer plans, stable-basis support,
//! and continuation retention / window-resumption sequences against an in-memory
//! reference model.
use proptest::prelude::*;
use std::collections::BTreeMap;
use worth_store_branch_deltas::{
    admit_continuation_layout_support, admit_stable_basis_layout_support,
    branch_semantic_authority, reject_branch_delta_read_plan,
    reject_broadened_continuation_receipt, reject_stable_basis_layout_descriptor,
    BranchDeltaLayerId, BranchDeltaLayoutAccessDenialKind, BranchDeltaReadPlan,
    BranchDeltaReadRequest, ContinuationLayoutReport, SameBranchDescendantWitness,
};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_live_query::{
    live_query_semantic_authority, ContinuationRetentionStatus, StableBasisId,
};

#[derive(Debug, Clone)]
enum DeltaRetentionOp {
    AdmitBranchWitness {
        branch_idx: u8,
    },
    AdmitDeltaRead {
        witness_branch: u8,
        plan_branch: u8,
        layer_id: u64,
        declared_rows: u32,
    },
    DeclareBasisAndContinuation {
        basis_id: u64,
        support_rows: u32,
        window_rows: u32,
        retained: bool,
    },
    ResumeContinuation {
        target_basis: u64,
        receipt_basis: u64,
        batch_rows: u32,
    },
    ProbeBroadenedAndRawRefusals {
        basis_id: u64,
        window_rows: u32,
        broadened_rows: u32,
    },
}

fn row_count_strategy() -> impl Strategy<Value = u32> {
    prop_oneof![
        3 => 0_u32..=64,
        2 => prop::sample::select(vec![
            0_u32,
            1,
            u16::MAX as u32 - 1,
            u16::MAX as u32,
            u16::MAX as u32 + 1,
            u32::MAX,
        ]),
    ]
}

fn op_strategy() -> impl Strategy<Value = DeltaRetentionOp> {
    prop_oneof![
        3 => (0_u8..6).prop_map(|branch_idx| DeltaRetentionOp::AdmitBranchWitness { branch_idx }),
        4 => (0_u8..6, 0_u8..6, 1_u64..=1_000, row_count_strategy()).prop_map(
            |(witness_branch, plan_branch, layer_id, declared_rows)| {
                DeltaRetentionOp::AdmitDeltaRead {
                    witness_branch,
                    plan_branch,
                    layer_id,
                    declared_rows,
                }
            },
        ),
        4 => (1_u64..=8, row_count_strategy(), row_count_strategy(), any::<bool>()).prop_map(
            |(basis_id, support_rows, window_rows, retained)| {
                DeltaRetentionOp::DeclareBasisAndContinuation {
                    basis_id,
                    support_rows,
                    window_rows,
                    retained,
                }
            },
        ),
        4 => (1_u64..=8, 1_u64..=8, row_count_strategy()).prop_map(
            |(target_basis, receipt_basis, batch_rows)| DeltaRetentionOp::ResumeContinuation {
                target_basis,
                receipt_basis,
                batch_rows,
            },
        ),
        2 => (1_u64..=8, row_count_strategy(), row_count_strategy()).prop_map(
            |(basis_id, window_rows, broadened_rows)| {
                DeltaRetentionOp::ProbeBroadenedAndRawRefusals {
                    basis_id,
                    window_rows,
                    broadened_rows,
                }
            },
        ),
    ]
}

fn lineage_name(idx: u8) -> String {
    format!("main/branch-{idx}")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn branch_deltas_and_continuation_retention_agree_with_reference_model(
        ops in prop::collection::vec(op_strategy(), 1..=48),
    ) {
        let branch_auth = branch_semantic_authority();
        let lq_auth = live_query_semantic_authority();

        let mut witnesses: BTreeMap<u8, (SameBranchDescendantWitness, String)> = BTreeMap::new();
        let mut continuations: BTreeMap<u64, (ContinuationLayoutReport, u32, ContinuationRetentionStatus)> =
            BTreeMap::new();

        for op in ops {
            match op {
                DeltaRetentionOp::AdmitBranchWitness { branch_idx } => {
                    let lineage = lineage_name(branch_idx);
                    let w = branch_auth.admit_same_branch_descendant(lineage.clone());
                    prop_assert_eq!(w.branch_lineage(), lineage.as_str());
                    witnesses.insert(branch_idx, (w, lineage));
                }
                DeltaRetentionOp::AdmitDeltaRead {
                    witness_branch,
                    plan_branch,
                    layer_id,
                    declared_rows,
                } => {
                    let plan_lineage = lineage_name(plan_branch);
                    let req = BranchDeltaReadRequest::new(
                        BranchDeltaLayerId(layer_id),
                        plan_lineage.clone(),
                    );
                    let plan = BranchDeltaReadPlan::new(req, declared_rows);

                    // Raw plan can never stand in for layout authority.
                    prop_assert_eq!(
                        reject_branch_delta_read_plan(&plan).unwrap_err().kind(),
                        BranchDeltaLayoutAccessDenialKind::BranchDeltaPlanCannotStandInForLayoutAuthority
                    );

                    if let Some((witness, w_lineage)) = witnesses.get(&witness_branch) {
                        let res = witness.admit_branch_delta_layout(&plan);
                        if w_lineage == &plan_lineage {
                            let report = res.expect("matching lineage must admit delta layout");
                            prop_assert_eq!(
                                report.family_id(),
                                DurableArtifactFamilyId::BranchDeltaArtifact
                            );
                            prop_assert_eq!(report.layer_id(), BranchDeltaLayerId(layer_id));
                            prop_assert_eq!(report.branch_lineage(), plan_lineage.as_str());
                            prop_assert_eq!(report.declared_delta_rows(), declared_rows);
                            let est = report.support_estimate();
                            prop_assert_eq!(est.planned_range_lookups(), 1);
                            prop_assert_eq!(est.planned_maintenance_reads(), 1);
                            prop_assert_eq!(
                                est.planned_range_steps(),
                                declared_rows.min(u16::MAX as u32) as u16
                            );
                        } else {
                            prop_assert_eq!(
                                res.unwrap_err().kind(),
                                BranchDeltaLayoutAccessDenialKind::BranchDeltaLineageDoesNotMatchWitness
                            );
                        }
                    }
                }
                DeltaRetentionOp::DeclareBasisAndContinuation {
                    basis_id,
                    support_rows,
                    window_rows,
                    retained,
                } => {
                    let id = StableBasisId(basis_id);
                    let status = if retained {
                        ContinuationRetentionStatus::Retained
                    } else {
                        ContinuationRetentionStatus::RetentionRebindRequired
                    };
                    let basis_plan = lq_auth.declare_stable_basis_support(id, support_rows, status);
                    let basis_report = admit_stable_basis_layout_support(&basis_plan).unwrap();
                    prop_assert_eq!(
                        basis_report.family_id(),
                        DurableArtifactFamilyId::PlacementStableBasis
                    );
                    prop_assert_eq!(basis_report.stable_basis_id(), id);
                    prop_assert_eq!(basis_report.declared_support_rows(), support_rows);
                    prop_assert_eq!(basis_report.retention_status(), status);
                    prop_assert_eq!(
                        basis_report.support_estimate().planned_page_touches(),
                        support_rows.min(u16::MAX as u32) as u16
                    );

                    let cont_plan = lq_auth.declare_continuation_window(id, window_rows, status);
                    let cont_report = admit_continuation_layout_support(&cont_plan).unwrap();
                    prop_assert_eq!(
                        cont_report.family_id(),
                        DurableArtifactFamilyId::SupportCursor
                    );
                    prop_assert_eq!(cont_report.stable_basis_id(), id);
                    prop_assert_eq!(cont_report.declared_window_rows(), window_rows);
                    prop_assert_eq!(cont_report.retention_status(), status);
                    prop_assert_eq!(
                        cont_report.support_estimate().planned_range_steps(),
                        window_rows.min(u16::MAX as u32) as u16
                    );
                    continuations.insert(basis_id, (cont_report, window_rows, status));
                }
                DeltaRetentionOp::ResumeContinuation {
                    target_basis,
                    receipt_basis,
                    batch_rows,
                } => {
                    let Some((report, declared_window, status)) = continuations.get(&target_basis) else {
                        continue;
                    };
                    let receipt_plan = lq_auth.declare_continuation_window(
                        StableBasisId(receipt_basis),
                        batch_rows,
                        ContinuationRetentionStatus::Retained,
                    );
                    let receipt = lq_auth.admit_narrow_batch(&receipt_plan, batch_rows);
                    let outcome = report.resume_bounded_continuation(&receipt);
                    if *status == ContinuationRetentionStatus::RetentionRebindRequired {
                        prop_assert_eq!(
                            outcome.unwrap_err().kind(),
                            BranchDeltaLayoutAccessDenialKind::ContinuationRebindRequired
                        );
                    } else if target_basis != receipt_basis || batch_rows > *declared_window {
                        prop_assert_eq!(
                            outcome.unwrap_err().kind(),
                            BranchDeltaLayoutAccessDenialKind::ContinuationWindowOutOfRange
                        );
                    } else {
                        prop_assert!(outcome.is_ok());
                    }
                }
                DeltaRetentionOp::ProbeBroadenedAndRawRefusals {
                    basis_id,
                    window_rows,
                    broadened_rows,
                } => {
                    let id = StableBasisId(basis_id);
                    prop_assert_eq!(
                        reject_stable_basis_layout_descriptor(id).unwrap_err().kind(),
                        BranchDeltaLayoutAccessDenialKind::StableBasisDescriptorCannotStandInForLayoutAuthority
                    );
                    let plan = lq_auth.declare_continuation_window(
                        id,
                        window_rows,
                        ContinuationRetentionStatus::Retained,
                    );
                    let broadened = lq_auth.record_broadened_batch(&plan, broadened_rows);
                    prop_assert_eq!(
                        reject_broadened_continuation_receipt(&broadened).unwrap_err().kind(),
                        BranchDeltaLayoutAccessDenialKind::BroadenedContinuationCannotStandInForBoundedSupport
                    );
                }
            }
        }
    }
}

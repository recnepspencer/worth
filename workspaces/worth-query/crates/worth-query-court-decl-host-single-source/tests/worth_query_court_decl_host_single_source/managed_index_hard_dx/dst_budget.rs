//! Tight-budget refusal and same-host 4-family post-refusal parity probe for Hard DX DST.
use super::dst_oracle::{
    check_watch, expected_row, q2, q3, qi, qp, FourFamilyOracle, UNIT_B2, UNIT_B3,
};
use super::operations::AssetSpec;
use super::schema::*;
use super::{create_asset, install};
use worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::MapKernelStop;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn b2(lo: i64, hi: i64) -> QueryBox2D {
    QueryBox2D {
        min_x: lo,
        min_y: lo,
        max_x: hi,
        max_y: hi,
    }
}

fn b3(lo: i64, hi: i64) -> QueryBox3D {
    QueryBox3D {
        min_x: lo,
        min_y: lo,
        min_z: lo,
        max_x: hi,
        max_y: hi,
        max_z: hi,
    }
}

pub fn verify_budget_refusal(oracle: &FourFamilyOracle) -> Result<(), String> {
    let seed: Vec<AssetSpec> = oracle.rows.values().take(1).cloned().collect();
    let tight = install(&seed, None, Some(850_000));
    let scope = request_scope();
    let principal = authenticate_operator(tight.installed_schema(), &scope);
    let req = tight.runtime().request(&principal, &scope);
    let idx_p = req
        .query(qp(""))
        .ensure_index_ready::<TitlePrefixIndex>()
        .map_err(|e| format!("tight ensure_index_ready prefix: {e:?}"))?;
    let idx_i = req
        .query(qi(FocusWindow { start: 0, end: 1 }))
        .ensure_index_ready::<FocusIntervalIndex>()
        .map_err(|e| format!("tight ensure_index_ready interval: {e:?}"))?;
    let idx_2 = req
        .query(q2(UNIT_B2))
        .ensure_index_ready::<EquipmentBox2DIndex>()
        .map_err(|e| format!("tight ensure_index_ready box2d: {e:?}"))?;
    let idx_3 = req
        .query(q3(UNIT_B3))
        .ensure_index_ready::<RouteBox3DIndex>()
        .map_err(|e| format!("tight ensure_index_ready box3d: {e:?}"))?;

    // Open baseline watches across ALL 4 families before triggering the budget-refused watch open.
    let (w_iv, w_b2, w_b3) = (
        FocusWindow {
            start: 1_000,
            end: 1_010,
        },
        b2(1_000, 1_010),
        b3(1_000, 1_010),
    );
    let (_, mut w_p) = req
        .query(qp("WatchHit-"))
        .watch_index::<TitlePrefixIndex>()
        .map_err(|e| format!("baseline prefix watch: {e:?}"))?;
    let (_, mut w_i) = req
        .query(qi(w_iv))
        .watch_index::<FocusIntervalIndex>()
        .map_err(|e| format!("baseline interval watch: {e:?}"))?;
    let (_, mut w_2) = req
        .query(q2(w_b2))
        .watch_index::<EquipmentBox2DIndex>()
        .map_err(|e| format!("baseline box2d watch: {e:?}"))?;
    let (_, mut w_3) = req
        .query(q3(w_b3))
        .watch_index::<RouteBox3DIndex>()
        .map_err(|e| format!("baseline box3d watch: {e:?}"))?;

    let shared = "m".repeat(49_990);
    let mut refused = false;
    let mut ws = Vec::new();
    for i in 0..64u8 {
        let prefix = format!("{shared}{i:02}");
        match req.query(qp(&prefix)).watch_index::<TitlePrefixIndex>() {
            Ok((_, w)) => ws.push(w),
            Err(WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause))
                if matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::Work(
                        WorthQueryManagedIndexWorkDenial::Controlled(
                            WorthQueryManagedIndexStop::ValueDecode(
                                WorthQueryValueDecodeDenial::Checkpoint(MapKernelStop::WorkCeiling)
                            )
                        )
                    )
                ) || matches!(
                    cause.cause(),
                    WorthQueryManagedIndexDenialCause::QueryRead(read)
                        if matches!(
                            read.kind(),
                            WorthQueryApplicationOneShotDenialKind::ExecutionRequest(
                                WorthQueryProviderSessionDenialKind::ExecutionResource {
                                    denial: WorthQueryManagedComputationResourceDenial::WorkExhausted,
                                    ..
                                }
                            )
                        )
                ) =>
            {
                refused = true;
                break;
            }
            Err(other) => {
                return Err(format!(
                    "expected exact WorkCeiling refusal, got Err({other:?})"
                ));
            }
        }
    }
    for w in ws {
        let _ = w.close();
    }
    if !refused {
        return Err("tight work ceiling failed to refuse".to_owned());
    }

    // Same-host post-refusal one-shot full-row parity across ALL 4 families.
    let (broad_iv, broad_b2, broad_b3) = (
        FocusWindow {
            start: -1_000,
            end: 1_000,
        },
        b2(-1_000, 1_000),
        b3(-1_000, 1_000),
    );
    let mut seed_oracle = FourFamilyOracle::default();
    for s in &seed {
        seed_oracle.rows.insert(s.key, s.clone());
    }
    for (name, got, exp) in [
        (
            "prefix",
            req.query(qp(""))
                .execute()
                .map_err(|e| format!("post-refusal prefix: {e:?}"))?
                .rows()
                .to_vec(),
            seed_oracle.prefix_rows(""),
        ),
        (
            "interval",
            req.query(qi(broad_iv))
                .execute()
                .map_err(|e| format!("post-refusal interval: {e:?}"))?
                .rows()
                .to_vec(),
            seed_oracle.interval_rows(-1_000, 1_000),
        ),
        (
            "box2d",
            req.query(q2(broad_b2))
                .execute()
                .map_err(|e| format!("post-refusal box2d: {e:?}"))?
                .rows()
                .to_vec(),
            seed_oracle.box2d_rows(broad_b2),
        ),
        (
            "box3d",
            req.query(q3(broad_b3))
                .execute()
                .map_err(|e| format!("post-refusal box3d: {e:?}"))?
                .rows()
                .to_vec(),
            seed_oracle.box3d_rows(broad_b3),
        ),
    ] {
        if got != exp {
            return Err(format!(
                "post-refusal-{name} projected row mismatch: {got:?} != {exp:?}"
            ));
        }
    }

    // Unrelated mutation must NOT wake any of the 4 family watches.
    create_asset(
        &tight,
        AssetSpec::i64_only(
            901,
            "Unrelated-After-Refusal",
            (500, 510),
            [500; 3],
            [510; 3],
        ),
        901,
    );
    check_watch(
        "post-refusal-unrelated-prefix",
        req.query(qp("WatchHit-")).next_watch(&mut w_p),
        &[],
        false,
    )?;
    check_watch(
        "post-refusal-unrelated-interval",
        req.query(qi(w_iv)).next_watch(&mut w_i),
        &[],
        false,
    )?;
    check_watch(
        "post-refusal-unrelated-box2d",
        req.query(q2(w_b2)).next_watch(&mut w_2),
        &[],
        false,
    )?;
    check_watch(
        "post-refusal-unrelated-box3d",
        req.query(q3(w_b3)).next_watch(&mut w_3),
        &[],
        false,
    )?;

    // Matching mutation MUST wake all 4 family watches with exact projected rows.
    let hit = AssetSpec::i64_only(
        902,
        "WatchHit-After-Refusal",
        (1_002, 1_008),
        [1_002; 3],
        [1_008; 3],
    );
    create_asset(&tight, hit.clone(), 902);
    let exp = [expected_row(&hit)];
    check_watch(
        "post-refusal-matching-prefix",
        req.query(qp("WatchHit-")).next_watch(&mut w_p),
        &exp,
        true,
    )?;
    check_watch(
        "post-refusal-matching-interval",
        req.query(qi(w_iv)).next_watch(&mut w_i),
        &exp,
        true,
    )?;
    check_watch(
        "post-refusal-matching-box2d",
        req.query(q2(w_b2)).next_watch(&mut w_2),
        &exp,
        true,
    )?;
    check_watch(
        "post-refusal-matching-box3d",
        req.query(q3(w_b3)).next_watch(&mut w_3),
        &exp,
        true,
    )?;

    let _ = (w_p.close(), w_i.close(), w_2.close(), w_3.close());
    idx_p.close();
    idx_i.close();
    idx_2.close();
    idx_3.close();
    Ok(())
}

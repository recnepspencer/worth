//! Owner-backed read-path and managed equality-index observation for the real
//! Directory Home DST driver.
//!
//! Closed and refused states are inspected by reopening the real Directory Home
//! on disk to observe its authoritative persisted program version, rows, and
//! managed index state — never by replaying driver-cached values.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::ApplicationBuildIdentity, WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::{
    application_runtime_surface as identity, checkpoint_capture,
};

use super::driver::{request_scope, Driver, LiveHome};
use super::reference_model::{
    ObservedState, ReferenceModel, RowState, PRIORITIES, ROW_IDS, V2_VALUES,
};
use super::v2_program::{v2_request, DaymarkV2Program, DaymarkV2Schema};
use super::v2_queries::AllTasksV2Read;
use super::v3_program::{v3_request, DaymarkV3Program, DaymarkV3Schema};
use super::v3_queries::{AllTasksV3Read, TasksByPriorityRead};

fn static_v2_val(val: &str) -> &'static str {
    V2_VALUES
        .into_iter()
        .find(|&candidate| candidate == val)
        .unwrap_or_else(|| panic!("unexpected v2 value `{val}`"))
}

fn static_priority(val: &str) -> &'static str {
    PRIORITIES
        .into_iter()
        .find(|&candidate| candidate == val)
        .unwrap_or_else(|| panic!("unexpected priority `{val}`"))
}

fn observe_v2_rows(
    home: &WorthQueryApplicationHome<DaymarkV2Schema, DaymarkV2Program>,
) -> Result<BTreeMap<&'static str, RowState>, String> {
    let scope = request_scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let result = runtime
        .request(&principal, &scope)
        .query(AllTasksV2Read("p-main".to_owned()))
        .execute()
        .map_err(|e| format!("AllTasksV2Read failed: {e:?}"))?;
    let mut rows = BTreeMap::new();
    for id in ROW_IDS {
        rows.insert(id, RowState::default());
    }
    for row in result.rows() {
        let id = ROW_IDS
            .into_iter()
            .find(|&candidate| candidate == row.task_key)
            .ok_or_else(|| format!("unexpected v2 task_key `{}`", row.task_key))?;
        rows.insert(
            id,
            RowState {
                present: true,
                v2val: static_v2_val(&row.v2_value),
                priority: "None",
            },
        );
    }
    Ok(rows)
}

fn verify_priority_equality_index(
    home: &WorthQueryApplicationHome<DaymarkV3Schema, DaymarkV3Program>,
    model: &ReferenceModel,
) -> Result<(), String> {
    let scope = request_scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    for priority in PRIORITIES {
        let indexed = runtime
            .request(&principal, &scope)
            .query(TasksByPriorityRead {
                project: "p-main".to_owned(),
                priority: priority.to_owned(),
            })
            .execute()
            .map_err(|e| format!("TasksByPriorityRead({priority}) failed: {e:?}"))?;
        let observed_keys: BTreeSet<&'static str> = indexed
            .rows()
            .iter()
            .map(|row| {
                ROW_IDS
                    .into_iter()
                    .find(|&c| c == row.task_key)
                    .expect("known row id")
            })
            .collect();
        let expected_keys = model.rows_with_priority(priority);
        if observed_keys != expected_keys {
            return Err(format!(
                "v3 Priority index mismatch for `{priority}`: observed={observed_keys:?}, model={expected_keys:?}"
            ));
        }
    }
    Ok(())
}

fn observe_v3_rows_and_verify_index(
    home: &WorthQueryApplicationHome<DaymarkV3Schema, DaymarkV3Program>,
    model: &ReferenceModel,
) -> Result<BTreeMap<&'static str, RowState>, String> {
    let scope = request_scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let result = runtime
        .request(&principal, &scope)
        .query(AllTasksV3Read("p-main".to_owned()))
        .execute()
        .map_err(|e| format!("AllTasksV3Read failed: {e:?}"))?;
    let mut rows = BTreeMap::new();
    for id in ROW_IDS {
        rows.insert(id, RowState::default());
    }
    for row in result.rows() {
        let id = ROW_IDS
            .into_iter()
            .find(|&candidate| candidate == row.task_key)
            .ok_or_else(|| format!("unexpected v3 task_key `{}`", row.task_key))?;
        rows.insert(
            id,
            RowState {
                present: true,
                v2val: static_v2_val(&row.v2_value),
                priority: static_priority(&row.priority),
            },
        );
    }
    verify_priority_equality_index(home, model)?;
    Ok(rows)
}

/// Inspect a closed Directory Home directly from disk via WORTH's real opening
/// and query owners. Fails immediately if `TransitionPending` stranded `v2` or
/// if persisted rows/indexes diverged from the reference model.
fn inspect_persisted_closed_home(
    location: &Path,
    build: ApplicationBuildIdentity,
    model: &ReferenceModel,
) -> Result<(u32, &'static str, BTreeMap<&'static str, RowState>), String> {
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let v2_attempt = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(v2_request(location.to_path_buf(), build).open_in_advancement(&phase))
    })
    .map_err(|e| format!("persisted v2 probe bootstrap failed: {e:?}"))?;

    match *v2_attempt {
        Ok(v2_home) => {
            let rows = observe_v2_rows(&v2_home)?;
            v2_home
                .close(&execution)
                .map_err(|e| format!("persisted v2 probe close failed: {e:?}"))?
                .reclaim();
            Ok((2, "None", rows))
        }
        Err(refused) => {
            let recorded_v3 = refused
                .transition_required()
                .is_some_and(|req| req.recorded().selected().identity() == "daymark.program.v3");
            if !recorded_v3 {
                return Err(format!(
                    "closed Home refused v2 without being on v3 (possible stranded TransitionPending): {refused:?}"
                ));
            }
            refused
                .close(&execution)
                .map_err(|e| format!("close persisted probe v2 refusal failed: {e:?}"))?
                .reclaim();
            let v3_home = with_bootstrap_advancement_from_request(execution, |phase| {
                Box::new(v3_request(location.to_path_buf(), build).open_in_advancement(&phase))
            })
            .map_err(|e| format!("persisted v3 probe bootstrap failed: {e:?}"))?
            .map_err(|e| format!("persisted v3 probe open failed: {e:?}"))?;
            let rows = observe_v3_rows_and_verify_index(&v3_home, model)?;
            v3_home
                .close(&execution)
                .map_err(|e| format!("persisted v3 probe close failed: {e:?}"))?
                .reclaim();
            Ok((3, "Ready", rows))
        }
    }
}

impl Driver {
    pub fn observe(&mut self, model: &ReferenceModel) -> Result<ObservedState, String> {
        let (disk_version, home_open, open_version, index_state, rows) = match &self.live {
            LiveHome::Closed => {
                let (disk_version, index_state, rows) =
                    inspect_persisted_closed_home(&self.location, self.build, model)?;
                (disk_version, false, 0, index_state, rows)
            }
            LiveHome::V2(home) => {
                let rows = observe_v2_rows(home)?;
                (2, true, 2, "None", rows)
            }
            LiveHome::V3(home) => {
                let rows = observe_v3_rows_and_verify_index(home, model)?;
                (3, true, 3, "Ready", rows)
            }
        };
        self.last_observed_rows = rows.clone();
        Ok(ObservedState {
            disk_version,
            home_open,
            open_version,
            index_state,
            transition_runs: self.transition_runs,
            outcome: self.last_outcome,
            rows,
        })
    }
}

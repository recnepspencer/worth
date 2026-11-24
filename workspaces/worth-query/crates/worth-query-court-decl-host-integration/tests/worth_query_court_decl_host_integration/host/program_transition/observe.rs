//! Read-path and managed equality-index observation for the real Directory Home
//! DST driver.

use std::collections::{BTreeMap, BTreeSet};

use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt, WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::application_runtime_surface as identity;

use super::driver::{request_scope, Driver, LiveHome};
use super::reference_model::{
    ObservedState, ReferenceModel, RowState, PRIORITIES, ROW_IDS, V2_VALUES,
};
use super::v2_program::{DaymarkV2Program, DaymarkV2Schema};
use super::v2_queries::AllTasksV2Read;
use super::v3_program::{DaymarkV3Program, DaymarkV3Schema};
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

impl Driver {
    pub fn observe(&mut self, model: &ReferenceModel) -> Result<ObservedState, String> {
        let (home_open, open_version) = match &self.live {
            LiveHome::Closed => (false, 0),
            LiveHome::V2(home) => {
                self.last_observed_rows = observe_v2_rows(home)?;
                (true, 2)
            }
            LiveHome::V3(home) => {
                self.last_observed_rows = observe_v3_rows_and_verify_index(home, model)?;
                (true, 3)
            }
        };
        Ok(ObservedState {
            disk_version: self.disk_version,
            home_open,
            open_version,
            index_state: self.index_state,
            transition_runs: self.transition_runs,
            outcome: self.last_outcome,
            rows: self.last_observed_rows.clone(),
        })
    }
}

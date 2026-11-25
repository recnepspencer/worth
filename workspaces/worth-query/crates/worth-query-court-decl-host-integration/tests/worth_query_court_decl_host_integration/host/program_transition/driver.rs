//! Real WORTH Directory Home execution driver for the 64-seed program
//! transition DST.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
    },
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    application_installation::ApplicationBuildIdentity,
    program_transition::{
        open_directory_with_transition, worth_query_default_build_id,
        WorthQueryProgramTransitionOpenConfig, WorthQueryProgramTransitionOpenDenial,
        WorthQueryTransitionOpenOutcome,
    },
    runtime::ExecutionAllocationPolicy,
    WorthQueryApplicationHome, WorthQueryHomeProgramTransitionDenial,
    WorthQueryHomeProgramTransitionOpenPhase,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};
use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;

use super::reference_model::{Op, RowState, ROW_IDS};
use super::v2_mutations::{CreateTaskV2, TaskV2Input, UpdateTaskV2};
use super::v2_program::{v2_request, DaymarkV2Program, DaymarkV2Schema};
use super::v3_mutations::{CreateTaskV3, TaskV3Input, UpdateTaskV3};
use super::v3_program::{v3_request, DaymarkV2ToV3, DaymarkV3Program, DaymarkV3Schema};

pub enum LiveHome {
    Closed,
    V2(Box<WorthQueryApplicationHome<DaymarkV2Schema, DaymarkV2Program>>),
    V3(Box<WorthQueryApplicationHome<DaymarkV3Schema, DaymarkV3Program>>),
}

pub struct Driver {
    pub(super) location: PathBuf,
    pub(super) build: ApplicationBuildIdentity,
    pub(super) live: LiveHome,
    pub(super) transition_runs: usize,
    pub(super) last_outcome: &'static str,
    pub(super) last_observed_rows: BTreeMap<&'static str, RowState>,
    next_idempotency: u64,
}

pub fn request_scope() -> WorthQueryRequestScope {
    let cancel = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancel.token(),
        WorthQueryAdmissionClock::system(),
    )
}

impl Driver {
    pub fn new(seed: u64) -> Self {
        let mut rows = BTreeMap::new();
        for id in ROW_IDS {
            rows.insert(id, RowState::default());
        }
        Self {
            location: fixture::location(),
            build: worth_query_default_build_id!(),
            live: LiveHome::Closed,
            transition_runs: 0,
            last_outcome: "Init",
            last_observed_rows: rows,
            next_idempotency: (seed + 1).wrapping_mul(0x10_000),
        }
    }

    fn next_key(&mut self) -> u64 {
        self.next_idempotency += 1;
        self.next_idempotency
    }

    pub fn execute(&mut self, op: &Op) -> Result<(), String> {
        let lease = checkpoint_capture::lease(128 * 1024 * 1024);
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        match *op {
            Op::CreateV2 => self.open_v2(execution, "CreatedV2"),
            Op::ReopenV2 => self.open_v2(execution, "ReopenedV2"),
            Op::WriteV2 { row, v2val } => self.write_v2(row, v2val),
            Op::CloseHome => self.close_home(execution),
            Op::OpenV3WithoutTransition => self.refuse_v3_without_transition(execution),
            Op::OpenV3WithTransitionBudgetStop => self.refuse_v3_budget_stop(execution),
            Op::OpenV3WithTransition => self.upgrade_v2_to_v3(execution),
            Op::OpenV2OnV3 => self.refuse_downgrade_v2_on_v3(execution),
            Op::WriteV3 {
                row,
                v2val,
                priority,
            } => self.write_v3(row, v2val, priority),
            Op::ReopenV3 => self.reopen_v3(execution),
        }
    }

    fn open_v2(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        outcome: &'static str,
    ) -> Result<(), String> {
        let req = v2_request(self.location.clone(), self.build);
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(req.open_in_advancement(&phase))
        })
        .map_err(|e| format!("bootstrap v2 failed: {e:?}"))?
        .map_err(|e| format!("open v2 refused: {e:?}"))?;
        self.live = LiveHome::V2(Box::new(home));
        self.last_outcome = outcome;
        Ok(())
    }

    fn write_v2(&mut self, row: &'static str, v2val: &'static str) -> Result<(), String> {
        let key = self.next_key();
        let present = self.last_observed_rows[row].present;
        let LiveHome::V2(home) = &self.live else {
            return Err("WriteV2 requires open v2 Home".into());
        };
        let scope = request_scope();
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let input = TaskV2Input {
            task_key: row.to_owned(),
            v2_value: v2val.to_owned(),
        };
        let committed = if present {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(UpdateTaskV2(input))
                .without_source()
                .idempotency(&key)
                .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        } else {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(CreateTaskV2(input))
                .without_source()
                .idempotency(&key)
                .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        }
        .map_err(|e| format!("WriteV2 failed: {e:?}"))?;
        if !matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ) {
            return Err(format!("WriteV2 did not commit: {committed:?}"));
        }
        self.last_outcome = "WroteV2";
        Ok(())
    }

    fn close_home(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        match std::mem::replace(&mut self.live, LiveHome::Closed) {
            LiveHome::V2(home) => (*home)
                .close(&execution)
                .map_err(|e| format!("close v2 refused: {e:?}"))?
                .reclaim(),
            LiveHome::V3(home) => (*home)
                .close(&execution)
                .map_err(|e| format!("close v3 refused: {e:?}"))?
                .reclaim(),
            LiveHome::Closed => return Err("CloseHome on already closed Home".into()),
        }
        self.last_outcome = "Closed";
        Ok(())
    }

    fn refuse_v3_without_transition(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        let req = v3_request(self.location.clone(), self.build);
        let refused = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(req.open_in_advancement(&phase))
        })
        .map_err(|e| format!("bootstrap v3 refusal failed: {e:?}"))?
        .err()
        .ok_or_else(|| "v3 open without transition on v2 Home must refuse".to_owned())?;
        let requirement = refused
            .transition_required()
            .ok_or_else(|| format!("expected typed transition_required(), got {refused:?}"))?;
        if requirement.recorded().selected().identity() != "daymark.program.v2"
            || requirement.requested().selected().identity() != "daymark.program.v3"
        {
            return Err("unexpected transition_required identities for v3 on v2".to_owned());
        }
        refused
            .close(&execution)
            .map_err(|e| format!("close refused v3 opening: {e:?}"))?
            .reclaim();
        self.last_outcome = "RefusedTransitionRequired";
        Ok(())
    }

    fn refuse_v3_budget_stop(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        let before = v2_request(self.location.clone(), self.build);
        let after = v3_request(self.location.clone(), self.build);
        let config = WorthQueryProgramTransitionOpenConfig::default_bounded().with_resources(
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(1),
                maximum_cold_record_slots: Some(1),
                maximum_derived_rows: Some(1),
            },
        );
        let outcome = with_bootstrap_advancement_from_request(execution, |phase| {
            open_directory_with_transition::<DaymarkV2ToV3>(&phase, before, after, config)
        })
        .map_err(|e| format!("outer bootstrap must not fail before transition boundary: {e:?}"))?;
        match outcome {
            Err(WorthQueryProgramTransitionOpenDenial::BudgetOrCancellationStopped { refused }) => {
                if !matches!(
                    refused.phase(),
                    WorthQueryHomeProgramTransitionOpenPhase::Transition {
                        cause: WorthQueryHomeProgramTransitionDenial::ColdSlotBudgetExceeded(_)
                            | WorthQueryHomeProgramTransitionDenial::Indexes(_),
                        ..
                    }
                ) {
                    return Err(format!(
                        "expected pre-intent Transition cold-slot/index budget stop, got {:?}",
                        refused.phase()
                    ));
                }
            }
            Err(other) => {
                return Err(format!(
                    "expected BudgetOrCancellationStopped, got {other:?}"
                ))
            }
            Ok(_) => return Err("budget-exhausted transition must never succeed".to_owned()),
        }
        self.last_outcome = "RefusedBudgetStop";
        Ok(())
    }

    fn upgrade_v2_to_v3(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        let before = v2_request(self.location.clone(), self.build);
        let after = v3_request(self.location.clone(), self.build);
        let config = WorthQueryProgramTransitionOpenConfig::default_bounded();
        let opened = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(open_directory_with_transition::<DaymarkV2ToV3>(
                &phase, before, after, config,
            ))
        })
        .map_err(|e| format!("bootstrap upgrade failed: {e:?}"))?
        .map_err(|e| format!("v2 -> v3 upgrade refused: {e:?}"))?;

        let (home, outcome) = opened.into_parts();
        let expected_backfilled = self
            .last_observed_rows
            .values()
            .filter(|r| r.present)
            .count();
        match outcome {
            WorthQueryTransitionOpenOutcome::UpgradedInPlace {
                from_version: 2,
                to_version: 3,
                backfilled_records,
                indexes_marked_for_rebuild,
            } if backfilled_records == expected_backfilled
                && indexes_marked_for_rebuild.len() == 1 => {}
            other => {
                return Err(format!(
                    "unexpected upgrade outcome (expected {expected_backfilled} backfilled, 1 index): {other:?}"
                ));
            }
        }
        self.live = LiveHome::V3(Box::new(home));
        self.transition_runs += 1;
        self.last_outcome = "UpgradedV2ToV3";
        Ok(())
    }

    fn refuse_downgrade_v2_on_v3(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        let req = v2_request(self.location.clone(), self.build);
        let refused = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(req.open_in_advancement(&phase))
        })
        .map_err(|e| format!("bootstrap downgrade check failed: {e:?}"))?
        .err()
        .ok_or_else(|| "downgrade OpenV2OnV3 must refuse".to_owned())?;
        let requirement = refused
            .transition_required()
            .ok_or_else(|| format!("expected typed transition_required(), got {refused:?}"))?;
        if requirement.recorded().selected().identity() != "daymark.program.v3"
            || requirement.requested().selected().identity() != "daymark.program.v2"
        {
            return Err("unexpected downgrade refusal identities for v2 on v3".to_owned());
        }
        refused
            .close(&execution)
            .map_err(|e| format!("close refused downgrade: {e:?}"))?
            .reclaim();
        self.last_outcome = "RefusedDowngrade";
        Ok(())
    }

    fn write_v3(
        &mut self,
        row: &'static str,
        v2val: &'static str,
        priority: &'static str,
    ) -> Result<(), String> {
        let key = self.next_key();
        let present = self.last_observed_rows[row].present;
        let LiveHome::V3(home) = &self.live else {
            return Err("WriteV3 requires open v3 Home".into());
        };
        let scope = request_scope();
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let input = TaskV3Input {
            task_key: row.to_owned(),
            v2_value: v2val.to_owned(),
            priority: priority.to_owned(),
        };
        let committed = if present {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(UpdateTaskV3(input))
                .without_source()
                .idempotency(&key)
                .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        } else {
            runtime
                .request(&principal, &scope)
                .advanced()
                .mutate(CreateTaskV3(input))
                .without_source()
                .idempotency(&key)
                .execute_in_program(runtime, ExecutionAllocationPolicy::SystemAllocation)
        }
        .map_err(|e| format!("WriteV3 failed: {e:?}"))?;
        if !matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ) {
            return Err(format!("WriteV3 did not commit: {committed:?}"));
        }
        self.last_outcome = "WroteV3";
        Ok(())
    }

    fn reopen_v3(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), String> {
        let before = v2_request(self.location.clone(), self.build);
        let after = v3_request(self.location.clone(), self.build);
        let config = WorthQueryProgramTransitionOpenConfig::default_bounded();
        let opened = with_bootstrap_advancement_from_request(execution, |phase| {
            Box::new(open_directory_with_transition::<DaymarkV2ToV3>(
                &phase, before, after, config,
            ))
        })
        .map_err(|e| format!("bootstrap reopen v3 failed: {e:?}"))?
        .map_err(|e| format!("reopen v3 refused: {e:?}"))?;
        let (home, outcome) = opened.into_parts();
        if outcome != (WorthQueryTransitionOpenOutcome::OpenedCurrent { program_version: 3 }) {
            return Err(format!(
                "ReopenV3 must report OpenedCurrent without re-running transition, got {outcome:?}"
            ));
        }
        self.live = LiveHome::V3(Box::new(home));
        self.last_outcome = "ReopenedV3";
        Ok(())
    }
}

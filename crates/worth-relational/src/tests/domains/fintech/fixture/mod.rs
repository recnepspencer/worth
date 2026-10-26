mod casebook;
mod entity_seeding;
mod relation_seeding;
mod schema_declarations;
mod seed_catalog;

use crate::facade::config::RelationalRuntimeProfile;
use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, PartitionId, RelationId};
use crate::facade::runtime::{RelationalReadView, RelationalRuntime, RelationalRuntimeApi};
use crate::facade::snapshots::SnapshotHandle;
use crate::facade::transactions::RecordRef;
use crate::query::data::PlannedQueryPacket;

use self::casebook::{build_casebook, build_workflow_cases};
use self::entity_seeding::seed_entities;
use self::relation_seeding::seed_relations;
use self::schema_declarations::fintech_schema_registry;
use self::seed_catalog::seeded_trade_cases;
use super::scales::FintechScale;

pub(super) const LEDGER_PARTITION: PartitionId = PartitionId(10);
pub(super) const MARKET_PARTITION: PartitionId = PartitionId(20);
pub(super) const RISK_PARTITION: PartitionId = PartitionId(30);

#[derive(Debug)]
pub(super) struct LedgerWorld {
    pub(super) desks: Vec<EntityId>,
    pub(super) books: Vec<EntityId>,
    pub(super) accounts: Vec<EntityId>,
    pub(super) counterparties: Vec<EntityId>,
    pub(super) trades: Vec<EntityId>,
    pub(super) settlements: Vec<EntityId>,
    pub(super) cash_events: Vec<EntityId>,
    pub(super) audit_records: Vec<EntityId>,
}

#[derive(Debug)]
pub(super) struct MarketWorld {
    pub(super) instruments: Vec<EntityId>,
    pub(super) market_points: Vec<EntityId>,
}

#[derive(Debug)]
pub(super) struct RiskWorld {
    pub(super) risk_views: Vec<EntityId>,
    pub(super) limits: Vec<EntityId>,
    pub(super) breaches: Vec<EntityId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FintechCaseRole {
    BaselinePortfolio,
    LateTradeCorrection,
    IntradayRisk,
    FailedSettlementRepair,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct FintechWorkflowCase {
    pub(super) role: FintechCaseRole,
    pub(super) desk: EntityId,
    pub(super) book: EntityId,
    pub(super) account: EntityId,
    pub(super) counterparty: EntityId,
    pub(super) trade: EntityId,
    pub(super) instrument: EntityId,
    pub(super) market_point: EntityId,
    pub(super) risk_view: EntityId,
    pub(super) settlement: EntityId,
    pub(super) cash_event: EntityId,
    pub(super) limit: EntityId,
    pub(super) breach: EntityId,
    pub(super) audit_record: EntityId,
}

#[derive(Debug)]
pub(super) struct FintechCasebook {
    pub(super) baseline_portfolio: FintechWorkflowCase,
    pub(super) late_trade_correction: FintechWorkflowCase,
    pub(super) intraday_risk: FintechWorkflowCase,
    pub(super) failed_settlement_repair: FintechWorkflowCase,
}

#[derive(Debug)]
pub(crate) struct FintechWorld {
    pub(crate) runtime: RelationalRuntime,
    pub(super) ledger: LedgerWorld,
    pub(super) market: MarketWorld,
    pub(super) risk: RiskWorld,
    pub(super) cases: FintechCasebook,
    pub(super) relations: Vec<RelationId>,
}

impl FintechWorld {
    pub(super) fn setup_world(execution: &worth_execution::ExecutionRequest<'_, '_>) -> Self {
        Self::setup_world_with(execution, FintechScale::smoke())
    }

    pub(super) fn setup_world_with(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scale: FintechScale,
    ) -> Self {
        Self::build(execution, scale, false)
    }

    pub(super) fn setup_persisted_world(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Self {
        Self::build(execution, FintechScale::smoke(), true)
    }

    pub(super) fn latest_snapshot(&self) -> SnapshotHandle {
        self.runtime
            .publication()
            .latest_bundle()
            .unwrap()
            .snapshot
            .clone()
    }

    pub(super) fn read_latest(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> RelationalReadView {
        self.runtime
            .read_truth()
            .read_snapshot(&self.latest_snapshot(), execution)
            .expect("read request remains live")
            .unwrap()
    }

    pub(super) fn read_query(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        packet: PlannedQueryPacket,
    ) -> crate::query::data::CanonicalQueryResult {
        self.runtime
            .read_truth()
            .execute_query_plan(
                self.runtime
                    .read_truth()
                    .plan_query_packet(snapshot, packet)
                    .expect("planned fintech query"),
                execution,
            )
            .expect("request remains live")
            .map(|read| read.outcome)
            .expect("executed fintech query")
            .result
    }

    fn explicit_probe_packet(
        &self,
        snapshot: &SnapshotHandle,
        label: &str,
        targets: Vec<RecordRef>,
    ) -> PlannedQueryPacket {
        let context = self
            .runtime
            .read_truth()
            .query_plan_context(snapshot)
            .expect("query plan context");
        PlannedQueryPacket::explicit_targets(label, context, targets)
    }

    pub(super) fn packet_for_portfolio_probe(
        &self,
        snapshot: &SnapshotHandle,
    ) -> PlannedQueryPacket {
        let case = self.baseline_portfolio_case();
        self.explicit_probe_packet(
            snapshot,
            "portfolio-check",
            vec![
                RecordRef::Entity(case.account),
                RecordRef::Entity(case.instrument),
                RecordRef::Entity(case.risk_view),
            ],
        )
    }

    pub(super) fn packet_for_case_probe(
        &self,
        role: FintechCaseRole,
        snapshot: &SnapshotHandle,
    ) -> PlannedQueryPacket {
        match role {
            FintechCaseRole::BaselinePortfolio => self.packet_for_portfolio_probe(snapshot),
            FintechCaseRole::LateTradeCorrection => self.packet_for_correction_probe(snapshot),
            FintechCaseRole::IntradayRisk => self.packet_for_intraday_risk_probe(snapshot),
            FintechCaseRole::FailedSettlementRepair => {
                self.packet_for_settlement_repair_probe(snapshot)
            }
        }
    }

    pub(super) fn packet_for_correction_probe(
        &self,
        snapshot: &SnapshotHandle,
    ) -> PlannedQueryPacket {
        let case = self.late_trade_correction_case();
        self.explicit_probe_packet(
            snapshot,
            "correction-probe",
            vec![
                RecordRef::Entity(case.trade),
                RecordRef::Entity(case.account),
                RecordRef::Entity(case.audit_record),
            ],
        )
    }

    pub(super) fn packet_for_intraday_risk_probe(
        &self,
        snapshot: &SnapshotHandle,
    ) -> PlannedQueryPacket {
        let case = self.intraday_risk_case();
        self.explicit_probe_packet(
            snapshot,
            "intraday-risk-probe",
            vec![
                RecordRef::Entity(case.instrument),
                RecordRef::Entity(case.risk_view),
                RecordRef::Entity(case.limit),
                RecordRef::Entity(case.breach),
            ],
        )
    }

    pub(super) fn packet_for_settlement_repair_probe(
        &self,
        snapshot: &SnapshotHandle,
    ) -> PlannedQueryPacket {
        let case = self.failed_settlement_repair_case();
        self.explicit_probe_packet(
            snapshot,
            "settlement-repair-probe",
            vec![
                RecordRef::Entity(case.settlement),
                RecordRef::Entity(case.cash_event),
                RecordRef::Entity(case.trade),
                RecordRef::Entity(case.audit_record),
            ],
        )
    }

    pub(super) fn baseline_portfolio_case(&self) -> FintechWorkflowCase {
        self.cases.baseline_portfolio
    }

    pub(super) fn late_trade_correction_case(&self) -> FintechWorkflowCase {
        self.cases.late_trade_correction
    }

    pub(super) fn intraday_risk_case(&self) -> FintechWorkflowCase {
        self.cases.intraday_risk
    }

    pub(super) fn failed_settlement_repair_case(&self) -> FintechWorkflowCase {
        self.cases.failed_settlement_repair
    }

    pub(super) fn workflow_case(&self, role: FintechCaseRole) -> FintechWorkflowCase {
        match role {
            FintechCaseRole::BaselinePortfolio => self.baseline_portfolio_case(),
            FintechCaseRole::LateTradeCorrection => self.late_trade_correction_case(),
            FintechCaseRole::IntradayRisk => self.intraday_risk_case(),
            FintechCaseRole::FailedSettlementRepair => self.failed_settlement_repair_case(),
        }
    }

    pub(super) fn create_analysis_branch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> BranchId {
        create_analysis_branch(execution, &self.runtime)
    }

    fn build(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scale: FintechScale,
        persisted: bool,
    ) -> Self {
        let builder = RelationalRuntimeApi::builder()
            .profile(RelationalRuntimeProfile::AiWorkflow)
            .schema_registry(fintech_schema_registry());
        let runtime = if persisted {
            builder.build_durable(crate::tests::support::TestDurableMedium::new().attachment())
        } else {
            builder.build()
        };
        let case_seeds = seeded_trade_cases(scale);
        let seeded = seed_entities(execution, &runtime, &case_seeds);
        let workflow_cases = build_workflow_cases(&case_seeds, &seeded);
        let relations = seed_relations(execution, &runtime, &case_seeds, &seeded, &workflow_cases);
        let casebook = build_casebook(&workflow_cases);

        Self {
            runtime,
            ledger: LedgerWorld {
                desks: seeded.desks,
                books: seeded.books,
                accounts: seeded.accounts,
                counterparties: seeded.counterparties,
                trades: seeded.trades,
                settlements: seeded.settlements,
                cash_events: seeded.cash_events,
                audit_records: seeded.audit_records,
            },
            market: MarketWorld {
                instruments: seeded.instruments,
                market_points: seeded.market_points,
            },
            risk: RiskWorld {
                risk_views: seeded.risk_views,
                limits: seeded.limits,
                breaches: seeded.breaches,
            },
            cases: casebook,
            relations,
        }
    }
}

pub(super) fn create_analysis_branch(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> BranchId {
    let branch = BranchId("analysis".to_string());
    runtime
        .history_authority()
        .fork_branch_from(execution, branch.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch
}

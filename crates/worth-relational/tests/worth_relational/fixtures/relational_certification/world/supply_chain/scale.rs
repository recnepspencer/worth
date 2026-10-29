pub(crate) use super::scenarios::BaselineName;
pub(crate) use super::schema::SchemaVersion;
pub(crate) use super::semantic_key::{Anchor, EntityKey};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ScaleName {
    Court,
    Standard,
    Scale,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CostDimension {
    DeltaSteps,
    TraceSteps,
    Observations,
    CargoLots,
    SetupEntities,
    SetupRelations,
    OracleSteps,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CostBudgetError {
    ProfileMismatch {
        expected: ScaleName,
        observed: ScaleName,
    },
    SeedMismatch {
        expected: u64,
        observed: u64,
    },
    BaselineMismatch {
        expected: BaselineName,
        observed: BaselineName,
    },
    SchemaMismatch {
        expected: SchemaVersion,
        observed: SchemaVersion,
    },
    Exceeded {
        profile: ScaleName,
        dimension: CostDimension,
        observed: usize,
        maximum: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScaleBudget {
    pub(crate) max_delta_steps: usize,
    pub(crate) max_trace_steps: usize,
    pub(crate) max_observations: usize,
    pub(crate) max_cargo_lots: usize,
    pub(crate) max_setup_entities: usize,
    pub(crate) max_setup_relations: usize,
    pub(crate) max_oracle_steps: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainCostReport {
    pub(crate) profile: ScaleName,
    pub(crate) seed: u64,
    pub(crate) baseline: BaselineName,
    pub(crate) schema: SchemaVersion,
    pub(crate) setup_entities: usize,
    pub(crate) setup_relations: usize,
    pub(crate) delta_steps: usize,
    pub(crate) trace_steps: usize,
    pub(crate) oracle_steps: usize,
    pub(crate) observations: usize,
    pub(crate) cargo_lots: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainCostInputs {
    pub(crate) baseline: BaselineName,
    pub(crate) schema: SchemaVersion,
    pub(crate) setup_entities: usize,
    pub(crate) setup_relations: usize,
    pub(crate) delta_steps: usize,
    pub(crate) trace_steps: usize,
    pub(crate) oracle_steps: usize,
    pub(crate) observations: usize,
    pub(crate) cargo_lots: usize,
}

impl SupplyChainCostReport {
    pub(crate) fn machine_report(&self) -> String {
        format!(
            "profile={:?};seed={};baseline={:?};schema={:?};setup_entities={};setup_relations={};delta_steps={};trace_steps={};oracle_steps={};observations={};cargo_lots={}",
            self.profile,
            self.seed,
            self.baseline,
            self.schema,
            self.setup_entities,
            self.setup_relations,
            self.delta_steps,
            self.trace_steps,
            self.oracle_steps,
            self.observations,
            self.cargo_lots
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SupplyChainScale {
    pub(crate) name: ScaleName,
    pub(crate) seed: u64,
    pub(crate) regions: usize,
    pub(crate) ports: usize,
    pub(crate) terminals: usize,
    pub(crate) berths: usize,
    pub(crate) vessels: usize,
    pub(crate) voyages: usize,
    pub(crate) port_calls: usize,
    pub(crate) cargo_lots: usize,
    pub(crate) budget: ScaleBudget,
}

impl SupplyChainScale {
    pub(crate) const fn court() -> Self {
        Self {
            name: ScaleName::Court,
            seed: 9_171,
            regions: 2,
            ports: 4,
            terminals: 8,
            berths: 16,
            vessels: 12,
            voyages: 16,
            port_calls: 48,
            cargo_lots: 128,
            budget: ScaleBudget {
                max_delta_steps: 8,
                max_trace_steps: 128,
                max_observations: 512,
                max_cargo_lots: 128,
                max_setup_entities: 512,
                max_setup_relations: 512,
                max_oracle_steps: 128,
            },
        }
    }

    pub(crate) const fn standard() -> Self {
        Self {
            name: ScaleName::Standard,
            seed: 9_172,
            regions: 4,
            ports: 16,
            terminals: 32,
            berths: 64,
            vessels: 64,
            voyages: 128,
            port_calls: 384,
            cargo_lots: 4_096,
            budget: ScaleBudget {
                max_delta_steps: 128,
                max_trace_steps: 1_024,
                max_observations: 8_192,
                max_cargo_lots: 4_096,
                max_setup_entities: 8_192,
                max_setup_relations: 4_096,
                max_oracle_steps: 1_024,
            },
        }
    }

    pub(crate) const fn scale() -> Self {
        Self {
            name: ScaleName::Scale,
            seed: 9_173,
            regions: 8,
            ports: 64,
            terminals: 128,
            berths: 256,
            vessels: 256,
            voyages: 512,
            port_calls: 1_536,
            cargo_lots: 65_536,
            budget: ScaleBudget {
                max_delta_steps: 512,
                max_trace_steps: 4_096,
                max_observations: 70_000,
                max_cargo_lots: 65_536,
                max_setup_entities: 70_000,
                max_setup_relations: 40_000,
                max_oracle_steps: 4_096,
            },
        }
    }

    pub(crate) const fn anchors(self) -> [Anchor; 14] {
        [
            Anchor::Meridian,
            Anchor::Southpoint,
            Anchor::MeridianContainer,
            Anchor::SouthpointContainer,
            Anchor::Atlas,
            Anchor::Beacon,
            Anchor::SouthpointBerth,
            Anchor::Aurora,
            Anchor::AuroraEastbound,
            Anchor::AuroraMeridian,
            Anchor::AuroraSouthpoint,
            Anchor::MedicalSupplies,
            Anchor::MachineParts,
            Anchor::AuroraArrival,
        ]
    }

    pub(crate) const fn count_for(self, key: EntityKey) -> usize {
        match key.kind {
            super::semantic_key::EntityKind::Port => self.ports,
            super::semantic_key::EntityKind::Terminal => self.terminals,
            super::semantic_key::EntityKind::Berth => self.berths,
            super::semantic_key::EntityKind::Vessel => self.vessels,
            super::semantic_key::EntityKind::Voyage => self.voyages,
            super::semantic_key::EntityKind::PortCall => self.port_calls,
            super::semantic_key::EntityKind::CargoLot => self.cargo_lots,
            super::semantic_key::EntityKind::Inspection => self.vessels,
        }
    }

    pub(crate) fn seeded(self, ordinal: usize) -> u64 {
        let mut value = self.seed.wrapping_add(ordinal as u64 * 0x9E37_79B9);
        value ^= value >> 30;
        value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value ^ (value >> 27)
    }

    pub(crate) fn region_index(self, ordinal: usize) -> usize {
        ordinal % self.regions
    }

    pub(crate) fn cost_report(self, inputs: SupplyChainCostInputs) -> SupplyChainCostReport {
        SupplyChainCostReport {
            profile: self.name,
            seed: self.seed,
            baseline: inputs.baseline,
            schema: inputs.schema,
            setup_entities: inputs.setup_entities,
            setup_relations: inputs.setup_relations,
            delta_steps: inputs.delta_steps,
            trace_steps: inputs.trace_steps,
            oracle_steps: inputs.oracle_steps,
            observations: inputs.observations,
            cargo_lots: inputs.cargo_lots,
        }
    }

    pub(crate) fn enforce_budget(
        self,
        report: &SupplyChainCostReport,
        expected_baseline: BaselineName,
        expected_schema: SchemaVersion,
    ) -> Result<(), CostBudgetError> {
        if report.profile != self.name {
            return Err(CostBudgetError::ProfileMismatch {
                expected: self.name,
                observed: report.profile,
            });
        }
        if report.seed != self.seed {
            return Err(CostBudgetError::SeedMismatch {
                expected: self.seed,
                observed: report.seed,
            });
        }
        if report.baseline != expected_baseline {
            return Err(CostBudgetError::BaselineMismatch {
                expected: expected_baseline,
                observed: report.baseline,
            });
        }
        if report.schema != expected_schema {
            return Err(CostBudgetError::SchemaMismatch {
                expected: expected_schema,
                observed: report.schema,
            });
        }
        let limits = [
            (
                CostDimension::DeltaSteps,
                report.delta_steps,
                self.budget.max_delta_steps,
            ),
            (
                CostDimension::TraceSteps,
                report.trace_steps,
                self.budget.max_trace_steps,
            ),
            (
                CostDimension::Observations,
                report.observations,
                self.budget.max_observations,
            ),
            (
                CostDimension::CargoLots,
                report.cargo_lots,
                self.budget.max_cargo_lots,
            ),
            (
                CostDimension::SetupEntities,
                report.setup_entities,
                self.budget.max_setup_entities,
            ),
            (
                CostDimension::SetupRelations,
                report.setup_relations,
                self.budget.max_setup_relations,
            ),
            (
                CostDimension::OracleSteps,
                report.oracle_steps,
                self.budget.max_oracle_steps,
            ),
        ];
        limits
            .into_iter()
            .find(|(_, observed, maximum)| observed > maximum)
            .map_or(Ok(()), |(dimension, observed, maximum)| {
                Err(CostBudgetError::Exceeded {
                    profile: self.name,
                    dimension,
                    observed,
                    maximum,
                })
            })
    }
}

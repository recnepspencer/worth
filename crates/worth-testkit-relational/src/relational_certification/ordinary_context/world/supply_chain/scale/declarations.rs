pub use crate::relational_certification::ordinary_context::world::supply_chain::scenarios::BaselineName;
pub use crate::relational_certification::ordinary_context::world::supply_chain::schema::SchemaVersion;
pub use {
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor,
    crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKey,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScaleName {
    Court,
    Standard,
    Scale,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CostDimension {
    DeltaSteps,
    TraceSteps,
    Observations,
    CargoLots,
    SetupEntities,
    SetupRelations,
    OracleSteps,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CostBudgetError {
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
pub struct ScaleBudget {
    pub max_delta_steps: usize,
    pub max_trace_steps: usize,
    pub max_observations: usize,
    pub max_cargo_lots: usize,
    pub max_setup_entities: usize,
    pub max_setup_relations: usize,
    pub max_oracle_steps: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupplyChainCostReport {
    pub profile: ScaleName,
    pub seed: u64,
    pub baseline: BaselineName,
    pub schema: SchemaVersion,
    pub setup_entities: usize,
    pub setup_relations: usize,
    pub delta_steps: usize,
    pub trace_steps: usize,
    pub oracle_steps: usize,
    pub observations: usize,
    pub cargo_lots: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SupplyChainCostInputs {
    pub baseline: BaselineName,
    pub schema: SchemaVersion,
    pub setup_entities: usize,
    pub setup_relations: usize,
    pub delta_steps: usize,
    pub trace_steps: usize,
    pub oracle_steps: usize,
    pub observations: usize,
    pub cargo_lots: usize,
}

impl SupplyChainCostReport {
    pub fn machine_report(&self) -> String {
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
pub struct SupplyChainScale {
    pub name: ScaleName,
    pub seed: u64,
    pub regions: usize,
    pub ports: usize,
    pub terminals: usize,
    pub berths: usize,
    pub vessels: usize,
    pub voyages: usize,
    pub port_calls: usize,
    pub cargo_lots: usize,
    pub budget: ScaleBudget,
}

impl SupplyChainScale {
    pub const fn court() -> Self {
        Self {
            name: crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Court,
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

    pub const fn standard() -> Self {
        Self {
            name: crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Standard,
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

    pub const fn scale() -> Self {
        Self {
            name: crate::relational_certification::ordinary_context::world::supply_chain::scale::ScaleName::Scale,
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

    pub const fn anchors(self) -> [Anchor; 14] {
        [
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Meridian,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Southpoint,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MeridianContainer,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointContainer,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Atlas,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Beacon,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::SouthpointBerth,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::Aurora,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraEastbound,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraMeridian,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraSouthpoint,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MedicalSupplies,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::MachineParts,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::Anchor::AuroraArrival,
        ]
    }

    pub const fn count_for(self, key: EntityKey) -> usize {
        match key.kind {
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Port => self.ports,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Terminal => self.terminals,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Berth => self.berths,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Vessel => self.vessels,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Voyage => self.voyages,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::PortCall => self.port_calls,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::CargoLot => self.cargo_lots,
            crate::relational_certification::ordinary_context::world::supply_chain::semantic_key::EntityKind::Inspection => self.vessels,
        }
    }

    pub fn seeded(self, ordinal: usize) -> u64 {
        let mut value = self.seed.wrapping_add(ordinal as u64 * 0x9E37_79B9);
        value ^= value >> 30;
        value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value ^ (value >> 27)
    }

    pub fn region_index(self, ordinal: usize) -> usize {
        ordinal % self.regions
    }

    pub fn cost_report(self, inputs: SupplyChainCostInputs) -> SupplyChainCostReport {
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

    pub fn enforce_budget(
        self,
        report: &SupplyChainCostReport,
        expected_baseline: BaselineName,
        expected_schema: SchemaVersion,
    ) -> Result<(), CostBudgetError> {
        if report.profile != self.name {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::ProfileMismatch {
                expected: self.name,
                observed: report.profile,
            });
        }
        if report.seed != self.seed {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::SeedMismatch {
                expected: self.seed,
                observed: report.seed,
            });
        }
        if report.baseline != expected_baseline {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::BaselineMismatch {
                expected: expected_baseline,
                observed: report.baseline,
            });
        }
        if report.schema != expected_schema {
            return Err(crate::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::SchemaMismatch {
                expected: expected_schema,
                observed: report.schema,
            });
        }
        let limits = [
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::DeltaSteps,
                report.delta_steps,
                self.budget.max_delta_steps,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::TraceSteps,
                report.trace_steps,
                self.budget.max_trace_steps,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::Observations,
                report.observations,
                self.budget.max_observations,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::CargoLots,
                report.cargo_lots,
                self.budget.max_cargo_lots,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupEntities,
                report.setup_entities,
                self.budget.max_setup_entities,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::SetupRelations,
                report.setup_relations,
                self.budget.max_setup_relations,
            ),
            (
                crate::relational_certification::ordinary_context::world::supply_chain::scale::CostDimension::OracleSteps,
                report.oracle_steps,
                self.budget.max_oracle_steps,
            ),
        ];
        limits
            .into_iter()
            .find(|(_, observed, maximum)| observed > maximum)
            .map_or(Ok(()), |(dimension, observed, maximum)| {
                Err(crate::relational_certification::ordinary_context::world::supply_chain::scale::CostBudgetError::Exceeded {
                    profile: self.name,
                    dimension,
                    observed,
                    maximum,
                })
            })
    }
}

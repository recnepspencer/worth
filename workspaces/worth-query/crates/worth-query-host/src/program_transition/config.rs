//! Bounded configuration for declarative Directory Home program transitions.

use worth_query_execution::facade::primary_graph::WorthQueryWorkflowDispositions;
use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;

/// Default finite per-entity-kind scan bound during declarative default backfill.
pub const DEFAULT_TRANSITION_MAX_ENTITIES_PER_KIND: usize = 10_000;

/// Default finite derived-index maintenance work budget during in-place upgrade.
pub const DEFAULT_TRANSITION_MAX_WORK_UNITS: usize = 1_000_000;

/// Default finite cold record slot bound during in-place upgrade.
pub const DEFAULT_TRANSITION_MAX_COLD_RECORD_SLOTS: usize = 100_000;

/// Default finite derived row bound during in-place upgrade.
pub const DEFAULT_TRANSITION_MAX_DERIVED_ROWS: usize = 100_000;

/// Typed refusal produced when transition open bounds are missing or zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProgramTransitionConfigDenial {
    /// Derived-index maintenance resources must all be finite and positive.
    FiniteResourcesRequired,
    /// Per-entity-kind backfill scan bound must be positive and strictly finite.
    InvalidEntityScanLimit,
}

impl std::fmt::Display for WorthQueryProgramTransitionConfigDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FiniteResourcesRequired => f.write_str(
                "program transition requires finite positive DerivedIndexMaintenanceBudget limits",
            ),
            Self::InvalidEntityScanLimit => {
                f.write_str("program transition requires a positive finite max_entities_per_kind")
            }
        }
    }
}

impl std::error::Error for WorthQueryProgramTransitionConfigDenial {}

/// Explicit bounded execution configuration for [`super::open_directory_with_transition`].
#[derive(Clone, Debug)]
pub struct WorthQueryProgramTransitionOpenConfig {
    resources: DerivedIndexMaintenanceBudget,
    max_entities_per_kind: usize,
    workflow_dispositions: Option<WorthQueryWorkflowDispositions>,
}

impl WorthQueryProgramTransitionOpenConfig {
    /// Construct a configuration with explicit finite transition bounds.
    pub fn new(resources: DerivedIndexMaintenanceBudget, max_entities_per_kind: usize) -> Self {
        Self {
            resources,
            max_entities_per_kind,
            workflow_dispositions: None,
        }
    }

    /// Construct a safe finite bounded configuration suitable for ordinary desktop apps.
    pub fn default_bounded() -> Self {
        Self::new(
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(DEFAULT_TRANSITION_MAX_WORK_UNITS),
                maximum_cold_record_slots: Some(DEFAULT_TRANSITION_MAX_COLD_RECORD_SLOTS),
                maximum_derived_rows: Some(DEFAULT_TRANSITION_MAX_DERIVED_ROWS),
            },
            DEFAULT_TRANSITION_MAX_ENTITIES_PER_KIND,
        )
    }

    pub fn with_resources(mut self, resources: DerivedIndexMaintenanceBudget) -> Self {
        self.resources = resources;
        self
    }

    pub fn with_max_entities_per_kind(mut self, max_entities_per_kind: usize) -> Self {
        self.max_entities_per_kind = max_entities_per_kind;
        self
    }

    pub fn with_workflow_dispositions(
        mut self,
        dispositions: WorthQueryWorkflowDispositions,
    ) -> Self {
        self.workflow_dispositions = Some(dispositions);
        self
    }

    pub fn resources(&self) -> DerivedIndexMaintenanceBudget {
        self.resources
    }

    pub const fn max_entities_per_kind(&self) -> usize {
        self.max_entities_per_kind
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        DerivedIndexMaintenanceBudget,
        usize,
        Option<WorthQueryWorkflowDispositions>,
    ) {
        (
            self.resources,
            self.max_entities_per_kind,
            self.workflow_dispositions,
        )
    }

    /// Fail closed before touching any filesystem or Home owner when any
    /// declared bound is absent, zero, or unbounded.
    pub fn validate(&self) -> Result<(), WorthQueryProgramTransitionConfigDenial> {
        let resources_ok = [
            self.resources.maximum_work_units,
            self.resources.maximum_cold_record_slots,
            self.resources.maximum_derived_rows,
        ]
        .into_iter()
        .all(|limit| matches!(limit, Some(1..)));
        if !resources_ok {
            return Err(WorthQueryProgramTransitionConfigDenial::FiniteResourcesRequired);
        }
        if self.max_entities_per_kind == 0 || self.max_entities_per_kind == usize::MAX {
            return Err(WorthQueryProgramTransitionConfigDenial::InvalidEntityScanLimit);
        }
        Ok(())
    }
}

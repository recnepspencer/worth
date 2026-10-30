use worth_foundational::facade::FoundationalPerformanceCounterName;

use super::{
    WorthQueryStructuralCounterAggregation, WorthQueryStructuralCounterMonotonicity,
    WorthQueryStructuralCounterReplayPosture, WorthQueryStructuralCounterRequiredness,
    WorthQueryStructuralCounterResetBoundary, WorthQueryStructuralCounterRole,
    WorthQueryStructuralCounterScope, WorthQueryStructuralCounterUnit,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryStructuralCounterSchema {
    name: FoundationalPerformanceCounterName,
    role: WorthQueryStructuralCounterRole,
    unit: WorthQueryStructuralCounterUnit,
    aggregation: WorthQueryStructuralCounterAggregation,
    monotonicity: WorthQueryStructuralCounterMonotonicity,
    scope: WorthQueryStructuralCounterScope,
    reset_boundary: WorthQueryStructuralCounterResetBoundary,
    requiredness: WorthQueryStructuralCounterRequiredness,
    replay: WorthQueryStructuralCounterReplayPosture,
}

impl WorthQueryStructuralCounterSchema {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: FoundationalPerformanceCounterName,
        role: WorthQueryStructuralCounterRole,
        unit: WorthQueryStructuralCounterUnit,
        aggregation: WorthQueryStructuralCounterAggregation,
        monotonicity: WorthQueryStructuralCounterMonotonicity,
        scope: WorthQueryStructuralCounterScope,
        reset_boundary: WorthQueryStructuralCounterResetBoundary,
        requiredness: WorthQueryStructuralCounterRequiredness,
        replay: WorthQueryStructuralCounterReplayPosture,
    ) -> Self {
        Self {
            name,
            role,
            unit,
            aggregation,
            monotonicity,
            scope,
            reset_boundary,
            requiredness,
            replay,
        }
    }

    pub fn name(&self) -> &FoundationalPerformanceCounterName {
        &self.name
    }

    pub const fn role(&self) -> WorthQueryStructuralCounterRole {
        self.role
    }

    pub fn unit(&self) -> &WorthQueryStructuralCounterUnit {
        &self.unit
    }

    pub fn aggregation(&self) -> &WorthQueryStructuralCounterAggregation {
        &self.aggregation
    }

    pub const fn monotonicity(&self) -> WorthQueryStructuralCounterMonotonicity {
        self.monotonicity
    }

    pub const fn scope(&self) -> WorthQueryStructuralCounterScope {
        self.scope
    }

    pub const fn reset_boundary(&self) -> WorthQueryStructuralCounterResetBoundary {
        self.reset_boundary
    }

    pub const fn requiredness(&self) -> WorthQueryStructuralCounterRequiredness {
        self.requiredness
    }

    pub const fn replay(&self) -> WorthQueryStructuralCounterReplayPosture {
        self.replay
    }

    pub(crate) fn canonicalize(&mut self) {
        self.aggregation.canonicalize();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryStructuralCounterContract {
    rows: Vec<WorthQueryStructuralCounterSchema>,
}

impl WorthQueryStructuralCounterContract {
    pub fn required_foundation(
        byte_counter: FoundationalPerformanceCounterName,
        element_counter: FoundationalPerformanceCounterName,
        structural_work_counter: FoundationalPerformanceCounterName,
    ) -> Self {
        Self::declare([
            foundation_schema(
                byte_counter,
                WorthQueryStructuralCounterRole::Bytes,
                WorthQueryStructuralCounterUnit::Bytes,
            ),
            foundation_schema(
                element_counter,
                WorthQueryStructuralCounterRole::Elements,
                WorthQueryStructuralCounterUnit::Elements,
            ),
            foundation_schema(
                structural_work_counter,
                WorthQueryStructuralCounterRole::StructuralWork,
                WorthQueryStructuralCounterUnit::Operations,
            ),
        ])
    }

    pub fn declare(rows: impl IntoIterator<Item = WorthQueryStructuralCounterSchema>) -> Self {
        let mut rows = rows.into_iter().collect::<Vec<_>>();
        for row in &mut rows {
            row.canonicalize();
        }
        rows.sort_by(|left, right| left.name.cmp(&right.name));
        Self { rows }
    }

    pub fn rows(&self) -> &[WorthQueryStructuralCounterSchema] {
        &self.rows
    }

    pub fn row(
        &self,
        name: &FoundationalPerformanceCounterName,
    ) -> Option<&WorthQueryStructuralCounterSchema> {
        self.rows
            .binary_search_by(|candidate| candidate.name.cmp(name))
            .ok()
            .map(|index| &self.rows[index])
    }

    pub fn byte_counter(&self) -> &FoundationalPerformanceCounterName {
        self.foundation_counter(WorthQueryStructuralCounterRole::Bytes)
    }

    pub fn element_counter(&self) -> &FoundationalPerformanceCounterName {
        self.foundation_counter(WorthQueryStructuralCounterRole::Elements)
    }

    pub fn structural_work_counter(&self) -> &FoundationalPerformanceCounterName {
        self.foundation_counter(WorthQueryStructuralCounterRole::StructuralWork)
    }

    pub(crate) fn is_valid(&self) -> bool {
        super::validation::contract_is_valid(self)
    }

    fn foundation_counter(
        &self,
        role: WorthQueryStructuralCounterRole,
    ) -> &FoundationalPerformanceCounterName {
        self.rows
            .iter()
            .find(|row| {
                row.role == role
                    && row.requiredness == WorthQueryStructuralCounterRequiredness::RequiredCore
            })
            .map(WorthQueryStructuralCounterSchema::name)
            .expect("validated artifact contracts retain required foundation counters")
    }
}

fn foundation_schema(
    name: FoundationalPerformanceCounterName,
    role: WorthQueryStructuralCounterRole,
    unit: WorthQueryStructuralCounterUnit,
) -> WorthQueryStructuralCounterSchema {
    WorthQueryStructuralCounterSchema::new(
        name,
        role,
        unit,
        WorthQueryStructuralCounterAggregation::Independent,
        WorthQueryStructuralCounterMonotonicity::NonDecreasing,
        WorthQueryStructuralCounterScope::ArtifactOccurrence,
        WorthQueryStructuralCounterResetBoundary::ArtifactOccurrence,
        WorthQueryStructuralCounterRequiredness::RequiredCore,
        WorthQueryStructuralCounterReplayPosture::Exact,
    )
}

//! Public #358: one acceptance rule for a row's restored facts and output
//! aspects. A fact or aspect holds by revision, or, for a row a graph
//! selection transported, when that row's own coordinate proves the original
//! revision at the selected step, unchanged since the restoring commit. It is
//! exactly what readmission's `is_current` accepts, so marks, the full
//! oracle and every fallback reader agree. The evidence is read lazily, only
//! after something moved; without a coordinate nothing extra is read.

use std::cell::RefCell;

use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial as Denial,
    RelationalRuntime, VisibilityProjectionView,
};
use worth_relational::facade::snapshots::SnapshotHandle;

use super::SelectionCoordinate;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::verification::work_budget::fact_is_current;
use crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::readmission_observation::record_historical_acceptance;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;

enum Evidence {
    Unread,
    /// Foreign, not a selection, a mismatched basis or an unprojectable
    /// snapshot: nothing moved is held.
    Unavailable,
    Read(Box<(RelationalHistoricalSelectionRead, VisibilityProjectionView)>),
}

pub struct RestoredAcceptance<'a, 'e, 'f> {
    execution: &'a worth_execution::ExecutionRequest<'e, 'f>,
    runtime: &'a RelationalRuntime,
    snapshot: &'a SnapshotHandle,
    coordinate: Option<&'a SelectionCoordinate>,
    evidence: RefCell<Evidence>,
}

impl<'a, 'e, 'f> RestoredAcceptance<'a, 'e, 'f> {
    pub fn new(
        execution: &'a worth_execution::ExecutionRequest<'e, 'f>,
        runtime: &'a RelationalRuntime,
        snapshot: &'a SnapshotHandle,
        coordinate: Option<&'a SelectionCoordinate>,
    ) -> Self {
        Self {
            execution,
            runtime,
            snapshot,
            coordinate,
            evidence: RefCell::new(Evidence::Unread),
        }
    }

    /// The fact is current by revision, or restored by this coordinate.
    pub fn fact_holds(&self, fact: &Fact) -> Result<bool, ConsumedOutputVerificationStop> {
        if fact_is_current(self.execution, fact, self.runtime, self.snapshot)? {
            return Ok(true);
        }
        self.held_by_evidence(|evidence, current| {
            fact.historical_revision_matches(self.execution, current, evidence)
        })
        .map_err(map_admission_stop)
    }

    /// A fact whose revision already moved still holds when restored by this
    /// coordinate.
    pub fn moved_fact_holds(&self, fact: &Fact) -> Result<bool, CompanionPreflightStop> {
        self.held_by_evidence(|evidence, current| {
            fact.historical_revision_matches(self.execution, current, evidence)
        })
    }

    /// The output is unchanged by revision, or each moved aspect is restored
    /// by this coordinate. Presence and retirement are the ordinary checks.
    pub fn witness_holds(
        &self,
        witness: &SealedNativeOutputWitness,
    ) -> Result<bool, CompanionPreflightStop> {
        if witness.unchanged_in(self.runtime, self.snapshot, self.execution)? {
            return Ok(true);
        }
        let mut stop = None;
        let held = self.held_by_evidence(|evidence, current| {
            match witness.unchanged_since_restoration(
                self.runtime,
                self.snapshot,
                self.execution,
                current,
                evidence,
            ) {
                Ok(answer) => answer,
                Err(cause) => {
                    stop = Some(cause);
                    Ok(false)
                }
            }
        })?;
        match stop {
            Some(cause) => Err(cause),
            None => Ok(held),
        }
    }

    fn held_by_evidence(
        &self,
        answer: impl FnOnce(
            &RelationalHistoricalSelectionRead,
            &VisibilityProjectionView,
        ) -> Result<bool, Denial>,
    ) -> Result<bool, CompanionPreflightStop> {
        let Some(coordinate) = self.coordinate else {
            return Ok(false);
        };
        let mut evidence = self.evidence.borrow_mut();
        if matches!(*evidence, Evidence::Unread) {
            *evidence = match coordinate.read_evidence(self.execution, self.runtime) {
                Ok(read) => match self.runtime.read_truth().project_snapshot(self.snapshot) {
                    Some(current) => Evidence::Read(Box::new((read, current))),
                    None => Evidence::Unavailable,
                },
                Err(denial) => match held(Err(denial)) {
                    Ok(_) => Evidence::Unavailable,
                    Err(stop) => return Err(stop),
                },
            };
        }
        let Evidence::Read(read) = &*evidence else {
            return Ok(false);
        };
        let held = held(answer(&read.0, &read.1))?;
        if held {
            record_historical_acceptance();
        }
        Ok(held)
    }
}

/// As readmission answers: a stop stays a stop; a foreign or mismatched
/// answer only declines.
fn held(answer: Result<bool, Denial>) -> Result<bool, CompanionPreflightStop> {
    match answer {
        Ok(answer) => Ok(answer),
        Err(Denial::Execution(cause)) => Err(CompanionPreflightStop::Execution(cause)),
        Err(Denial::Request(stop)) => Err(CompanionPreflightStop::Execution(stop.into())),
        Err(
            Denial::ForeignOwner
            | Denial::NotGraphSelection { .. }
            | Denial::SelectedBasisMismatch { .. },
        ) => Ok(false),
    }
}

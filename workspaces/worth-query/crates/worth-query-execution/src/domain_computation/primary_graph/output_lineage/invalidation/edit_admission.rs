use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::super::super::request_local::{assert_request_local, RequestLocal};
use super::admission::IndexAdmission;
use super::{index_capacity, retention, SourceInvalidationOwner};
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;

/// A caller retains this meter across its complete derived-edit operation.
/// It can also be carried by the authenticated required-set coordinator rather
/// than creating a fresh allowance for every settlement. It is request-local:
/// no kernel or concurrent compute charges it.
pub(in crate::domain_computation) struct InvalidationEditAdmission {
    budget: CompanionPreflightBudget,
    counters: AdmissionCounters,
    request_local: RequestLocal,
}

assert_request_local!(
    InvalidationEditAdmission,
    CarriedRequestInvalidationAdmission,
    ReservedExternalWork<'static>,
);

#[derive(Default)]
struct AdmissionTotals {
    work: u64,
    bytes: u64,
    navigation: u64,
    ordered_operations: u64,
}

enum AdmissionCounters {
    Local(AdmissionTotals),
    Carried(Arc<CarriedAdmissionCounters>),
}

struct CarriedAdmissionCounters {
    runtime_instance_id: u64,
    totals: Mutex<AdmissionTotals>,
    _retained: Arc<RetainedInvalidationCapacity>,
}

/// Registry-issued request custody, consumed by managed publication. It shares
/// the original spent counters and cannot mint a new remaining-based allowance.
pub(in crate::domain_computation::primary_graph) struct CarriedRequestInvalidationAdmission {
    admission: InvalidationEditAdmission,
}

/// A maximum charged to the existing carried Work counter before an external
/// operation starts. Settlement returns only unused units; forgetting this
/// custody keeps the full maximum charged.
#[must_use]
pub(in crate::domain_computation::primary_graph) struct ReservedExternalWork<'a> {
    admission: &'a mut InvalidationEditAdmission,
    reserved: u64,
}

impl ReservedExternalWork<'_> {
    /// Nested work uses the original meter while this conditional maximum is
    /// reserved. Settlement refunds only this reservation's unused share.
    pub(in crate::domain_computation::primary_graph) fn admission(
        &mut self,
    ) -> &mut InvalidationEditAdmission {
        self.admission
    }

    pub(in crate::domain_computation::primary_graph) fn settle(
        self,
        spent: u64,
    ) -> Result<(), CompanionPreflightStop> {
        let unused = self
            .reserved
            .checked_sub(spent)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        self.admission.with_totals_mut(|totals| {
            totals.work = totals
                .work
                .checked_sub(unused)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            Ok(())
        })
    }
}

impl CarriedRequestInvalidationAdmission {
    pub(in crate::domain_computation::primary_graph) fn into_admission(
        self,
    ) -> InvalidationEditAdmission {
        self.admission
    }
}

impl InvalidationEditAdmission {
    pub(in crate::domain_computation::primary_graph) fn new(
        budget: CompanionPreflightBudget,
    ) -> Self {
        Self {
            budget,
            counters: AdmissionCounters::Local(AdmissionTotals::default()),
            request_local: PhantomData,
        }
    }

    /// One request advance: framework preparation (principal capture,
    /// admitted lookup, Query readmission and validation) and the producers it
    /// runs. Its work has no aggregate ceiling because every part is already
    /// bounded where it is declared: preparation by the installed program,
    /// each producer by its own producer-work lane, and each Query read by its
    /// declared maximum. Work is still counted at every charge; bytes stay
    /// limited by the installed preparation-bytes allowance.
    pub(in crate::domain_computation::primary_graph) const fn structurally_bounded_request(
        maximum_preparation_bytes: u64,
    ) -> Self {
        Self {
            budget: CompanionPreflightBudget {
                maximum_work_visits: u64::MAX,
                maximum_preparation_bytes,
            },
            counters: AdmissionCounters::Local(AdmissionTotals {
                work: 0,
                bytes: 0,
                navigation: 0,
                ordered_operations: 0,
            }),
            request_local: PhantomData,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn charge_selected_index_read(
        &mut self,
        work: worth_relational::facade::indexes::SelectedIndexReadWork,
        bytes: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.admit_read_scratch(bytes)?;
        match work {
            worth_relational::facade::indexes::SelectedIndexReadWork::Operation(work) => {
                self.charge_external_work(work)
            }
            worth_relational::facade::indexes::SelectedIndexReadWork::OrderedNavigation(
                navigation,
            ) => self.charge_ordered_operations(1, navigation),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn charged_work(&self) -> u64 {
        self.with_totals(|totals| totals.work)
    }

    pub(in crate::domain_computation::primary_graph) fn charged_bytes(&self) -> u64 {
        self.with_totals(|totals| totals.bytes)
    }

    pub(in crate::domain_computation::primary_graph) fn remaining_work(&self) -> usize {
        usize::try_from(self.budget.maximum_work_visits - self.charged_work()).unwrap_or(usize::MAX)
    }

    pub(in crate::domain_computation::primary_graph) fn carry_for_publication(
        &mut self,
        owner: &SourceInvalidationOwner,
    ) -> Result<CarriedRequestInvalidationAdmission, CompanionPreflightStop> {
        if let AdmissionCounters::Carried(carried) = &self.counters {
            if carried.runtime_instance_id != owner.runtime_instance_id {
                return Err(CompanionPreflightStop::SelectedSourceMismatch);
            }
        }
        self.work(1)?;
        if matches!(self.counters, AdmissionCounters::Local(_)) {
            let bytes = index_capacity::arc_bytes::<CarriedAdmissionCounters>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            self.bytes(bytes)?;
            let retained = retention::reserve(&owner.resources, bytes, self)?;
            let AdmissionCounters::Local(totals) = std::mem::replace(
                &mut self.counters,
                AdmissionCounters::Local(AdmissionTotals::default()),
            ) else {
                unreachable!("the first custody transition owns local counters")
            };
            self.counters = AdmissionCounters::Carried(Arc::new(CarriedAdmissionCounters {
                runtime_instance_id: owner.runtime_instance_id,
                totals: Mutex::new(totals),
                _retained: retained,
            }));
        }
        let AdmissionCounters::Carried(carried) = &self.counters else {
            unreachable!("request counters were transferred before custody")
        };
        Ok(CarriedRequestInvalidationAdmission {
            admission: Self {
                budget: self.budget,
                counters: AdmissionCounters::Carried(Arc::clone(carried)),
                request_local: PhantomData,
            },
        })
    }

    fn with_totals<T>(&self, read: impl FnOnce(&AdmissionTotals) -> T) -> T {
        match &self.counters {
            AdmissionCounters::Local(totals) => read(totals),
            AdmissionCounters::Carried(carried) => read(
                &carried
                    .totals
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            ),
        }
    }

    fn with_totals_mut<T>(&mut self, edit: impl FnOnce(&mut AdmissionTotals) -> T) -> T {
        match &mut self.counters {
            AdmissionCounters::Local(totals) => edit(totals),
            AdmissionCounters::Carried(carried) => edit(
                &mut carried
                    .totals
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            ),
        }
    }

    /// Scratch is charged by the same installed preparation owner before a
    /// verifier grows its worklist. The allowance spans the complete closure.
    pub(in crate::domain_computation) fn admit_read_scratch(
        &mut self,
        bytes: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.bytes(bytes)
    }

    pub(in crate::domain_computation) fn charge_external_work(
        &mut self,
        units: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.work(units)
    }

    /// Reserve the Query read's admitted maximum against the same counter
    /// before it performs work. Other carried claimants see the reserved
    /// amount until this custody settles its actual spent work.
    pub(in crate::domain_computation::primary_graph) fn reserve_external_work(
        &mut self,
        maximum: u64,
    ) -> Result<ReservedExternalWork<'_>, CompanionPreflightStop> {
        self.work(maximum)?;
        Ok(ReservedExternalWork {
            admission: self,
            reserved: maximum,
        })
    }

    /// A bounded read that reports what it spent, paid from this meter. The
    /// remaining work is reserved before the read, the read stays within it,
    /// and the reservation settles at what the read spent. A read that fails
    /// keeps its reservation charged. A read that reports spending more than
    /// it was given is a bug, never a budget answer: the only stop returned
    /// is the reservation's own.
    pub(in crate::domain_computation::primary_graph) fn reserved_read<Answer, Failure>(
        &mut self,
        read: impl FnOnce(usize) -> Result<(Answer, usize), Failure>,
    ) -> Result<Result<Answer, Failure>, CompanionPreflightStop> {
        let maximum = self.remaining_work();
        let reserved = self.reserve_external_work(u64::try_from(maximum).unwrap_or(u64::MAX))?;
        match read(maximum) {
            Ok((answer, spent)) => {
                let spent = u64::try_from(spent).unwrap_or(u64::MAX);
                debug_assert!(
                    spent <= reserved.reserved,
                    "a reserved read spent {spent} of {}",
                    reserved.reserved
                );
                let spent = spent.min(reserved.reserved);
                reserved
                    .settle(spent)
                    .expect("settling within a reservation returns only its unused work");
                Ok(Ok(answer))
            }
            Err(failure) => Ok(Err(failure)),
        }
    }

    /// A closure's visited set uses the same pinned im ordered index as the
    /// reverse index. Exact settlement identities contain only fixed-width
    /// structural identities; they own no variable comparison payload.
    #[track_caller]
    pub(in crate::domain_computation::primary_graph) fn admit_visited_settlement(
        &mut self,
        entries: usize,
    ) -> Result<(), CompanionPreflightStop> {
        use super::super::RecordedSettlementIdentity;
        use std::sync::Arc;
        let comparisons = super::index_capacity::ordered_navigation_work(entries)
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        // The identity is a fixed structural key. Each candidate comparison
        // is one visit; its inline width is priced by ordered_edit bytes.
        self.work(comparisons)?;
        self.ordered_edit::<Arc<RecordedSettlementIdentity>, ()>(entries)
    }
}

mod meter;

#[cfg(test)]
mod carried_tests;

use std::sync::{Arc, Mutex};
use worth_relational::facade::mvcc::{CompanionPreflightBudget, CompanionPreflightStop};

use super::admission::IndexAdmission;
use super::{index_capacity, retention, SourceInvalidationOwner};
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;

/// A caller retains this meter across its complete derived-edit operation.
/// It can also be carried by the authenticated required-set coordinator rather
/// than creating a fresh allowance for every settlement.
pub(in crate::domain_computation) struct InvalidationEditAdmission {
    budget: CompanionPreflightBudget,
    counters: AdmissionCounters,
}

#[derive(Default)]
struct AdmissionTotals {
    work: u64,
    bytes: u64,
    navigation: u64,
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
            }),
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

    /// Physical ordered-index navigation over shared registries is admitted
    /// and reported separately from the declared logical work. Its height is
    /// bounded by the owning registry's installed record ledger, so another
    /// consumer's population never spends this request's declared allowance.
    pub(in crate::domain_computation) fn charge_navigation(
        &mut self,
        units: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.with_totals_mut(|totals| {
            totals.navigation = totals
                .navigation
                .checked_add(units)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            Ok(())
        })
    }

    /// Ordered-index operations over a shared registry: one declared unit per
    /// operation, with their height-dependent paths reported as navigation.
    pub(in crate::domain_computation) fn charge_ordered_operations(
        &mut self,
        operations: u64,
        navigation: u64,
    ) -> Result<(), CompanionPreflightStop> {
        self.work(operations)?;
        self.charge_navigation(navigation)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn charged_navigation(&self) -> u64 {
        self.with_totals(|totals| totals.navigation)
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

impl IndexAdmission for InvalidationEditAdmission {
    fn work(&mut self, visits: u64) -> Result<(), CompanionPreflightStop> {
        let maximum = self.budget.maximum_work_visits;
        self.with_totals_mut(|totals| {
            let required = totals
                .work
                .checked_add(visits)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
            if required > maximum {
                return Err(CompanionPreflightStop::WorkExhausted { required, maximum });
            }
            totals.work = required;
            Ok(())
        })
    }

    fn navigation(&mut self, units: u64) -> Result<(), CompanionPreflightStop> {
        self.charge_navigation(units)
    }

    fn bytes(&mut self, bytes: u64) -> Result<(), CompanionPreflightStop> {
        let maximum = self.budget.maximum_preparation_bytes;
        let mut current = 0;
        let result = self.with_totals_mut(|totals| {
            current = totals.bytes;
            let required = totals
                .bytes
                .checked_add(bytes)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            if required > maximum {
                return Err(CompanionPreflightStop::PreparationMemoryExhausted {
                    required,
                    maximum,
                });
            }
            totals.bytes = required;
            Ok(())
        });
        if let Err(stop) = &result {
            crate::domain_computation::primary_graph::composed_output_diagnostics::preparation_denied(
                stop, current, bytes, maximum,
            );
        }
        result
    }
}

#[cfg(test)]
mod carried_tests;

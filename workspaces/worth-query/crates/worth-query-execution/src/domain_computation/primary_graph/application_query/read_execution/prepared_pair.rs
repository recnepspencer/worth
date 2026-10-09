//! The worker carries a scope-free read capability and a closed raw binding.
//! Native reads still borrow the installed owner under its runtime lock.
pub(in crate::domain_computation::primary_graph::application_query) mod batch;

use super::super::{
    derived_view::WorthQueryManagedDerivedViewDenial as Denial,
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};
use super::{read_plan::ReadPlan, OneShotReadWorkObservation, RawNonLiveKernelOutcome};
use crate::domain_computation::provider_session::{
    WorthQueryManagedGraphReadDenial, WorthQueryPreparedReadCompletion,
    WorthQueryPreparedSessionRead,
};
use std::cell::{Cell, RefCell};
use worth_execution::{ChargedBytes, MapKernelContext, MapKernelFailure, MapKernelStop};
use worth_foundational::facade::AspectValue;
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph::application_query) struct PreparedRead<'a> {
    pub(in crate::domain_computation::primary_graph::application_query) plan: ReadPlan<'a>,
    pub(in crate::domain_computation::primary_graph::application_query) session:
        WorthQueryPreparedSessionRead<'a>,
    pub(in crate::domain_computation::primary_graph::application_query) buffer:
        WorthQueryApplicationResultBufferReservation,
    pub(in crate::domain_computation::primary_graph::application_query) batch:
        Option<batch::PreparedBatchRead>,
}

/// Construction remains with the owner-side admission stage.
pub(in crate::domain_computation::primary_graph::application_query) struct PreparedPair<'a> {
    pub(in crate::domain_computation::primary_graph::application_query) root: EntityId,
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph::application_query) witness:
        Option<std::sync::Arc<super::dispatch_witness::DispatchWitness>>,
    pub(in crate::domain_computation::primary_graph::application_query) first: PreparedRead<'a>,
    pub(in crate::domain_computation::primary_graph::application_query) second: PreparedRead<'a>,
    pub(in crate::domain_computation::primary_graph::application_query) binding_slot: &'a str,
    pub(in crate::domain_computation::primary_graph::application_query) binding_value:
        &'a AspectValue,
}

pub(in crate::domain_computation::primary_graph::application_query) struct ReadOutput {
    pub(in crate::domain_computation::primary_graph::application_query) raw:
        RawNonLiveKernelOutcome,
    pub(in crate::domain_computation::primary_graph::application_query) proof:
        WorthQueryPreparedReadCompletion,
    pub(in crate::domain_computation::primary_graph::application_query) batch_rows:
        Option<super::super::WorthQueryApplicationQueryBatchMemory>,
}

pub(in crate::domain_computation::primary_graph::application_query) struct PairOutput {
    pub(in crate::domain_computation::primary_graph::application_query) first: ReadOutput,
    pub(in crate::domain_computation::primary_graph::application_query) second: ReadOutput,
}

impl ChargedBytes for PreparedRead<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            plan,
            session,
            buffer,
            batch: _,
        } = self;
        plan.additional_charged_bytes()
            .saturating_add(session.additional_charged_bytes())
            .saturating_add(buffer.additional_charged_bytes())
    }
}
impl ChargedBytes for PreparedPair<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        // Root/binding are inline or borrowed. The test witness is shared with
        // its test-thread owner and is not a production allocation.
        let Self {
            root: _root,
            first,
            second,
            binding_slot: _binding_slot,
            binding_value: _binding_value,
            #[cfg(test)]
                witness: _witness,
        } = self;
        first
            .additional_charged_bytes()
            .saturating_add(second.additional_charged_bytes())
    }
}
impl ChargedBytes for ReadOutput {
    fn additional_charged_bytes(&self) -> u64 {
        let Self {
            raw,
            proof,
            batch_rows: _,
        } = self;
        raw.additional_charged_bytes()
            .saturating_add(proof.additional_charged_bytes())
    }
}
impl ChargedBytes for PairOutput {
    fn additional_charged_bytes(&self) -> u64 {
        let Self { first, second } = self;
        first
            .additional_charged_bytes()
            .saturating_add(second.additional_charged_bytes())
    }
}

impl PreparedPair<'_> {
    pub(in crate::domain_computation::primary_graph::application_query) fn run(
        self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<PairOutput, MapKernelFailure<Denial>> {
        #[cfg(test)]
        if let Some(witness) = &self.witness {
            witness.enter(self.root);
        }
        #[cfg(test)]
        let root = self.root;
        #[cfg(test)]
        let witness = self.witness.clone();
        let outcome = self.compute(context);
        #[cfg(test)]
        if let Some(witness) = &witness {
            witness.complete(root);
        }
        outcome
    }
    fn compute(
        self,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<PairOutput, MapKernelFailure<Denial>> {
        context.checkpoint(1)?;
        #[cfg(test)]
        if let Some(witness) = &self.witness {
            witness.charged(self.root, 1);
        }
        let first = self.first.run(
            self.root,
            context,
            #[cfg(test)]
            self.witness.as_deref(),
        )?;
        let [row] = first.raw.raw.rows.raw_rows() else {
            return Err(MapKernelFailure::Domain(Denial::IncompleteDependencies));
        };
        if row.entity_id() != self.root
            || row.field(self.binding_slot).map(|field| field.value()) != Some(self.binding_value)
        {
            return Err(MapKernelFailure::Domain(Denial::IncompleteDependencies));
        }
        #[cfg(test)]
        if let Some(witness) = &self.witness {
            witness.before_second(self.root);
        }
        let second = self.second.run(
            self.root,
            context,
            #[cfg(test)]
            self.witness.as_deref(),
        )?;
        Ok(PairOutput { first, second })
    }
}

impl PreparedRead<'_> {
    fn run(
        self,
        root: EntityId,
        context: &mut MapKernelContext<'_, '_>,
        #[cfg(test)] witness: Option<&super::dispatch_witness::DispatchWitness>,
    ) -> Result<ReadOutput, MapKernelFailure<Denial>> {
        let Self {
            plan,
            session,
            buffer,
            batch,
        } = self;
        let entered = Cell::new(false);
        let maximum = batch
            .as_ref()
            .map_or(plan.maximum_work, batch::PreparedBatchRead::maximum);
        let stop = Cell::new(None);
        let meter = RefCell::new(context);
        let charge = |units: usize, subject: &str| {
            let result = u64::try_from(units)
                .map_err(|_| MapKernelStop::WorkCounterOverflow)
                .and_then(|units| meter.borrow_mut().checkpoint(units));
            #[cfg(test)]
            if result.is_ok() {
                if let Some(witness) = witness {
                    witness.charged(root, units as u64);
                }
            }
            result.map_err(|cause| {
                stop.set(Some(cause));
                super::read_execution_denial(
                    super::WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                    subject,
                )
            })
        };
        let spent = OneShotReadWorkObservation::with_charge(&charge);
        let check = |subject: &str| charge(0, subject);
        let read = session.execute(|runtime, graph| {
            entered.set(true);
            super::read_prepared_root_rows(
                runtime,
                graph,
                &plan,
                buffer,
                Some(&spent),
                maximum,
                super::ReadInterruption::Execution(&check),
            )
        });
        let batch_rows = if let Some(batch) = batch {
            let actual = if entered.get() {
                Some(
                    spent
                        .read()
                        .ok_or(MapKernelFailure::Domain(Denial::WorkCounterOverflow))?,
                )
            } else {
                None
            };
            batch.settle(actual).map_err(|denial| {
                MapKernelFailure::Domain(Denial::BatchResource { root, denial })
            })?
        } else {
            None
        };
        if let Some(stop) = stop.get() {
            return Err(MapKernelFailure::Stop(stop));
        }
        use super::super::authorized_read::WorthQueryAuthorizedApplicationReadDenial as ReadDenial;
        use super::super::one_shot::map_authorized_read_denial;
        match read {
            Ok((Ok(raw), proof)) => Ok(ReadOutput {
                raw,
                proof,
                batch_rows,
            }),
            Ok((Err(read), _)) => Err(MapKernelFailure::Domain(Denial::ReadDenied {
                root,
                denial: map_authorized_read_denial(ReadDenial::Read(read), plan.name),
            })),
            Err(
                WorthQueryManagedGraphReadDenial::MutationSession
                | WorthQueryManagedGraphReadDenial::ForeignBasis
                | WorthQueryManagedGraphReadDenial::ForeignGraph
                | WorthQueryManagedGraphReadDenial::ForeignReadProof
                | WorthQueryManagedGraphReadDenial::TerminalReleaseMismatch,
            ) => Err(MapKernelFailure::Domain(Denial::ReadDenied {
                root,
                denial: map_authorized_read_denial(ReadDenial::Session, plan.name),
            })),
        }
    }
}

const _: fn() = || {
    fn send<T: Send>() {}
    send::<PreparedPair<'static>>();
};

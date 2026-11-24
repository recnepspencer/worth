//! Borrowed consumption of the structural reader's original request and payer.
use crate::contracts::content::codec_control::RelationalContentControl;
use crate::storage::content::RelationalContentDenial as Denial;

use super::{
    CustomInvariantWorkMeter, CustomPreparationStop, StructuralReadError, StructuralWalkDenialKind,
};

pub use crate::contracts::validation::custom_rule::work_meter::native_rule_debit::CustomInvariantNativeCheckpointDenial;

/// Read work and retention admission on the original custom-invariant meter.
/// This handle creates no allowance and grants no graph or mutation authority.
pub struct CustomInvariantReadBudget<'borrow> {
    execution: worth_execution::ExecutionRequest<'borrow, 'borrow>,
    work: &'borrow CustomInvariantWorkMeter,
}

impl<'borrow> CustomInvariantReadBudget<'borrow> {
    pub(crate) fn execution(&self) -> worth_execution::ExecutionRequest<'borrow, 'borrow> {
        self.execution
    }

    /// Spend the installed rule allowance and the actual SDK context once.
    /// Source visits and provider steps share the immediate original SDK account.
    pub fn checkpoint_native(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        units: u64,
    ) -> Result<(), CustomInvariantNativeCheckpointDenial> {
        self.native_debit()
            .checkpoint(self.execution, context, units)
    }

    pub(crate) fn native_debit(
        &self,
    ) -> crate::contracts::validation::custom_rule::work_meter::native_rule_debit::NativeRuleDebit<'_>
    {
        self.work.native_debit()
    }

    pub(crate) fn debit_native_rule_work(
        &self,
        units: u64,
    ) -> Result<(), crate::contracts::execution::denial_cause::RelationalExecutionDenialCause> {
        use crate::contracts::execution::denial_cause::RelationalExecutionDenialCause as Cause;
        self.execution.check_live().map_err(Cause::from)?;
        if self.work.try_debit_native_work(units) {
            return Ok(());
        }
        Err(match self.work.leased_stop() {
            Some(CustomPreparationStop::Cancelled) => Cause::Cancelled,
            Some(CustomPreparationStop::DeadlineElapsed) => Cause::DeadlineElapsed,
            Some(CustomPreparationStop::MemoryAdmission(denial)) => {
                crate::contracts::execution::denial::lease_denial(
                    worth_execution::LeaseDenial::MemoryExhausted(denial),
                )
            }
            Some(CustomPreparationStop::MemoryExhausted) => Cause::ScratchCapacityExceeded,
            Some(CustomPreparationStop::UncheckedCustomKernel) => Cause::UncheckedCustomKernel,
            Some(CustomPreparationStop::Request(stop)) => {
                Cause::from(worth_execution::WorkCeilingDenial::Stopped(stop))
            }
            _ => Cause::WorkExhausted,
        })
    }

    pub(super) fn new(
        execution: worth_execution::ExecutionRequest<'borrow, 'borrow>,
        work: &'borrow CustomInvariantWorkMeter,
    ) -> Self {
        Self { execution, work }
    }

    pub fn check_live(&self) -> Result<(), StructuralWalkDenialKind> {
        crate::contracts::validation::custom_rule::interruption::check_live(self.execution)
            .map_err(StructuralWalkDenialKind::Interrupted)
    }

    pub fn checkpoint(&self, units: u64) -> Result<(), StructuralWalkDenialKind> {
        self.check_live()?;
        self.work
            .try_charge_work(units)
            .then_some(())
            .ok_or_else(|| self.refusal(StructuralReadError::WorkBudgetExceeded))
    }

    /// Admit backing before allocating it. Retention remains on the existing
    /// native payer; an escaping complete walk retains that payer itself.
    pub fn claim_retained(&self, bytes: u64) -> Result<(), StructuralWalkDenialKind> {
        self.checkpoint(0)?;
        self.work
            .try_claim_bytes(bytes)
            .then_some(())
            .ok_or_else(|| self.refusal(StructuralReadError::WorkBudgetExceeded))
    }

    /// Share only the payer already charged by this exact structural reader.
    pub(crate) fn retained_payer(&self) -> CustomInvariantWorkMeter {
        self.work.clone()
    }

    pub(crate) fn check_scratch_peak(&self, bytes: u64) -> Result<(), StructuralWalkDenialKind> {
        self.checkpoint(0)?;
        crate::validation::custom_rule::registration::CustomInvariantLeaseBudget::check_scratch_peak(self.work, bytes)
            .then_some(())
            .ok_or_else(|| self.refusal(StructuralReadError::WorkBudgetExceeded))
    }

    pub(crate) fn refusal(&self, error: StructuralReadError) -> StructuralWalkDenialKind {
        self.native_debit().refusal(error)
    }
}

impl CustomInvariantReadBudget<'_> {
    /// Certify the exact installed contract under this actual Source allowance.
    pub fn require_content_contract(
        &self,
        frame: &crate::storage::content::frame::RelationalContentFrame,
        name: &str,
        version: u32,
        maximum_frame_bytes: usize,
    ) -> Result<
        (),
        crate::contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial,
    > {
        run_source(self, |control| {
            frame.require_contract_controlled(control, name, version, maximum_frame_bytes)
        })
    }
}

/// Keep the exact Source refusal outside codec vocabulary.
fn run_source<T>(
    budget: &CustomInvariantReadBudget<'_>,
    operation: impl FnOnce(&mut RelationalContentControl<'_, '_, '_>) -> Result<T, Denial>,
) -> Result<
    T,
    crate::contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial,
> {
    use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial;
    use crate::contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Read;
    let (result, source_stop) = crate::storage::content::control::run_codec(
        &budget.execution(),
        Some(budget.native_debit()),
        operation,
    );
    if let Some(cause) = source_stop {
        return Err(DerivedRelationReadDenial::Structural(cause).into());
    }
    result.map_err(|cause| match cause {
        Denial::Execution(cause) => Read::Source(DerivedRelationReadDenial::Request(cause)),
        cause => Read::Content(cause),
    })
}

impl CustomInvariantReadBudget<'_> {
    pub(crate) fn read_field_content(
        &self,
        view: &worth_relational_branch::internal_port::content::source::NativeContentSourceView,
        row: crate::contracts::identity::data::record_ids::EntityId,
        kind: crate::contracts::identity::data::primitives::KindId,
        locator: &worth_foundational::facade::AspectFieldLocator,
        value: Option<&worth_foundational::facade::AspectValue>,
    ) -> Result<
        Option<crate::storage::content::frame::RelationalContentFrame>,
        crate::contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial,
    > {
        run_source(self, |control| {
            view.read(control, row, kind, locator, value)
        })
    }
}

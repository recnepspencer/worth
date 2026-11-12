//! Funded immutable program windows, interpreted against actual original records.
use super::{WorldDurableMediumDenial as Medium, WorldReadmissionDenial as Denial};
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_foundational::facade::{
    CanonicalDigestId, DurableWorldOwnerIdentity, HomeProgramRevisionBoundary,
};

/// Descriptive source input, not a live Home authority or permission to replay.
/// Original Home selection and later actual owner attachment remain mandatory.
#[derive(Debug)]
pub struct WorldHomeProgramLineage {
    owner: DurableWorldOwnerIdentity,
    initial: CanonicalDigestId,
    current: CanonicalDigestId,
    rows: ExecutionArray<HomeProgramRevisionBoundary>,
    steps: ExecutionArray<worth_foundational::facade::HomeProgramTransitionProgramStep>,
    _memory: Option<ExecutionMemoryReservation>,
    _step_names: Option<ExecutionMemoryReservation>,
}
impl WorldHomeProgramLineage {
    pub fn from_recorded(
        owner: DurableWorldOwnerIdentity,
        initial: CanonicalDigestId,
        current: CanonicalDigestId,
        rows: impl ExactSizeIterator<Item = HomeProgramRevisionBoundary>,
        steps: impl ExactSizeIterator<
            Item = worth_foundational::facade::HomeProgramTransitionProgramStep,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let mut captured = ExecutionRequestArrayBuilder::allocate(*request, rows.len())
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        let mut expected = initial;
        let mut through = None;
        for row in rows {
            charge(request)?;
            if row.predecessor() != expected
                || row.through().owner() != owner
                || through.is_some_and(|before| before >= row.through().ordinal())
            {
                return Err(Denial::ProgramRevisionMismatch);
            }
            expected = row.successor();
            through = Some(row.through().ordinal());
            captured
                .push(row)
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        }
        if expected != current {
            return Err(Denial::ProgramRevisionMismatch);
        }
        let lineage = Self::retain(
            owner,
            initial,
            current,
            captured
                .seal()
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
            request,
        )?;
        lineage.with_program_steps(steps, request)
    }

    fn retain(
        owner: DurableWorldOwnerIdentity,
        initial: CanonicalDigestId,
        current: CanonicalDigestId,
        rows: ExecutionArray<HomeProgramRevisionBoundary>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let (layout, _) = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed World lineage allocation layout");
        let memory = request
            .reserve_retained_memory(layout.pad_to_align().size() as u64)
            .map_err(|cause| Denial::Medium(Medium::Request(cause)))?;
        Ok(Arc::new(Self {
            owner,
            initial,
            current,
            rows,
            steps: ExecutionRequestArrayBuilder::allocate(*request, 0)
                .and_then(|array| array.seal())
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
            _memory: memory,
            _step_names: None,
        }))
    }

    pub(crate) fn extend(
        previous: Option<&Self>,
        boundary: HomeProgramRevisionBoundary,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let rows = previous.map_or(&[][..], |previous| previous.rows.elements());
        let count = rows
            .len()
            .checked_add(1)
            .ok_or(Denial::JournalSourceMismatch)?;
        let initial = previous.map_or(boundary.predecessor(), |previous| previous.initial);
        if previous.is_some_and(|previous| {
            previous.owner != boundary.through().owner()
                || previous.current != boundary.predecessor()
                || previous
                    .rows
                    .elements()
                    .last()
                    .is_some_and(|prior| prior.through().ordinal() >= boundary.through().ordinal())
        }) {
            return Err(Denial::ProgramRevisionMismatch);
        }
        let mut captured = ExecutionRequestArrayBuilder::allocate(*request, count)
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        for row in rows.iter().copied().chain(std::iter::once(boundary)) {
            charge(request)?;
            captured
                .push(row)
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        }
        let extended = Self::retain(
            boundary.through().owner(),
            initial,
            boundary.successor(),
            captured
                .seal()
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?,
            request,
        )?;
        extended.with_program_steps(
            previous
                .map_or(&[][..], |p| p.steps.elements())
                .iter()
                .cloned(),
            request,
        )
    }

    pub(crate) fn with_program_steps(
        self: Arc<Self>,
        additional: impl ExactSizeIterator<
            Item = worth_foundational::facade::HomeProgramTransitionProgramStep,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<Self>, Denial> {
        let count = self
            .steps
            .elements()
            .len()
            .checked_add(additional.len())
            .ok_or(Denial::JournalSourceMismatch)?;
        // Durable branch names are bounded at 256 bytes. Fund every cloned
        // name before evaluating the lazy source iterator.
        let name_bytes = count
            .checked_mul(256)
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Denial::JournalSourceMismatch)?;
        let names = request
            .reserve_retained_memory(name_bytes)
            .map_err(|cause| Denial::Medium(Medium::Request(cause)))?;
        let mut steps = ExecutionRequestArrayBuilder::allocate(*request, count)
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        let mut prior = None;
        for step in self.steps.elements().iter().cloned().chain(additional) {
            charge(request)?;
            let attempt = step.original().attempt();
            if attempt.owner() != self.owner
                || prior.is_some_and(|old| old >= attempt.ordinal())
                || !self.matches_recorded_attempt(
                    *step.recorded().bytes(),
                    attempt.ordinal(),
                    request,
                )?
            {
                return Err(Denial::ProgramRevisionMismatch);
            }
            prior = Some(attempt.ordinal());
            steps
                .push(step)
                .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        }
        let mut lineage = Arc::try_unwrap(self).map_err(|_| Denial::JournalSourceMismatch)?;
        lineage.steps = steps
            .seal()
            .map_err(|cause| Denial::Medium(Medium::Backing(cause)))?;
        lineage._step_names = names;
        Ok(Arc::new(lineage))
    }
    /// Resolve descriptive exact-step attribution; this grants no runtime authority.
    pub fn recorded_activation_for_original(
        &self,
        original: &worth_foundational::facade::CompositeAttemptAssociation,
        recorded: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<[u8; 32], Denial> {
        let rows = self.steps.elements();
        // Exact original membership, never a broad attempt-window exemption.
        let mut lower = 0;
        let mut upper = rows.len();
        while lower < upper {
            charge(request)?;
            let middle = lower + (upper - lower) / 2;
            if rows[middle].original().attempt().ordinal() < original.attempt().ordinal() {
                lower = middle + 1;
            } else {
                upper = middle;
            }
        }
        if let Some(step) = rows
            .get(lower)
            .filter(|step| step.original().attempt() == original.attempt())
        {
            if step.original() != original || step.recorded().bytes() != &recorded {
                return Err(Denial::ProgramRevisionMismatch);
            }
            return Ok(*step.activation().bytes());
        }
        Ok(recorded)
    }

    pub(crate) fn owner(&self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub(crate) fn current(&self) -> CanonicalDigestId {
        self.current
    }
    pub(crate) fn initial(&self) -> CanonicalDigestId {
        self.initial
    }
    /// Interpret a label against these recorded windows. This observes DATA;
    /// it grants no Home, Source or publication authority. The receiver must
    /// independently admit the original protected Home that supplied it.
    pub fn matches_recorded_attempt(
        &self,
        revision: [u8; 32],
        attempt: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Denial> {
        let rows = self.rows.elements();
        let mut lower = 0;
        let mut upper = rows.len();
        while lower < upper {
            charge(request)?;
            let middle = lower + (upper - lower) / 2;
            if rows[middle].through().ordinal() < attempt {
                lower = middle + 1;
            } else {
                upper = middle;
            }
        }
        let expected = rows
            .get(lower)
            .map_or(self.current, |row| row.predecessor());
        Ok(expected.bytes() == &revision)
    }
    pub(crate) fn admits_checkpoint(
        &self,
        revision: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, Denial> {
        if self.current.bytes() == &revision || self.initial.bytes() == &revision {
            return Ok(true);
        }
        for row in self.rows.elements() {
            charge(request)?;
            if row.successor().bytes() == &revision {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
fn charge(request: &ExecutionRequest<'_, '_>) -> Result<(), Denial> {
    request
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(1)
            } else {
                context.poll()
            }
        })
        .map_err(|cause| Denial::Medium(Medium::Request(cause)))?
        .0
        .map_err(|cause| {
            Denial::Medium(Medium::Request(
                worth_execution::WorkCeilingDenial::Stopped(cause),
            ))
        })
}

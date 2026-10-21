//! Count-before-build custody for the returned descriptive projection.
use super::*;
use std::mem::size_of;
use worth_foundational::facade::{
    AspectFieldLocator, CanonicalFieldPath, FieldKey, PortableAspectContractBasis,
};
type Error = ValidatedMutationTouchProjectionError;

#[derive(Default, Clone, Copy)]
pub(super) struct Extent {
    pub touches: usize,
    pub structs: usize,
    pub nested: usize,
}
pub(super) struct TouchCollection<'p, 'c> {
    pub extent: Extent,
    pub backing: Option<TouchBacking<'p>>,
    pub failure: Option<Error>,
    pub checkpoint: &'c mut dyn FnMut(u64) -> Result<(), Error>,
}
impl<'p> TouchCollection<'p, '_> {
    fn reach(&mut self, nested: usize, whole: bool) -> bool {
        if self.failure.is_some() {
            return false;
        }
        let count = if whole {
            &mut self.extent.structs
        } else {
            &mut self.extent.touches
        };
        match count
            .checked_add(1)
            .zip(self.extent.nested.checked_add(nested))
        {
            Some((next, bytes)) => {
                *count = next;
                self.extent.nested = bytes;
            }
            None => {
                self.failure = Some(capacity());
                return false;
            }
        }
        if let Err(error) = (self.checkpoint)(1) {
            self.failure = Some(error);
            return false;
        }
        true
    }
    pub fn insert(&mut self, touch: ValidatedMutationTouch) {
        if self.reach(0, false)
            && let Some(backing) = &mut self.backing
        {
            backing.touches.push(touch);
        }
    }
    pub fn field(&mut self, kind: KindId, locator: &AspectFieldLocator) {
        if self.reach(locator.owned_allocation_capacity_bytes(), false)
            && let Some(backing) = &mut self.backing
        {
            backing
                .touches
                .push(ValidatedMutationTouch::WriteEntityField {
                    kind,
                    locator: locator.clone(),
                });
        }
    }
    pub fn selected_field(
        &mut self,
        kind: KindId,
        basis: &PortableAspectContractBasis,
        field: &FieldKey,
    ) {
        let bytes = basis
            .key()
            .owned_allocation_capacity_bytes()
            .checked_add(field.owned_allocation_capacity_bytes())
            .and_then(|n| n.checked_add(size_of::<FieldKey>()));
        let Some(bytes) = bytes else {
            self.failure = Some(capacity());
            return;
        };
        if self.reach(bytes, false)
            && let Some(backing) = &mut self.backing
        {
            let locator = crate::transactions::data::planned_aspect_field_locator(
                basis.key().clone(),
                CanonicalFieldPath::single(field.clone()),
            );
            backing
                .touches
                .push(ValidatedMutationTouch::WriteEntityField { kind, locator });
        }
    }
    pub fn whole_field(&mut self, touch: ValidatedEntityStructTouch<'p>) {
        if touch.field.is_none() {
            // Reserve one possible unrepresentable result before group resolution.
            let Some(count) = self.extent.touches.checked_add(1) else {
                self.failure = Some(capacity());
                return;
            };
            self.extent.touches = count;
        }
        if self.reach(0, true)
            && let Some(backing) = &mut self.backing
        {
            backing.structs.push(touch);
        }
    }
}
pub(super) fn capacity() -> Error {
    execution(crate::execution::RelationalExecutionDenialCause::ResultCapacityExceeded)
}
pub(super) fn execution(cause: crate::execution::RelationalExecutionDenialCause) -> Error {
    Error::Execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
        partition_identity: None,
    })
}
impl Extent {
    pub fn bytes(self) -> Result<u64, Error> {
        self.touches
            .checked_mul(size_of::<ValidatedMutationTouch>())
            .and_then(|n| {
                self.structs
                    .checked_mul(size_of::<ValidatedEntityStructTouch<'_>>())
                    .and_then(|s| n.checked_add(s))
            })
            .and_then(|n| n.checked_add(self.nested))
            .and_then(|n| n.checked_add(size_of::<TouchBacking<'_>>() + 2 * size_of::<usize>()))
            .and_then(|n| u64::try_from(n).ok())
            .ok_or_else(capacity)
    }
    pub fn allocate<'p>(
        self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<TouchBacking<'p>, Error> {
        let mut backing = TouchBacking {
            touches: Vec::new(),
            structs: Vec::new(),
            _memory: memory,
        };
        backing
            .touches
            .try_reserve_exact(self.touches)
            .map_err(|_| capacity())?;
        backing
            .structs
            .try_reserve_exact(self.structs)
            .map_err(|_| capacity())?;
        Ok(backing)
    }
}

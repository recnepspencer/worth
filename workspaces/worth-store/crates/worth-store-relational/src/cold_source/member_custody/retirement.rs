//! One original guard's actual remaining registration; no source replacement.
use super::MemberCustodyOwner;
use crate::cold_source::StoreColdSource;
use std::{
    any::Any,
    sync::{Arc, TryLockError},
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::{
    RelationalDurableMemberRetirementObservation as O,
    RelationalDurableMemberRetirementRefusal as R,
};

impl StoreColdSource {
    pub(in crate::cold_source) fn validate_custody_owner(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), R> {
        if original.is::<crate::cold_source::selected_read::StoreSelectedRead>() {
            return self.validate_selected_custody(original);
        }
        let owner = original
            .downcast_ref::<MemberCustodyOwner>()
            .ok_or(R::ForeignCustody)?;
        if !Arc::ptr_eq(&owner.state, &self.custody.state)
            || !owner
                .serving
                .as_ref()
                .is_some_and(|serving| Arc::ptr_eq(serving, &self.serving))
        {
            return Err(R::ForeignCustody);
        }
        Ok(())
    }

    pub(in crate::cold_source) fn advance_custody_owner(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> O {
        if original.is::<crate::cold_source::selected_read::StoreSelectedRead>() {
            return self.advance_selected_custody(execution, original);
        }
        if let Err(cause) = self.validate_custody_owner(original) {
            return O::Refused(cause);
        }
        if let Err(cause) = execution.consult() {
            return O::Stopped(cause);
        }
        let owner = original
            .downcast_mut::<MemberCustodyOwner>()
            .expect("validated original guard");
        if owner.next < owner.members.len() {
            let mut state = match owner.state.try_write() {
                Ok(state) => state,
                Err(TryLockError::WouldBlock) => return O::Busy,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
            };
            let member = owner.members[owner.next];
            if !super::release_member(&mut state.counts, &member, &mut owner.retired_cells) {
                return O::Refused(R::ForeignCustody);
            }
            owner.next += 1;
        }
        if owner.next == owner.members.len() {
            // Even an originally empty registration owns an extra Serving Arc.
            // The exact borrowed source owns another immutable Arc throughout;
            // this cannot invoke the physical provider's final destructor.
            drop(owner.serving.take());
            O::Complete
        } else {
            O::Advanced
        }
    }
}

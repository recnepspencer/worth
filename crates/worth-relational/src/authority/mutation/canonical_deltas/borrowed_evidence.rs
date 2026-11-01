//! Borrowed view of the SAME canonical comparison; no copied schema/value owner.
use super::data::{
    AuthoritativePatchDeltaOperation as Operation, CanonicalAspectDeltaEvidence as Owned,
    LifecycleTransitionClass,
};
use crate::identity::data::EntityId;
use crate::publication::patch::data::RecordStructuralChange;
use worth_foundational::facade::{
    AspectContract, AspectKey, AspectValue, ContractValidatedAspectValue, FieldLevelAspectPatch,
    StructAspectValue,
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum BorrowedPatchOperation<'a> {
    WholeAspectSet {
        value: &'a ContractValidatedAspectValue,
    },
    WholeAspectClear {
        contract: &'a AspectContract,
    },
    FieldLevelPatch {
        patch: &'a FieldLevelAspectPatch,
    },
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum BorrowedAspectDeltaEvidence<'a> {
    ScalarAspectValueTransition {
        old_present: bool,
        new_present: bool,
        old_value: Option<&'a AspectValue>,
        new_value: Option<&'a AspectValue>,
    },
    StructAspectValueTransition {
        old_present: bool,
        new_present: bool,
        old_value: Option<&'a StructAspectValue>,
        new_value: Option<&'a StructAspectValue>,
    },
    EndpointIdentity {
        old: Option<EntityId>,
        new: Option<EntityId>,
    },
    Lifecycle {
        transition: LifecycleTransitionClass,
    },
    Structural {
        change: RecordStructuralChange,
    },
    AuthoritativePatch {
        operation: BorrowedPatchOperation<'a>,
    },
}
impl BorrowedAspectDeltaEvidence<'_> {
    pub(super) fn into_owned(self, key: &AspectKey) -> Owned {
        let locator = worth_foundational::facade::AspectValueLocator::whole_aspect(
            worth_foundational::facade::AspectLocator::new(
                worth_foundational::facade::LocatorAuthority::Authoritative,
                key.clone(),
            ),
        );
        match self {
            Self::ScalarAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
            } => Owned::ScalarAspectValueTransition {
                locator,
                old_present,
                new_present,
                old_value: old_value.cloned(),
                new_value: new_value.cloned(),
            },
            Self::StructAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
            } => Owned::StructAspectValueTransition {
                locator,
                old_present,
                new_present,
                old_value: old_value.cloned(),
                new_value: new_value.cloned(),
            },
            Self::EndpointIdentity { old, new } => Owned::EndpointIdentity { locator, old, new },
            Self::Lifecycle { transition } => Owned::Lifecycle {
                locator,
                transition,
            },
            Self::Structural { change } => Owned::Structural { locator, change },
            Self::AuthoritativePatch { operation } => Owned::AuthoritativePatch {
                locator,
                operation: match operation {
                    BorrowedPatchOperation::WholeAspectSet { value } => Operation::WholeAspectSet {
                        value: value.clone(),
                    },
                    BorrowedPatchOperation::WholeAspectClear { contract } => {
                        Operation::WholeAspectClear {
                            contract: contract.clone(),
                        }
                    }
                    BorrowedPatchOperation::FieldLevelPatch { patch } => {
                        Operation::FieldLevelPatch {
                            patch: patch.clone(),
                        }
                    }
                },
            },
        }
    }
}
impl Owned {
    pub(crate) fn borrowed(&self) -> BorrowedAspectDeltaEvidence<'_> {
        use BorrowedAspectDeltaEvidence as View;
        match self {
            Self::ScalarAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
                ..
            } => View::ScalarAspectValueTransition {
                old_present: *old_present,
                new_present: *new_present,
                old_value: old_value.as_ref(),
                new_value: new_value.as_ref(),
            },
            Self::StructAspectValueTransition {
                old_present,
                new_present,
                old_value,
                new_value,
                ..
            } => View::StructAspectValueTransition {
                old_present: *old_present,
                new_present: *new_present,
                old_value: old_value.as_ref(),
                new_value: new_value.as_ref(),
            },
            Self::EndpointIdentity { old, new, .. } => View::EndpointIdentity {
                old: *old,
                new: *new,
            },
            Self::Lifecycle { transition, .. } => View::Lifecycle {
                transition: *transition,
            },
            Self::Structural { change, .. } => View::Structural { change: *change },
            Self::AuthoritativePatch { operation, .. } => View::AuthoritativePatch {
                operation: match operation {
                    Operation::WholeAspectSet { value } => {
                        BorrowedPatchOperation::WholeAspectSet { value }
                    }
                    Operation::WholeAspectClear { contract } => {
                        BorrowedPatchOperation::WholeAspectClear { contract }
                    }
                    Operation::FieldLevelPatch { patch } => {
                        BorrowedPatchOperation::FieldLevelPatch { patch }
                    }
                },
            },
        }
    }
}

#[cfg(test)]
mod tests;

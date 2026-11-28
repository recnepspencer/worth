pub(crate) use crate::expression::model::{Expr, IdentitySpec, SignalValue};
pub(crate) use crate::recipe::model::{
    KeyedRecipeFamilySpec, KeyedSetValue, KeyedSourceFamilySpec, RecipeFamilyReadSpec,
    RecipeReadSpec, RecipeSpec, SourceSpec, TransactionOp,
};
pub(crate) use crate::runtime::adapters::RuntimeEnvelope;
pub(crate) use crate::runtime::compute_callbacks;
pub(crate) use crate::runtime::compute_callbacks::{
    ComputeCallbackFailure, ComputeCallbackFailureClass,
};
pub(crate) use crate::runtime::core::RuntimeCore;
pub(crate) use crate::runtime::policy::{RuntimePolicyPreset, RuntimePolicySpec};
pub(crate) use std::cell::RefCell;
pub(crate) use std::rc::Rc;

pub(crate) fn number(value: f64) -> Expr {
    Expr::Value {
        value: SignalValue::Number(value),
    }
}

pub(crate) fn read(id: &str) -> Expr {
    Expr::Read { id: id.to_owned() }
}

pub(crate) fn assert_digest_shape(digest: &str) {
    assert_eq!(digest.len(), 64);
    assert!(digest
        .chars()
        .all(|character| character.is_ascii_hexdigit()));
}

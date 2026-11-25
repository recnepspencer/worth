//! Typed vocabulary of a stock scope grant: every capability dimension a grant
//! row carries is a Rust enum with an identity scalar binding, never a string.
//!
//! [`ScopeAccess`] is the grant's access level. [`ScopeGrantAction`],
//! [`ScopeGrantPurpose`] and [`ScopeGrantDisclosure`] are the fixed action,
//! purpose and disclosure dimensions capability contracts match exactly.
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationScalarValueBinding, BoolApplicationValueBinding,
};

use worth_query_declaration::__worth_query_vocabulary_binding as vocabulary_binding;

use super::types::ScopeAccess;

/// The action every stock scope grant authorizes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopeGrantAction {
    Access,
}

/// The purpose every stock scope grant is issued for.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopeGrantPurpose {
    Shared,
}

/// The disclosure level a stock scope grant permits: the governed scope view.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ScopeGrantDisclosure {
    ScopeView,
}

vocabulary_binding!(
    /// Identity binding of a grant's [`ScopeAccess`] level.
    ScopeAccessBinding for ScopeAccess = "worth.shared_access.scope_access.v1" {
        ReadOnly, ReadAndWatch,
    }
);
vocabulary_binding!(
    /// Identity binding of [`ScopeGrantAction`].
    ScopeGrantActionBinding for ScopeGrantAction = "worth.shared_access.grant_action.v1" {
        Access,
    }
);
vocabulary_binding!(
    /// Identity binding of [`ScopeGrantPurpose`].
    ScopeGrantPurposeBinding for ScopeGrantPurpose = "worth.shared_access.grant_purpose.v1" {
        Shared,
    }
);
vocabulary_binding!(
    /// Identity binding of [`ScopeGrantDisclosure`].
    ScopeGrantDisclosureBinding for ScopeGrantDisclosure =
        "worth.shared_access.grant_disclosure.v1" {
        ScopeView,
    }
);

/// A stock grant value whose binding encodes every member: the grant
/// vocabulary and `bool`.
#[doc(hidden)]
pub trait ScopeGrantValue {
    type Binding: ApplicationScalarValueBinding<Value = Self>;
}
impl ScopeGrantValue for ScopeAccess {
    type Binding = ScopeAccessBinding;
}
impl ScopeGrantValue for ScopeGrantAction {
    type Binding = ScopeGrantActionBinding;
}
impl ScopeGrantValue for ScopeGrantPurpose {
    type Binding = ScopeGrantPurposeBinding;
}
impl ScopeGrantValue for ScopeGrantDisclosure {
    type Binding = ScopeGrantDisclosureBinding;
}
impl ScopeGrantValue for bool {
    type Binding = BoolApplicationValueBinding;
}

/// The encoded stock grant value `value`.
#[doc(hidden)]
pub fn scope_grant_value<Value: ScopeGrantValue>(
    value: Value,
) -> ApplicationEncodedScalarValue<Value::Binding> {
    match ApplicationEncodedScalarValue::try_new(value) {
        Ok(encoded) => encoded,
        Err(_) => unreachable!("stock grant vocabulary and bool encode every member"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_foundational::facade::{AspectValue, InternedString};
    use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;

    #[test]
    fn every_access_level_round_trips_and_foreign_text_is_refused() {
        for access in [ScopeAccess::ReadOnly, ScopeAccess::ReadAndWatch] {
            let encoded = ScopeAccessBinding::encode(&access).unwrap();
            assert_eq!(ScopeAccessBinding::decode(&encoded).unwrap(), access);
        }
        let foreign = AspectValue::String(InternedString::from("read_and_watch"));
        assert!(ScopeAccessBinding::decode(&foreign).is_err());
    }
}

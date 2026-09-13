use super::{ApplicationFieldPresence, ApplicationScalarValueBinding};

pub trait DeclaredApplicationFieldValue {
    type Value: 'static;
    type Binding: ApplicationScalarValueBinding<Value = Self::Value>;
    const PRESENCE: ApplicationFieldPresence;
}

/// Marker for a schema field whose value must exist on every live record.
pub trait RequiredApplicationFieldValue: DeclaredApplicationFieldValue {}

/// Marker for a schema field whose value may be lawfully absent.
pub trait OptionalApplicationFieldValue: DeclaredApplicationFieldValue {}

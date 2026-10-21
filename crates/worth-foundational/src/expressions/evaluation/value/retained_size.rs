use super::{Composite, ExpressionValue, Repr};

impl ExpressionValue {
    /// A conservative size of heap backing reachable from this immutable value.
    /// Shared subvalues are counted per occurrence; the inline receiver itself
    /// is excluded. This is physical retention, not semantic evaluation work.
    pub fn retained_allocation_bytes(&self) -> usize {
        let arc = 2 * std::mem::size_of::<usize>();
        match &self.0 {
            Repr::String(value) => arc.saturating_add(value.len()),
            Repr::Bytes(value) | Repr::Id(value) => arc.saturating_add(value.len()),
            Repr::Some(value) => arc
                .saturating_add(std::mem::size_of::<Self>())
                .saturating_add(value.retained_allocation_bytes()),
            Repr::List(value) | Repr::Map(value) | Repr::Record(value) | Repr::Entry(value) => {
                value.items.iter().fold(
                    arc.saturating_add(std::mem::size_of::<Composite>())
                        .saturating_add(
                            value
                                .items
                                .len()
                                .saturating_mul(std::mem::size_of::<Self>()),
                        ),
                    |bytes, item| bytes.saturating_add(item.retained_allocation_bytes()),
                )
            }
            Repr::Bits { limbs, .. } => arc.saturating_add(std::mem::size_of_val(limbs.as_ref())),
            Repr::Logic4 { value, unknown, .. } => arc
                .saturating_mul(2)
                .saturating_add(std::mem::size_of_val(value.as_ref()))
                .saturating_add(std::mem::size_of_val(unknown.as_ref())),
            Repr::Bool(_)
            | Repr::Integer(_)
            | Repr::Float32(_)
            | Repr::Float64(_)
            | Repr::Decimal { .. }
            | Repr::Enum(_)
            | Repr::Quantity(_)
            | Repr::None => 0,
        }
    }
}

//! Type-level instance identity.
//!
//! Everything else this crate expresses is *kind*-level: `Proof` and
//! `AuthorityWitness` are `PhantomData`, so two runtimes of the same type are
//! indistinguishable. A witness proves the lane, never which instance — which
//! is why milestone 9.16 Phase 8 fell back to comparing a `u64` at run time.
//!
//! A brand closes that gap without a value. [`with_brand`] opens a scope
//! carrying a fresh, invariant lifetime; values bound inside it are
//! [`Branded`] with that lifetime, and the compiler refuses to substitute a
//! value from one scope where another scope's is expected. Rung 1: the mistake
//! is unrepresentable, and the brand costs nothing at run time.
//!
//! **Known limit — read before reaching for this.** Every use must sit inside
//! the branding scope. A brand cannot be stored in a struct that outlives the
//! `with_brand` call, so it does not fit a long-lived carrier such as a
//! runtime handle held in a registry. For those, a process-unique identity
//! *value* compared at run time is the right tool, and it belongs to the
//! runtime that owns the counter — not here. This crate has no clock and no
//! counter by design.

use std::marker::PhantomData;

/// Invariant in `'id`, so two brands never unify.
///
/// `fn(&'id ()) -> &'id ()` puts the lifetime in both argument and return
/// position, which makes it invariant. Covariance here would let a longer
/// brand shorten to match a shorter one, and the separation would silently
/// stop holding.
type Invariant<'id> = PhantomData<fn(&'id ()) -> &'id ()>;

/// The authority to bind values into one branding scope.
///
/// Held by the closure [`with_brand`] calls, and by nothing else. There is no
/// public constructor: a `Brand` can only be received, never made, so a
/// consumer cannot conjure a second token for someone else's scope.
#[derive(Debug, PartialEq, Eq)]
pub struct Brand<'id> {
    invariant: Invariant<'id>,
}

impl<'id> Brand<'id> {
    fn seal() -> Self {
        Self {
            invariant: PhantomData,
        }
    }

    /// Bind a value into this scope.
    ///
    /// Values bound by the same `Brand` share a lifetime and are freely
    /// interchangeable; values from a different scope are a type error.
    pub fn bind<T>(&self, value: T) -> Branded<'id, T> {
        Branded {
            value,
            invariant: PhantomData,
        }
    }
}

/// A value that belongs to exactly one branding scope.
///
/// `size_of::<Branded<'_, T>>() == size_of::<T>()` — the brand is carried
/// entirely in the type.
#[derive(Debug, PartialEq, Eq)]
pub struct Branded<'id, T> {
    value: T,
    invariant: Invariant<'id>,
}

impl<'id, T> Branded<'id, T> {
    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }

    /// Rebrand the payload, keeping the scope.
    ///
    /// The brand survives because it is the *instance* that was established,
    /// not the shape of what it holds.
    pub fn map<U>(self, transform: impl FnOnce(T) -> U) -> Branded<'id, U> {
        Branded {
            value: transform(self.value),
            invariant: PhantomData,
        }
    }
}

/// Open a branding scope.
///
/// The `for<'id>` bound is what makes the brand *generative*: the closure must
/// accept **any** lifetime, so it cannot assume its brand equals another
/// scope's, and the compiler will not unify two calls.
///
/// ```
/// use worth_proof::with_brand;
///
/// let total = with_brand(|brand| {
///     let left = brand.bind(2_u32);
///     let right = brand.bind(3_u32);
///     // Same scope, so these compose.
///     left.into_value() + right.into_value()
/// });
/// assert_eq!(total, 5);
/// ```
///
/// Two scopes do not mix; `tests/ui/contracts/compile_fail/brands_do_not_cross_scopes.rs`
/// pins that as a compile error.
pub fn with_brand<R>(scope: impl for<'id> FnOnce(Brand<'id>) -> R) -> R {
    scope(Brand::seal())
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::{with_brand, Branded};

    const _: () = assert!(size_of::<Branded<'static, u64>>() == size_of::<u64>());
    const _: () = assert!(size_of::<Branded<'static, ()>>() == 0);

    #[test]
    fn values_bound_by_one_brand_stay_usable_together() {
        let joined = with_brand(|brand| {
            let left = brand.bind("left");
            let right = brand.bind("right");
            format!("{}-{}", left.into_value(), right.into_value())
        });

        assert_eq!(joined, "left-right");
    }

    #[test]
    fn mapping_preserves_the_scope_while_changing_the_payload() {
        let length = with_brand(|brand| brand.bind("payload").map(str::len).into_value());

        assert_eq!(length, 7);
    }

    #[test]
    fn a_brand_can_be_carried_out_of_the_scope_only_by_unbinding() {
        // The escape hatch is deliberate and visible: `into_value` drops the
        // brand. What must not exist is a way to keep the brand while leaving
        // the scope, which the invariant lifetime forbids.
        let escaped = with_brand(|brand| brand.bind(7_u8).into_value());

        assert_eq!(escaped, 7);
    }
}

//! State that belongs to one request's thread of control.
//!
//! A request's meters and readers may move with the request, but no two
//! threads may share one. A kernel is `Fn + Sync` and a concurrent compute
//! shares what it captures, so a request-local type is never `Sync`: it
//! carries [`RequestLocal`], and asserts so beside its definition with
//! [`assert_request_local`].

use std::cell::Cell;
use std::marker::PhantomData;

/// The marker of request-local state: `Send`, never `Sync`.
pub(in crate::domain_computation) type RequestLocal = PhantomData<Cell<()>>;

/// Fails to compile when `$ty` is `Sync`.
macro_rules! assert_request_local {
    ($($ty:ty),+ $(,)?) => {
        $(
            const _: fn() = || {
                trait AmbiguousIfSync<A> {
                    fn probe() {}
                }
                impl<T: ?Sized> AmbiguousIfSync<()> for T {}
                struct IfSync;
                impl<T: ?Sized + Sync> AmbiguousIfSync<IfSync> for T {}
                let _ = <$ty as AmbiguousIfSync<_>>::probe;
            };
        )+
    };
}
pub(in crate::domain_computation) use assert_request_local;

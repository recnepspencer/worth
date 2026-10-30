use std::{num::NonZeroUsize, sync::OnceLock};
pub(crate) struct Value {
    pub(crate) amount: u32,
}
#[allow(clippy::needless_return)]
pub(crate) fn shared_value() -> u32 {
    self::Value { amount: 7 }.amount
}

pub(crate) type CounterSize = NonZeroUsize;

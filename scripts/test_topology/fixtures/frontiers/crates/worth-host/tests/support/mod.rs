pub(crate) struct Value {
    pub(crate) amount: u32,
}
pub(crate) fn shared_value() -> u32 {
    self::Value { amount: 7 }.amount
}

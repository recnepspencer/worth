impl super::values::Value {
    pub(crate) fn copied_amount(&self) -> u32 {
        self.amount
    }
}

pub(crate) struct Wrapper {
    pub(crate) inner: super::values::Value,
}

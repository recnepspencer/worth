pub(in super::super) struct OracleWalk;
impl OracleWalk {
    pub(in super::super) fn after_effect() -> Self {
        Self
    }
}
impl super::super::refreshed_rejoin::OccurrenceSearchCharge for OracleWalk {
    type Refusal = core::convert::Infallible;
    fn occurrence_path(&mut self, _: usize) -> Result<(), Self::Refusal> {
        Ok(())
    }
    fn inspect_row(&mut self) -> Result<(), Self::Refusal> {
        Ok(())
    }
}

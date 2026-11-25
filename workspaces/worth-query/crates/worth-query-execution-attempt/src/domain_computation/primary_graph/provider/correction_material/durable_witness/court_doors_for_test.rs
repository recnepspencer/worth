//! Sealed test doors over a durable witness's canonical fact.
use super::*;

impl WorthQueryPrimaryGraphProvider {
    /// Reads the occurrence's real canonical fact, lets a court forge it, then
    /// seals it through the production rejoin.
    #[doc(hidden)]
    pub fn seal_forged_durable_fact_for_test(
        &self,
        execution: ExecutionRequest<'_, '_>,
        originals: &WorldProgramActivationHistory,
        owner: RuntimeWorldOwnerIdentity,
        receipt: &RelationalCommitReceipt,
        forge: impl FnOnce(&mut Fact),
    ) -> Result<Option<Sealed>, Denial> {
        self.graph.with_runtime(&execution, |native| {
            let home = DurableOriginals {
                originals,
                owner,
                native,
            };
            let Some((_, Some(mut fact))) = self.durable_fact(execution, &home, receipt)? else {
                return Ok(None);
            };
            forge(&mut fact);
            home.seal(execution, receipt, fact)
        })
    }
}

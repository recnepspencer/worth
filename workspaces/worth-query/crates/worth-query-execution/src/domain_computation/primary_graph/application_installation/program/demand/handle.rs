use super::*;

impl<Schema, Program, Demand> WorthQueryAdmittedProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn observed_source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, Demand>,
    > {
        self.admitted.observed_source()
    }

    /// Charged owner work comparing the admitted child's checkpoint output.
    pub const fn checkpoint_readmission_work_units(&self) -> u64 {
        self.admitted.checkpoint_readmission_work_units()
    }

    /// Saturating sum of owner-computed charged-work ceilings for accepted
    /// checkpoint readmissions; excludes decoding, rejected candidates and
    /// adoption. `u64::MAX` marks saturation, not a finite aggregate guarantee.
    pub const fn checkpoint_readmission_work_bound(&self) -> u64 {
        self.admitted.checkpoint_readmission_work_bound()
    }

    /// Largest admitted scratch bound among the child's accepted checkpoint
    /// readmissions. This is not measured allocation or resident memory.
    pub const fn checkpoint_readmission_charged_preparation_bytes(&self) -> u64 {
        self.admitted
            .checkpoint_readmission_charged_preparation_bytes()
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryOutputDemandDenial> {
        self.admitted.notifications()
    }

    pub fn close(&mut self) {
        self.admitted.close();
    }
}

impl<Schema, Program, Demand> WorthQuerySettledProgramOutput<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn retained(&self) -> std::sync::Arc<WorthQueryOutputDemandSettlement> {
        std::sync::Arc::clone(&self.retained)
    }
}

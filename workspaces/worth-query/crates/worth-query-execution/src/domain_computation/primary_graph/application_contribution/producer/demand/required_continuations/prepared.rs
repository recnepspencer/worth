use super::*;

impl<Schema, Family> PreparedRequiredFreshSlot<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    WorthQueryAdmittedOutputDemand<Schema, Family>: Send + Sync,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    /// Allocate the typed erasure slot before the successor can be admitted.
    /// The real demand and its interest are moved into this storage later.
    pub(in crate::domain_computation::primary_graph) fn prepare(
        registry: &WorthQueryOutputDemandRegistry,
        claim: SelectedRequiredRefreshClaim,
        installed: &InstalledProducerProvider<Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let pending_provenance =
            RequiredSuccessorProvenance::prepare_from_ready(&claim, installed, admission)?;
        let slot_bytes = std::mem::size_of::<TypedRequiredSuccessor<Schema, Family>>();
        let initialized_work = slot_bytes
            .checked_add(std::mem::size_of::<
                WorthQueryAdmittedOutputDemand<Schema, Family>,
            >())
            .and_then(|work| work.checked_add(3))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(initialized_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let capacity =
            registry.reserve_required_continuation_capacity(slot_bytes, slot_bytes, admission)?;
        let successor = Box::new(TypedRequiredSuccessor {
            demand: None,
            pending_claim: Some(claim),
            pending_provenance: Some(pending_provenance),
            predecessor: None,
        });
        Ok(Self {
            successor,
            capacity,
        })
    }

    /// The exact owned claim used by this prepared slot is borrowed for the
    /// successor's registry admission and predecessor transfer. It cannot be
    /// exchanged for a later claim before `finish`.
    pub(in crate::domain_computation::primary_graph) fn claim(
        &self,
    ) -> &SelectedRequiredRefreshClaim {
        self.successor
            .pending_claim
            .as_ref()
            .expect("prepared required successor owns its Ready refresh claim")
    }

    /// Bind the actual issued mode to the newly admitted demand before the
    /// registry replacement, scheduling, or producer effect can run.
    pub(in crate::domain_computation::primary_graph) fn bind_admitted(
        &mut self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
    ) {
        assert!(demand.progression_provenance.required().is_none());
        demand.progression_provenance = DemandProgressionProvenance::RequiredSuccessor(
            self.successor
                .pending_provenance
                .take()
                .expect("prepared required successor carries one authentic mode"),
        );
    }

    pub(in crate::domain_computation::primary_graph) fn finish(
        mut self,
        demand: WorthQueryAdmittedOutputDemand<Schema, Family>,
        outcome: RequiredFreshOutcome,
    ) -> RequiredFreshProgress<Schema> {
        assert!(demand.progression_provenance.required().is_some());
        self.successor.predecessor = Some(
            self.successor
                .pending_claim
                .take()
                .expect("prepared required successor owns its original claim")
                .into_predecessor(),
        );
        self.successor.demand = Some(demand);
        RequiredFreshProgress {
            contacts: super::contacts::ContactAttribution::Unowned,
            outcome: Some(outcome),
            successor: self.successor,
            capacity: self.capacity,
        }
    }
}

#[cfg(feature = "test-query-execution-observer")]
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Funded storage of one typed required successor, before its admission.
    #[doc(hidden)]
    pub fn required_successor_custody_bytes_for_test<
        Family: WorthQueryProducerOutputFamily<Schema>,
    >() -> usize {
        std::mem::size_of::<TypedRequiredSuccessor<Schema, Family>>()
    }
}

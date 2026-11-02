//! Attack only retained descriptive facts from genuine owner-issued settlements.
use super::*;

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn assert_original_provenance(
        &self,
        other: Option<&Self>,
        foreign_disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
        mut check: impl FnMut(&Self) -> Result<(), WorthQueryOutputDemandDenial>,
    ) {
        let original = self.observation.original_settled_producer().unwrap();
        let mut sources = Vec::new();
        match original.source() {
            OriginalSettledProducerSource::Committed {
                settlement,
                publication,
                disclosure,
                ..
            } => {
                assert!(settlement.is_some());
                assert!(publication.is_some());
                let mut corrupted = original.source.clone();
                if let OriginalSettledProducerSource::Committed { settlement, .. } = &mut corrupted
                {
                    *settlement = None;
                }
                sources.push(("committed settlement", corrupted));
                let mut corrupted = original.source.clone();
                if let OriginalSettledProducerSource::Committed { publication, .. } = &mut corrupted
                {
                    *publication = None;
                }
                sources.push(("publication", corrupted));
                if disclosure.is_some() || foreign_disclosure.is_some() {
                    let mut corrupted = original.source.clone();
                    if let OriginalSettledProducerSource::Committed { disclosure, .. } =
                        &mut corrupted
                    {
                        *disclosure = if disclosure.is_some() {
                            None
                        } else {
                            foreign_disclosure
                        };
                    }
                    sources.push(("disclosure", corrupted));
                }
            }
            OriginalSettledProducerSource::StableReused { .. } => {
                let foreign = other
                    .unwrap()
                    .application_commit_receipt()
                    .unwrap()
                    .exact_output_settlement()
                    .unwrap()
                    .clone();
                let OriginalSettledProducerSource::StableReused { settlement } = original.source()
                else {
                    unreachable!()
                };
                assert_ne!(
                    foreign.as_ref(),
                    settlement.as_ref(),
                    "independent earlier producer publication"
                );
                sources.push((
                    "reused settlement",
                    OriginalSettledProducerSource::StableReused {
                        settlement: foreign,
                    },
                ));
            }
            OriginalSettledProducerSource::Restored { checkpoint } => {
                let mut checkpoint = checkpoint.clone();
                checkpoint.idempotency_key[0] ^= 1;
                sources.push((
                    "restored checkpoint",
                    OriginalSettledProducerSource::Restored { checkpoint },
                ));
            }
        }
        check(self).unwrap();
        for (fact, source) in sources {
            let seed = Arc::new(OriginalSettledProducerObservation {
                runtime_authority: original.runtime_authority,
                schema: original.schema.clone(),
                read: original.read.clone(),
                source,
                correspondence: original.correspondence.clone(),
                _memory: None,
            });
            let candidate = WorthQueryOutputDemandSettlement {
                posture: self.posture,
                runtime_authority: self.runtime_authority,
                schema_binding: self.schema_binding.clone(),
                producer_identity: self.producer_identity.as_str().to_owned().into(),
                output_family_identity: self.output_family_identity.clone(),
                receipt: self
                    .application_commit_receipt()
                    .cloned()
                    .map(SettlementReceiptCustody::Owned),
                stable: self.stable.clone(),
                output_correspondence: self.output_correspondence.clone(),
                restored_source: self.restored_source.clone(),
                readiness_delivery: self.readiness_delivery.clone(),
                producer_contacts_in_this_demand: self.producer_contacts_in_this_demand,
                observation: self.observation.with_original_producer(seed),
            };
            let cause = check(&candidate).expect_err(fact);
            assert!(
                matches!(
                    cause.kind(),
                    WorthQueryOutputDemandDenialKind::ForeignSettlement
                ),
                "{fact}: {cause:?}"
            );
        }
    }
}

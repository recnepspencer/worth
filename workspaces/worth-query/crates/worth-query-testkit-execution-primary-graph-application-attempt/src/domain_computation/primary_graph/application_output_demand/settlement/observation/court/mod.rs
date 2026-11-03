//! Attack descriptive provenance and receipts; never manufacture accepted authority.

use crate::domain_computation::primary_graph::application_attempt::compare_and_commit::commit_receipt::certification::CertificationWorthQueryApplicationCommitReceiptForTest;
use std::sync::Arc;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::OriginalSettledProducerObservation;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::OriginalSettledProducerSource;
use worth_query_execution::domain_computation::primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::receipt_custody::SettlementReceiptCustody;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandSettlement;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::producer_identity;
use worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::provenance;
use worth_query_execution::facade::domain_computation::WorthQueryOutputSettlementProvenanceDenial as Reason;

/// Test doors on a library type, owned by the moved courts.
pub trait CourtWorthQueryOutputDemandSettlementForTest: Sized {
    fn assert_original_provenance(
        &self,
        other: Option<&Self>,
        foreign_disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
        check: impl FnMut(&Self) -> Result<(), WorthQueryOutputDemandDenial>,
    );
    fn assert_receipt_disclosure_provenance(
        &self,
        expected: Arc<OriginalSourceInspectionDisclosure>,
        foreign: Arc<OriginalSourceInspectionDisclosure>,
        check: impl FnMut(&Self) -> Result<(), WorthQueryOutputDemandDenial>,
    );
    fn with_corrupted_provenance(
        &self,
        source: OriginalSettledProducerSource,
        receipt: Option<WorthQueryApplicationCommitReceipt>,
    ) -> Self;
}

impl CourtWorthQueryOutputDemandSettlementForTest for WorthQueryOutputDemandSettlement {
    fn assert_original_provenance(
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
                sources.push((
                    "absent committed settlement",
                    corrupted,
                    Reason::OriginalSettlementMissing,
                ));
                if let Some(other) = other {
                    let foreign = other
                        .application_commit_receipt()
                        .unwrap()
                        .exact_output_settlement()
                        .unwrap()
                        .clone();
                    assert!(!Arc::ptr_eq(settlement.as_ref().unwrap(), &foreign));
                    let mut corrupted = original.source.clone();
                    if let OriginalSettledProducerSource::Committed { settlement, .. } =
                        &mut corrupted
                    {
                        *settlement = Some(foreign);
                    }
                    sources.push((
                        "present foreign committed settlement",
                        corrupted,
                        Reason::ForeignSettlementOwner,
                    ));
                }
                let mut corrupted = original.source.clone();
                if let OriginalSettledProducerSource::Committed { publication, .. } = &mut corrupted
                {
                    *publication = None;
                }
                sources.push(("publication", corrupted, Reason::PublicationMismatch));
                match (disclosure, foreign_disclosure) {
                    (Some(_), _) => {
                        let mut corrupted = original.source.clone();
                        if let OriginalSettledProducerSource::Committed { disclosure, .. } =
                            &mut corrupted
                        {
                            *disclosure = None;
                        }
                        sources.push((
                            "absent disclosure",
                            corrupted,
                            Reason::OriginalDisclosureMissing,
                        ));
                    }
                    (None, Some(foreign)) => {
                        let mut corrupted = original.source.clone();
                        if let OriginalSettledProducerSource::Committed { disclosure, .. } =
                            &mut corrupted
                        {
                            *disclosure = Some(foreign);
                        }
                        sources.push((
                            "receipt disclosure absent",
                            corrupted,
                            Reason::ReceiptDisclosureMissing,
                        ));
                    }
                    (None, None) => {}
                }
            }
            OriginalSettledProducerSource::StableReused { settlement } => {
                let foreign = other
                    .unwrap()
                    .application_commit_receipt()
                    .unwrap()
                    .exact_output_settlement()
                    .unwrap()
                    .clone();
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
                    Reason::ReusedSettlementOwnerMismatch,
                ));
            }
            OriginalSettledProducerSource::Restored { checkpoint } => {
                let mut checkpoint = checkpoint.clone();
                checkpoint.idempotency_key[0] ^= 1;
                sources.push((
                    "restored checkpoint",
                    OriginalSettledProducerSource::Restored { checkpoint },
                    Reason::RestoredCheckpointMismatch,
                ));
            }
        }
        check(self).unwrap();
        if let OriginalSettledProducerSource::Committed { .. } = original.source() {
            let candidate = self.with_corrupted_provenance(
                original.source.clone(),
                Some(
                    self.application_commit_receipt()
                        .unwrap()
                        .clone()
                        .without_settlement_description(),
                ),
            );
            let cause = check(&candidate).expect_err("receipt settlement absent");
            assert_eq!(
                cause.kind(),
                WorthQueryOutputDemandDenialKind::SettlementProvenance(
                    Reason::ReceiptSettlementMissing
                )
            );
        }
        for (fact, source, expected) in sources {
            let candidate =
                self.with_corrupted_provenance(source, self.application_commit_receipt().cloned());
            let cause = check(&candidate).expect_err(fact);
            assert_eq!(
                cause.kind(),
                WorthQueryOutputDemandDenialKind::SettlementProvenance(expected),
                "{fact}: {cause:?}"
            );
            assert_eq!(cause.recovery_posture(), worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Terminal);
        }
    }

    fn assert_receipt_disclosure_provenance(
        &self,
        expected: Arc<OriginalSourceInspectionDisclosure>,
        foreign: Arc<OriginalSourceInspectionDisclosure>,
        mut check: impl FnMut(&Self) -> Result<(), WorthQueryOutputDemandDenial>,
    ) {
        assert!(
            !Arc::ptr_eq(&expected, &foreign),
            "two genuine capture owners"
        );
        let mut source = self
            .observation
            .original_settled_producer()
            .unwrap()
            .source
            .clone();
        let OriginalSettledProducerSource::Committed { disclosure, .. } = &mut source else {
            panic!("the actual committed producer supplies this corruption court");
        };
        *disclosure = Some(expected.clone());
        let receipt = self.application_commit_receipt().unwrap();
        // Both are explicitly corrupted descriptive receipts, not issued disclosure authority.
        let matching = self.with_corrupted_provenance(
            source.clone(),
            Some(receipt.clone().with_corrupted_original_disclosure(expected)),
        );
        check(&matching)
            .expect("matching retained disclosure and corrupted receipt reach the real reader");
        let corrupted = self.with_corrupted_provenance(
            source,
            Some(receipt.clone().with_corrupted_original_disclosure(foreign)),
        );
        let cause = check(&corrupted).expect_err("present foreign receipt disclosure");
        assert_eq!(
            cause.kind(),
            WorthQueryOutputDemandDenialKind::SettlementProvenance(Reason::ForeignDisclosureOwner),
            "real receipt call site"
        );
    }

    fn with_corrupted_provenance(
        &self,
        source: OriginalSettledProducerSource,
        receipt: Option<WorthQueryApplicationCommitReceipt>,
    ) -> Self {
        let original = self.observation.original_settled_producer().unwrap();
        let seed = Arc::new(OriginalSettledProducerObservation {
            runtime_authority: original.runtime_authority,
            schema: original.schema.clone(),
            read: original.read.clone(),
            source,
            correspondence: original.correspondence.clone(),
            _memory: None,
        });
        Self {
            posture: self.posture,
            runtime_authority: self.runtime_authority,
            schema_binding: self.schema_binding.clone(),
            producer_identity: self.producer_identity.as_str().to_owned().into(),
            output_family_identity: self.output_family_identity.clone(),
            receipt: receipt.map(SettlementReceiptCustody::Owned),
            stable: self.stable.clone(),
            output_correspondence: self.output_correspondence.clone(),
            restored_source: self.restored_source.clone(),
            readiness_delivery: self.readiness_delivery.clone(),
            producer_contacts_in_this_demand: self.producer_contacts_in_this_demand,
            observation: self.observation.with_original_producer(seed),
        }
    }
}

/// Test doors on a library type, owned by the moved courts.
pub trait CourtOriginalSourceInspectionDisclosureForTest: Sized {
    fn court_rejects_present_foreign_disclosure(self: Arc<Self>, foreign: Arc<Self>);
}

impl CourtOriginalSourceInspectionDisclosureForTest for OriginalSourceInspectionDisclosure {
    fn court_rejects_present_foreign_disclosure(self: Arc<Self>, foreign: Arc<Self>) {
        assert!(
            !Arc::ptr_eq(&self, &foreign),
            "independently captured owners"
        );
        worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::provenance::require_disclosure_owner(Some(&self), Some(&self)).unwrap();
        worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::provenance::require_disclosure_owner(None, None).unwrap();
        for (original, expected, reason) in [
            (Some(&foreign), Some(&self), Reason::ForeignDisclosureOwner),
            (None, Some(&self), Reason::OriginalDisclosureMissing),
            (Some(&self), None, Reason::ReceiptDisclosureMissing),
        ] {
            let cause = worth_query_execution::domain_computation::primary_graph::application_output_demand::settlement::observation::provenance::require_disclosure_owner(original, expected)
                .expect_err("literal disclosure cause");
            assert_eq!(
                cause.kind(),
                WorthQueryOutputDemandDenialKind::SettlementProvenance(reason)
            );
        }
    }
}

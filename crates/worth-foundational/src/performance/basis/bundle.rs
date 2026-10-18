use crate::performance::claims::FoundationalPerformanceClaimSurface;
use crate::performance::layouts::{
    FoundationalLayoutAnnotatedClaimConstructionDenial, FoundationalLayoutIntentClaim,
};

use super::attachments::{
    FoundationalPerformanceContractName, FoundationalPerformanceCounterSpec,
    FoundationalPerformanceSupportingEvidenceRow,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundationalPerformanceBundleConstructionDenial {
    DuplicateContractName,
    DuplicateCounterSpec,
    DuplicateSupportingEvidenceRow,
    LayoutAttachment(FoundationalLayoutAnnotatedClaimConstructionDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundationalPerformanceBundle<Claim> {
    claim: Claim,
    layout_intent_claim: Option<FoundationalLayoutIntentClaim>,
    contract_names: Vec<FoundationalPerformanceContractName>,
    counter_specs: Vec<FoundationalPerformanceCounterSpec>,
    supporting_evidence_rows: Vec<FoundationalPerformanceSupportingEvidenceRow>,
}

impl<Claim> FoundationalPerformanceBundle<Claim>
where
    Claim: FoundationalPerformanceClaimSurface,
{
    pub const fn claim(&self) -> &Claim {
        &self.claim
    }

    pub const fn layout_intent_claim(&self) -> Option<&FoundationalLayoutIntentClaim> {
        self.layout_intent_claim.as_ref()
    }

    pub fn contract_names(&self) -> &[FoundationalPerformanceContractName] {
        &self.contract_names
    }

    pub fn counter_specs(&self) -> &[FoundationalPerformanceCounterSpec] {
        &self.counter_specs
    }

    pub(crate) fn supporting_evidence_rows(
        &self,
    ) -> &[FoundationalPerformanceSupportingEvidenceRow] {
        &self.supporting_evidence_rows
    }

    pub fn boundary(&self) -> crate::performance::FoundationalPerformanceBoundary {
        self.claim.boundary()
    }
}

#[derive(Debug, Clone)]
pub struct FoundationalPerformanceBundleBuilder<Claim> {
    claim: Claim,
    layout_intent_claim: Option<FoundationalLayoutIntentClaim>,
    contract_names: Vec<FoundationalPerformanceContractName>,
    counter_specs: Vec<FoundationalPerformanceCounterSpec>,
    supporting_evidence_rows: Vec<FoundationalPerformanceSupportingEvidenceRow>,
}

impl<Claim> FoundationalPerformanceBundleBuilder<Claim>
where
    Claim: FoundationalPerformanceClaimSurface,
{
    pub fn new(claim: Claim) -> Self {
        Self {
            claim,
            layout_intent_claim: None,
            contract_names: Vec::new(),
            counter_specs: Vec::new(),
            supporting_evidence_rows: Vec::new(),
        }
    }

    pub fn attach_layout_intent_claim(
        mut self,
        layout_intent_claim: FoundationalLayoutIntentClaim,
    ) -> Self {
        self.layout_intent_claim = Some(layout_intent_claim);
        self
    }

    pub fn attach_contract_name(
        mut self,
        contract_name: FoundationalPerformanceContractName,
    ) -> Self {
        self.contract_names.push(contract_name);
        self
    }

    pub fn attach_counter_spec(mut self, counter_spec: FoundationalPerformanceCounterSpec) -> Self {
        self.counter_specs.push(counter_spec);
        self
    }

    pub fn attach_supporting_evidence_row(
        mut self,
        supporting_evidence_row: FoundationalPerformanceSupportingEvidenceRow,
    ) -> Self {
        self.supporting_evidence_rows.push(supporting_evidence_row);
        self
    }

    pub fn finish(
        mut self,
    ) -> Result<FoundationalPerformanceBundle<Claim>, FoundationalPerformanceBundleConstructionDenial>
    {
        if let Some(layout_intent_claim) = &self.layout_intent_claim
            && self.claim.access_pattern() != layout_intent_claim.access_pattern()
        {
            return Err(
                FoundationalPerformanceBundleConstructionDenial::LayoutAttachment(
                    FoundationalLayoutAnnotatedClaimConstructionDenial::AccessPatternMismatch {
                        claim_access_pattern: self.claim.access_pattern(),
                        layout_access_pattern: layout_intent_claim.access_pattern(),
                    },
                ),
            );
        }

        self.contract_names.sort();
        if self
            .contract_names
            .windows(2)
            .any(|window| window[0] == window[1])
        {
            return Err(FoundationalPerformanceBundleConstructionDenial::DuplicateContractName);
        }

        self.counter_specs
            .sort_by(|left, right| left.name().cmp(right.name()));
        if self
            .counter_specs
            .windows(2)
            .any(|window| window[0].name() == window[1].name())
        {
            return Err(FoundationalPerformanceBundleConstructionDenial::DuplicateCounterSpec);
        }

        self.supporting_evidence_rows
            .sort_by(|left, right| left.code().cmp(right.code()));
        if self
            .supporting_evidence_rows
            .windows(2)
            .any(|window| window[0].code() == window[1].code())
        {
            return Err(
                FoundationalPerformanceBundleConstructionDenial::DuplicateSupportingEvidenceRow,
            );
        }

        Ok(FoundationalPerformanceBundle {
            claim: self.claim,
            layout_intent_claim: self.layout_intent_claim,
            contract_names: self.contract_names,
            counter_specs: self.counter_specs,
            supporting_evidence_rows: self.supporting_evidence_rows,
        })
    }
}

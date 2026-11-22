use super::{
    artifact_authority_denial_detail, WorthQueryArtifactAuthorityMatch, WorthQueryArtifactDenial,
    WorthQueryArtifactDenialKind, WorthQueryArtifactHandleCore,
    WorthQueryArtifactProductionAdmission, WorthQueryArtifactTransferAdmission,
};

impl WorthQueryArtifactHandleCore {
    pub(super) fn validate_transfer(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        self.validate_binding(admission)?;
        let binding = self.owner.binding();
        if !binding
            .contract
            .contract()
            .consumer_roles()
            .iter()
            .any(|role| role == &admission.consumer_stage)
        {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::ConsumerRoleNotInstalled,
                "consumer stage is not admitted by the installed artifact contract",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_output(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        self.validate_binding(admission)?;
        if !self
            .owner
            .binding()
            .contract
            .contract()
            .producer_roles()
            .iter()
            .any(|role| role == &admission.consumer_stage)
        {
            return Err(self.denial(
                WorthQueryArtifactDenialKind::ProducerRoleNotInstalled,
                "output stage is not admitted by the installed artifact contract",
            ));
        }
        Ok(())
    }

    pub(super) fn validate_replacement_binding(
        &self,
        admission: &WorthQueryArtifactProductionAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let binding = self.owner.binding();
        let authority = admission.authority();
        self.validate_authority_match(WorthQueryArtifactAuthorityMatch {
            runtime: binding.domain_authority.runtime_authority()
                == authority.domain_authority.runtime_authority(),
            generation: binding
                .domain_authority
                .is_current_installation_generation()
                && authority
                    .domain_authority
                    .is_current_installation_generation()
                && binding.domain_authority.installation_generation()
                    == authority.domain_authority.installation_generation(),
            operation: binding.operation_identity == authority.operation_identity
                && binding.binding_identity == authority.binding_identity,
            run: binding.run_identity == authority.run_identity,
            stage: self.holder_stage == authority.stage_identity,
            basis: binding.basis_identity == authority.basis_identity,
            payload_owner: binding.contract.owner() == authority.contract.owner(),
            contract: binding.contract.contract().identity()
                == authority.contract.contract().identity(),
        })
    }

    fn validate_binding(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let binding = self.owner.binding();
        self.validate_authority_match(WorthQueryArtifactAuthorityMatch {
            runtime: binding.domain_authority.runtime_authority()
                == admission.domain_authority.runtime_authority(),
            generation: binding
                .domain_authority
                .is_current_installation_generation()
                && admission
                    .domain_authority
                    .is_current_installation_generation()
                && binding.domain_authority.installation_generation()
                    == admission.domain_authority.installation_generation(),
            operation: binding.operation_identity == admission.operation_identity
                && binding.binding_identity == admission.binding_identity,
            run: binding.run_identity == admission.run_identity,
            stage: self.holder_stage == admission.predecessor_stage,
            basis: binding.basis_identity == admission.basis_identity,
            payload_owner: binding.contract.owner() == admission.expected_contract.owner(),
            contract: binding.contract.contract().identity()
                == admission.expected_contract.contract().identity(),
        })
    }

    pub(super) fn contract_matches(
        &self,
        reference: &worth_query_installation::facade::WorthQueryArtifactContractReference,
    ) -> bool {
        let contract = self.owner.binding().contract.contract();
        contract.family() == reference.family()
            && contract.schema_version() == reference.schema_version()
            && contract.protocol_version() == reference.protocol_version()
    }

    fn denial(
        &self,
        kind: WorthQueryArtifactDenialKind,
        detail: &'static str,
    ) -> WorthQueryArtifactDenial {
        WorthQueryArtifactDenial::new(
            kind,
            Some(self.owner.binding().contract.contract().family().as_str()),
            detail,
        )
    }

    fn validate_authority_match(
        &self,
        authority_match: WorthQueryArtifactAuthorityMatch,
    ) -> Result<(), WorthQueryArtifactDenial> {
        match authority_match.denial_kind() {
            Some(kind) => Err(self.denial(kind, artifact_authority_denial_detail(kind))),
            None => Ok(()),
        }
    }
}

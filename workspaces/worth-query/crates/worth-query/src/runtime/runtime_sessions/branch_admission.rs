//! Branch admission and exact comparison-basis capture.
use super::*;

impl WorthQueryRuntime {
    pub fn branch<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.branch_with_options(execution, label, WorthQueryBranchOptions::default())
    }

    pub fn branch_with_options<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.try_branch_with_options(execution, label, options)
    }

    pub fn try_branch<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.try_branch_with_options(execution, label, WorthQueryBranchOptions::default())
    }

    pub fn try_branch_with_options<'a>(
        &'a mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
        options: WorthQueryBranchOptions,
    ) -> Result<WorthQueryBranchSession<'a>, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        self.admit_branch_session_label(&label)?;
        let basis_admission = self.branch_basis_admission(
            label.clone(),
            options.effect_policy(),
            "runtime-branch-basis-admission",
        );
        Ok(WorthQueryBranchSession::new(
            execution,
            label,
            self,
            options,
            basis_admission,
        ))
    }

    pub(crate) fn capture_branch_comparison_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        label: WorthQuerySessionLabel,
    ) -> Result<WorthQueryRuntimeBranchComparisonBasis, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::BranchPreview)?;
        let admission = self.branch_basis_admission(
            label,
            WorthQueryEffectPolicy::DeriveOnly,
            "runtime-branch-comparison-basis-admission",
        );
        Ok(WorthQueryRuntimeBranchComparisonBasis::new(
            admission,
            self.current_snapshot_identity_with_request(execution),
        ))
    }

    pub(super) fn branch_basis_admission(
        &self,
        label: WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        evidence_tag: &'static str,
    ) -> WorthQueryBranchBasisAdmission {
        let branch_support_evidence = self
            .backend
            .support_profile()
            .support_for(WorthQueryRuntimeFacadeFamily::BranchPreview)
            .map(|support| support.evidence().to_vec())
            .unwrap_or_default();
        let evidence_rows = std::iter::once(WorthQueryBasisAdmissionEvidenceRow::tagged(
            evidence_tag,
            evidence_tag,
        ))
        .chain(
            branch_support_evidence
                .into_iter()
                .map(WorthQueryBasisAdmissionEvidenceRow::support_profile_token),
        )
        .collect::<Vec<_>>();
        WorthQueryBranchBasisAdmission::new(
            &self.evidence_authority,
            label,
            effect_policy,
            evidence_rows,
        )
    }
}

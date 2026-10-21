//! Complete original native output association before disposition or performance.
use crate::branch::AdmittedRelationalBranchBasis;
use crate::durability::authority::{
    DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority,
};
use crate::runtime::{
    RelationalSchemaTransitionAdmissionDenial as Denial,
    RelationalSchemaTransitionAdmissionDenialKind as Kind,
};

impl DurabilityRecoveryAuthority<'_> {
    /// Re-admit the COMPLETE original native publication on the recovered SOURCE
    /// basis, including roles whose retirement is only staged. This validates
    /// association, not successor survival, retirement permission or currentness.
    /// Native target publication independently validates successor preservation.
    #[allow(clippy::too_many_arguments)]
    pub fn validate_recovered_codec_output_correspondence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recovered: &RecoveredRelationalRuntimeAuthority,
        basis: &AdmittedRelationalBranchBasis,
        producer: &str,
        dependency: &[u8; 32],
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
        roles: &[crate::mvcc::RequiredOutputRoleTarget],
    ) -> Result<(), Denial> {
        crate::execution::check_request(execution).map_err(Denial::request)?;
        let state = recovered.codec_transition.as_ref().ok_or_else(|| {
            Denial::new(Kind::RecoveredAuthorityMismatch, "no staged codec recovery")
        })?;
        if recovered.runtime_instance_id != self.runtime.runtime_instance_id()
            || !recovered.owns_image(basis.descriptor())
            || state.transaction_id.is_none()
            || state.publication_custody.is_none()
            || state.retirement.is_none()
        {
            return Err(Denial::new(
                Kind::RecoveredAuthorityMismatch,
                "output correspondence requires the exact staged recovery owner",
            ));
        }
        let result = crate::history::data::validate_codec_output_correspondence(
            execution,
            self.runtime,
            basis,
            producer,
            dependency,
            meaning,
            roles,
        );
        crate::execution::check_request(execution).map_err(Denial::request)?;
        result.map_err(|stop| match stop {
            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                Denial::request(cause)
            }
            crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
                Denial::new(Kind::CodecClosureRejected, detail)
            }
        })?;
        Ok(())
    }
}

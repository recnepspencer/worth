//! Conservative invalidation of one carried output publication by canonical effects.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence;

impl WorthQueryApplicationCommittedChanges {
    /// This detects mutation of a declared full-GEN output, not field creation.
    /// The diagnostic join must still prove the exact field's Native revision.
    pub(in crate::domain_computation::primary_graph) fn affects_output_records(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        outputs: &WorthQueryApplicationOutputCorrespondence,
    ) -> Result<bool, worth_execution::MapKernelStop> {
        for patch in &self.envelope.patch.authoritative_record_patches {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)?;
            let RecordRef::Entity(entity) = patch.target else {
                continue;
            };
            for (_, _, _, output) in outputs.native_witness_roles() {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)?;
                if entity == output
                    && (patch.structural_change != RecordStructuralChange::Updated
                        || patch.authoritative_patch.full_grammar_operation_count() != 0)
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

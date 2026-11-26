//! The correction mechanism, demand and reconciliation of an aftermath basis.
use super::{AftermathBasisBuilder, WorthQueryInstalledReconciliationProcedure};
use worth_query_declaration::facade::application_aftermath::{
    PortableApplicationAftermathContract, PortableCorrectionMechanism,
};

pub(super) fn push_correction_contract(
    builder: &mut AftermathBasisBuilder,
    portable: &PortableApplicationAftermathContract,
    reconciliation: Option<&WorthQueryInstalledReconciliationProcedure>,
) {
    match portable.mechanism() {
        Some(PortableCorrectionMechanism::RecordedInverse(inverse)) => {
            builder.text("mechanism", "recorded-inverse");
            builder.text("inverse-operation", inverse.inverse_operation_slot());
            builder.text(
                "lowering-correspondence",
                inverse.lowering_correspondence().correspondence_slot(),
            );
            builder.text(
                "preimage-byte-bound",
                inverse
                    .preimage_demand()
                    .maximum_encoded_bytes()
                    .to_string(),
            );
            if let Some(output) = inverse.preimage_demand().output_fields() {
                builder.text("preimage-selection", match output.capture() { worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture::ChangedPreserved => "changed-output-fields", worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture::CompleteRetired => "retired-output-fields" });
                builder.text("preimage-output-role", output.role());
                builder.text("preimage-output-entity", output.entity());
            }
            for (index, locus) in inverse.preimage_demand().loci().iter().enumerate() {
                builder.text(format!("preimage-entity-{index}"), locus.entity());
                builder.text(format!("preimage-aspect-{index}"), locus.aspect());
                builder.text(format!("preimage-field-{index}"), locus.field());
            }
            // Omission grants enter identity only when declared, so a
            // correction without grants keeps its established identity.
            for (index, locus) in inverse
                .preimage_demand()
                .omitted_reversals()
                .iter()
                .enumerate()
            {
                builder.text(format!("omitted-reversal-entity-{index}"), locus.entity());
                builder.text(format!("omitted-reversal-aspect-{index}"), locus.aspect());
                builder.text(format!("omitted-reversal-field-{index}"), locus.field());
            }
            for (index, locus) in inverse.preimage_demand().relations().iter().enumerate() {
                builder.text(format!("preimage-relation-{index}-kind"), locus.relation());
                builder.text(format!("preimage-relation-{index}-from"), locus.from());
                builder.text(format!("preimage-relation-{index}-to"), locus.to());
            }
            push_postcondition(builder, inverse.postcondition());
        }
        Some(PortableCorrectionMechanism::Compensation(compensation)) => {
            builder.text("mechanism", "compensation");
            builder.text(
                "compensating-operation",
                compensation.compensating_operation_slot(),
            );
            push_postcondition(builder, compensation.postcondition());
        }
        None => {
            builder.text("mechanism", "none");
        }
    }
    if let Some(reconciliation) = reconciliation {
        builder.text("reconciliation", reconciliation.procedure_slot());
    } else {
        builder.text("reconciliation", "none");
    }
}

fn push_postcondition(
    builder: &mut AftermathBasisBuilder,
    postcondition: &worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition,
) {
    use worth_query_declaration::facade::application_aftermath::DeclaredAftermathPostcondition;
    match postcondition {
        DeclaredAftermathPostcondition::ExactPriorTruth => {
            builder.text("postcondition", "exact-prior-truth");
        }
        DeclaredAftermathPostcondition::InvariantRestored { invariant } => {
            builder.text("postcondition", "invariant-restored");
            builder.text("postcondition-invariant", invariant);
        }
        DeclaredAftermathPostcondition::BusinessPostcondition { identity } => {
            builder.text("postcondition", "business-postcondition");
            builder.text("postcondition-business", identity);
        }
    }
}

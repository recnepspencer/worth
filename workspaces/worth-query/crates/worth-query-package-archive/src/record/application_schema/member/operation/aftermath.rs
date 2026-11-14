use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredCompensation, DeclaredCorrectionAuthority,
    DeclaredLoweringCorrespondenceRef, DeclaredReconciliationProcedure,
    PortableApplicationAftermathContract, PortableChangedFieldOutput, PortableCorrectionMechanism,
    PortablePreImageDemand, PortablePreImageLocus, PortablePreImageRelationLocus,
    PortableRecordedInverse,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::super::super::super::decode_budget::RecordDecodeAttempt;
use super::super::super::super::sequence::{decode_sequence, write_sequence};
use super::super::super::wire_vocabulary::{
    decode_optional, decode_usize, write_optional, write_usize,
};

mod correction_binding;

pub(super) fn write(
    output: &mut dyn BinaryEncodingSink,
    contract: &PortableApplicationAftermathContract,
) -> Result<(), Denial> {
    output.u16(authority_tag(contract.authority()))?;
    write_optional(output, contract.mechanism(), write_mechanism)?;
    write_optional(output, contract.reconciliation(), |output, value| {
        output.text(value.procedure_slot())
    })?;
    if let Some(binding) = contract.correction_binding() {
        correction_binding::write(output, binding)?;
    }
    Ok(())
}

pub(super) fn decode(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
    has_correction_binding: bool,
) -> Result<PortableApplicationAftermathContract, Denial> {
    let authority = authority_from_tag(input.u16()?)?;
    let mechanism = match input.u16()? {
        0 => None,
        1 => Some(decode_mechanism(input, budget)?),
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    let reconciliation = decode_optional(input, |input| {
        let expression = DeclaredReconciliationProcedure::new(input.text()?.to_owned());
        expression
            .validate()
            .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
        Ok(expression)
    })?;
    let contract = PortableApplicationAftermathContract::from_untrusted_fields(
        authority,
        mechanism,
        reconciliation,
    );
    Ok(if has_correction_binding {
        contract.with_untrusted_correction_binding(correction_binding::decode(input, budget)?)
    } else {
        contract
    })
}

fn write_mechanism(
    output: &mut dyn BinaryEncodingSink,
    mechanism: &PortableCorrectionMechanism,
) -> Result<(), Denial> {
    match mechanism {
        PortableCorrectionMechanism::RecordedInverse(value) => {
            let relations = value.preimage_demand().relations();
            if let Some(target) = value.preimage_demand().changed_output() {
                output.u16(if relations.is_empty() { 4 } else { 5 })?;
                output.text(target.role())?;
                output.text(target.entity())?;
            } else {
                output.u16(if relations.is_empty() { 1 } else { 3 })?;
            }
            write_recorded_inverse(output, value)?;
            if !relations.is_empty() {
                write_sequence(output, relations, |output, locus| {
                    output.text(locus.relation())?;
                    output.text(locus.from())?;
                    output.text(locus.to())
                })?;
            }
            Ok(())
        }
        PortableCorrectionMechanism::Compensation(value) => {
            output.u16(2)?;
            output.text(value.compensating_operation_slot())?;
            write_postcondition(output, value.postcondition())
        }
    }
}

fn decode_mechanism(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<PortableCorrectionMechanism, Denial> {
    match input.u16()? {
        1 => {
            decode_recorded_inverse(input, budget).map(PortableCorrectionMechanism::RecordedInverse)
        }
        2 => {
            let slot = input.text()?.to_owned();
            let postcondition = decode_postcondition(input)?;
            let expression = DeclaredCompensation::new(slot, postcondition);
            expression
                .validate()
                .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
            Ok(PortableCorrectionMechanism::Compensation(expression))
        }
        3 => {
            let value = decode_recorded_inverse(input, budget)?;
            let relations = decode_sequence(input, budget, 12, |input, _| {
                Ok(PortablePreImageRelationLocus::from_untrusted_fields(
                    input.text()?.to_owned(),
                    input.text()?.to_owned(),
                    input.text()?.to_owned(),
                ))
            })?;
            if relations.is_empty() {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            let demand = value
                .preimage_demand()
                .clone()
                .with_untrusted_relations(relations);
            Ok(PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    value.inverse_operation_slot().to_owned(),
                    value.lowering_correspondence().clone(),
                    value.postcondition().clone(),
                    demand,
                ),
            ))
        }
        tag @ (4 | 5) => {
            let output = PortableChangedFieldOutput::from_untrusted_fields(
                input.text()?.to_owned(),
                input.text()?.to_owned(),
            );
            let value = decode_recorded_inverse(input, budget)?;
            if output.role().trim().is_empty()
                || output.entity().trim().is_empty()
                || value.preimage_demand().loci().is_empty()
                || value
                    .preimage_demand()
                    .loci()
                    .iter()
                    .any(|locus| locus.entity() != output.entity())
            {
                return Err(Denial::new(Kind::InvalidRecordShape));
            }
            let mut demand = value
                .preimage_demand()
                .clone()
                .with_untrusted_changed_output(output);
            if tag == 5 {
                let relations = decode_sequence(input, budget, 12, |input, _| {
                    Ok(PortablePreImageRelationLocus::from_untrusted_fields(
                        input.text()?.to_owned(),
                        input.text()?.to_owned(),
                        input.text()?.to_owned(),
                    ))
                })?;
                if relations.is_empty() {
                    return Err(Denial::new(Kind::InvalidRecordShape));
                }
                demand = demand.with_untrusted_relations(relations);
            }
            Ok(PortableCorrectionMechanism::RecordedInverse(
                PortableRecordedInverse::from_untrusted_fields(
                    value.inverse_operation_slot().to_owned(),
                    value.lowering_correspondence().clone(),
                    value.postcondition().clone(),
                    demand,
                ),
            ))
        }
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

fn write_recorded_inverse(
    output: &mut dyn BinaryEncodingSink,
    value: &PortableRecordedInverse,
) -> Result<(), Denial> {
    output.text(value.inverse_operation_slot())?;
    output.text(value.lowering_correspondence().correspondence_slot())?;
    write_postcondition(output, value.postcondition())?;
    write_sequence(output, value.preimage_demand().loci(), |output, locus| {
        output.text(locus.entity())?;
        output.text(locus.aspect())?;
        output.text(locus.field())
    })?;
    write_usize(output, value.preimage_demand().maximum_encoded_bytes())
}

fn decode_recorded_inverse(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<PortableRecordedInverse, Denial> {
    let inverse_operation_slot = input.text()?.to_owned();
    let lowering_correspondence = DeclaredLoweringCorrespondenceRef::new(input.text()?.to_owned());
    lowering_correspondence
        .validate()
        .map_err(|_| Denial::new(Kind::InvalidRecordShape))?;
    let postcondition = decode_postcondition(input)?;
    let loci = decode_sequence(input, budget, 12, |input, _| {
        Ok(PortablePreImageLocus::from_untrusted_fields(
            input.text()?.to_owned(),
            input.text()?.to_owned(),
            input.text()?.to_owned(),
        ))
    })?;
    let preimage_demand = PortablePreImageDemand::from_untrusted_fields(loci, decode_usize(input)?);
    Ok(PortableRecordedInverse::from_untrusted_fields(
        inverse_operation_slot,
        lowering_correspondence,
        postcondition,
        preimage_demand,
    ))
}

fn write_postcondition(
    output: &mut dyn BinaryEncodingSink,
    value: &DeclaredAftermathPostcondition,
) -> Result<(), Denial> {
    match value {
        DeclaredAftermathPostcondition::ExactPriorTruth => output.u16(1),
        DeclaredAftermathPostcondition::InvariantRestored { invariant } => {
            output.u16(2)?;
            output.text(invariant)
        }
        DeclaredAftermathPostcondition::BusinessPostcondition { identity } => {
            output.u16(3)?;
            output.text(identity)
        }
    }
}

fn decode_postcondition(
    input: &mut BinaryInput<'_>,
) -> Result<DeclaredAftermathPostcondition, Denial> {
    match input.u16()? {
        1 => Ok(DeclaredAftermathPostcondition::ExactPriorTruth),
        2 => Ok(DeclaredAftermathPostcondition::InvariantRestored {
            invariant: input.text()?.to_owned(),
        }),
        3 => Ok(DeclaredAftermathPostcondition::BusinessPostcondition {
            identity: input.text()?.to_owned(),
        }),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

const fn authority_tag(value: DeclaredCorrectionAuthority) -> u16 {
    match value {
        DeclaredCorrectionAuthority::RuntimeAlone => 1,
        DeclaredCorrectionAuthority::RuntimeWithExternalOwner => 2,
        DeclaredCorrectionAuthority::NotCorrectable => 3,
    }
}
fn authority_from_tag(tag: u16) -> Result<DeclaredCorrectionAuthority, Denial> {
    match tag {
        1 => Ok(DeclaredCorrectionAuthority::RuntimeAlone),
        2 => Ok(DeclaredCorrectionAuthority::RuntimeWithExternalOwner),
        3 => Ok(DeclaredCorrectionAuthority::NotCorrectable),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}

//! The authored source retains its actual owned payload capacities.
use super::{SignalConditionalCondition, SignalConditionalContractDefinition};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};
use worth_foundational::facade::{AspectValue, InternedString};
impl RetainedStorageMeasurement for SignalConditionalContractDefinition {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        match &self.condition {
            SignalConditionalCondition::DeltaThreshold(threshold) => threshold
                .unit_identity
                .retained_heap_charge(work)?
                .checked_add(value_charge(&threshold.threshold, work)?),
            SignalConditionalCondition::Always
            | SignalConditionalCondition::AspectFilter(_)
            | SignalConditionalCondition::OnDemand
            | SignalConditionalCondition::RuntimePredicate
            | SignalConditionalCondition::TemporalWake => Ok(Charge::ZERO),
        }
    }
}
fn value_charge(value: &AspectValue, work: &mut Preparation) -> Result<Charge, Denial> {
    work.visit()?;
    match value {
        AspectValue::Decimal(value) => value.0.retained_heap_charge(work),
        AspectValue::BigInt(value) => value.0.retained_heap_charge(work),
        AspectValue::Rational(value) => value
            .numerator
            .0
            .retained_heap_charge(work)?
            .checked_add(value.denominator.0.retained_heap_charge(work)?),
        AspectValue::String(InternedString::Raw(value)) => value.retained_heap_charge(work),
        AspectValue::String(InternedString::Stored(value)) => {
            work.visit()?;
            let bytes = worth_foundational::facade::StoredValueText::fresh_decode_allocation_bytes(
                value.as_string().capacity(),
            )
            .ok_or(Denial::ChargeOverflow)?;
            Charge::capacity::<u8>(bytes)
        }
        AspectValue::InlineBytes(value) => {
            Charge::capacity::<u8>(value.owned_allocation_capacity_bytes())
        }
        AspectValue::Null
        | AspectValue::Bool(_)
        | AspectValue::Int8(_)
        | AspectValue::Int16(_)
        | AspectValue::Int32(_)
        | AspectValue::Int64(_)
        | AspectValue::UInt8(_)
        | AspectValue::UInt16(_)
        | AspectValue::UInt32(_)
        | AspectValue::UInt64(_)
        | AspectValue::Float32(_)
        | AspectValue::Float64(_)
        | AspectValue::String(InternedString::Symbol(_))
        | AspectValue::Uuid(_)
        | AspectValue::Date(_)
        | AspectValue::Time(_)
        | AspectValue::Timestamp(_)
        | AspectValue::TimestampTz(_)
        | AspectValue::EntityRef(_)
        | AspectValue::ContentRef(_) => Ok(Charge::ZERO),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::aspect::AspectMask;
    use crate::data::conditional_execution::*;

    #[test]
    fn conditional_source_charge_includes_literal_and_unit_allocation_capacities() {
        let mut literal = String::with_capacity(32_768);
        literal.push_str("authored literal");
        let mut units = String::with_capacity(8_192);
        units.push_str("authored units");
        let expected = literal.capacity() + units.capacity();
        let definition = SignalConditionalContractDefinition {
            condition: SignalConditionalCondition::DeltaThreshold(
                SignalDeltaThresholdContract::new(
                    AspectValue::String(InternedString::Raw(literal)),
                    units,
                    SignalThresholdValueFamily::Integer,
                    SignalThresholdComparisonDomain::AbsoluteDifference,
                    SignalThresholdBoundary::Inclusive,
                ),
            ),
            dependency_aspects: AspectMask::ALL,
            trigger_aspects: AspectMask::ALL,
            dependency_comparator: SignalConditionalVersionComparator::Exact,
            output_comparator: SignalConditionalVersionComparator::Exact,
            artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
        };
        let charge = definition
            .retained_heap_charge(&mut Preparation::new(None))
            .unwrap();
        assert_eq!(charge.bytes(), expected as u64);
        assert!(
            definition.installation_retained_bytes()
                >= expected + std::mem::size_of_val(&definition)
        );
    }
}

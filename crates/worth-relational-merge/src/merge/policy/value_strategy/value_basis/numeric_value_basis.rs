use worth_foundational::facade::AspectValue;

use super::{PolicyAspectValueBasis, PolicyScalarValue, PolicyValueLookupFailure};

pub(crate) struct PolicyNumericValueBasis<'a> {
    basis: &'a PolicyAspectValueBasis,
}

impl<'a> PolicyNumericValueBasis<'a> {
    pub(crate) const fn new(basis: &'a PolicyAspectValueBasis) -> Self {
        Self { basis }
    }

    pub(crate) fn source_i64(&self) -> Result<i64, PolicyValueLookupFailure> {
        scalar_i64(self.basis.source()?)
    }

    pub(crate) fn target_i64(&self) -> Result<i64, PolicyValueLookupFailure> {
        scalar_i64(self.basis.target()?)
    }

    pub(crate) fn base_i64(&self) -> Result<i64, PolicyValueLookupFailure> {
        scalar_i64(self.basis.base()?)
    }
}

fn scalar_i64(value: &PolicyScalarValue) -> Result<i64, PolicyValueLookupFailure> {
    let _proof_locator = value.locator();
    let _canonical_locator_basis = value.locator_basis();
    let _value_provenance = value.provenance();
    let _foundational_source_basis = value.source_basis();
    match value.value() {
        AspectValue::Int8(value) => Ok(i64::from(*value)),
        AspectValue::Int16(value) => Ok(i64::from(*value)),
        AspectValue::Int32(value) => Ok(i64::from(*value)),
        AspectValue::Int64(value) => Ok(*value),
        AspectValue::UInt8(value) => Ok(i64::from(*value)),
        AspectValue::UInt16(value) => Ok(i64::from(*value)),
        AspectValue::UInt32(value) => Ok(i64::from(*value)),
        AspectValue::UInt64(value) => {
            i64::try_from(*value).map_err(|_| PolicyValueLookupFailure::InvalidValueShape)
        }
        _ => Err(PolicyValueLookupFailure::InvalidValueShape),
    }
}

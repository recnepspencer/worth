use std::sync::Arc;

use super::{
    RetainedStorageCharge as Charge, RetainedStoragePreparation as Preparation,
    RetainedStoragePreparationDenial as Denial,
};

/// Heap representation reachable from a payload, excluding the inline payload
/// itself. Owners charge allocated capacity separately from initialized values.
/// Implementations must not permit shared interior mutation of measured extents;
/// size-changing writes belong to the owning storage's accounted mutation path.
pub(crate) trait RetainedStorageMeasurement {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial>;
}

macro_rules! inline_only {
    ($($ty:ty),* $(,)?) => {$ (
        impl RetainedStorageMeasurement for $ty {
            fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
                work.visit()?;
                Ok(Charge::ZERO)
            }
        }
    )*};
}

inline_only!((), bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

impl<T: RetainedStorageMeasurement, const N: usize> RetainedStorageMeasurement for [T; N] {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = Charge::ZERO;
        for value in self {
            charge = charge.checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

impl RetainedStorageMeasurement for String {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        Charge::capacity::<u8>(self.capacity())
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for Vec<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = Charge::capacity::<T>(self.capacity())?;
        for value in self {
            charge = charge.checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for Option<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.as_ref()
            .map_or(Ok(Charge::ZERO), |value| value.retained_heap_charge(work))
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for Arc<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        arc_allocation_charge::<T>()?.checked_add(self.as_ref().retained_heap_charge(work)?)
    }
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for Box<T> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        Charge::capacity::<T>(1)?.checked_add(self.as_ref().retained_heap_charge(work)?)
    }
}

impl<A: RetainedStorageMeasurement, B: RetainedStorageMeasurement> RetainedStorageMeasurement
    for (A, B)
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.0
            .retained_heap_charge(work)?
            .checked_add(self.1.retained_heap_charge(work)?)
    }
}

/// Arc stores two reference counters and an aligned payload. Two alignment
/// allowances conservatively cover header-to-payload and trailing padding.
pub(crate) fn arc_allocation_charge<T>() -> Result<Charge, Denial> {
    Charge::capacity::<usize>(2)?
        .checked_add(Charge::capacity::<T>(1)?)?
        .checked_add(
            Charge::capacity::<u8>(std::mem::align_of::<T>().max(std::mem::align_of::<usize>()))?
                .checked_mul(2)?,
        )
}

impl<T: RetainedStorageMeasurement> RetainedStorageMeasurement for Arc<[T]> {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let alignment = std::mem::align_of::<T>().max(std::mem::align_of::<usize>());
        let mut charge = Charge::capacity::<usize>(2)?
            .checked_add(Charge::capacity::<T>(self.len())?)?
            .checked_add(Charge::capacity::<u8>(alignment)?.checked_mul(2)?)?;
        for value in self.iter() {
            charge = charge.checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

impl<K: RetainedStorageMeasurement, V: RetainedStorageMeasurement> RetainedStorageMeasurement
    for std::collections::BTreeMap<K, V>
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = super::btree_structure_charge::<K, V>(self.len())?;
        for (key, value) in self {
            charge = charge
                .checked_add(key.retained_heap_charge(work)?)?
                .checked_add(value.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

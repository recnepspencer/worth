use smallvec::{Array, SmallVec};

use super::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl<A: Array> RetainedStorageMeasurement for SmallVec<A>
where
    A::Item: RetainedStorageMeasurement,
{
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let mut charge = if self.spilled() {
            Charge::capacity::<A::Item>(self.capacity())?
        } else {
            Charge::ZERO
        };
        for item in self {
            charge = charge.checked_add(item.retained_heap_charge(work)?)?;
        }
        Ok(charge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_vector_charge_distinguishes_inline_payloads_from_retained_spill_capacity() {
        let mut values = SmallVec::<[String; 2]>::new();
        let payload = String::with_capacity(4_096);
        let payload_capacity = payload.capacity() as u64;
        values.push(payload);
        assert!(!values.spilled());
        assert_eq!(
            values
                .retained_heap_charge(&mut Preparation::new(2))
                .unwrap()
                .bytes(),
            payload_capacity
        );
        assert!(matches!(
            values.retained_heap_charge(&mut Preparation::new(1)),
            Err(Denial::WorkExhausted { maximum_visits: 1 })
        ));

        values.reserve(32);
        assert!(values.spilled());
        let spill_bytes = (values.capacity() * std::mem::size_of::<String>()) as u64;
        assert_eq!(
            values
                .retained_heap_charge(&mut Preparation::new(2))
                .unwrap()
                .bytes(),
            payload_capacity + spill_bytes
        );
        values.clear();
        assert_eq!(
            values
                .retained_heap_charge(&mut Preparation::new(1))
                .unwrap()
                .bytes(),
            spill_bytes
        );
        values.shrink_to_fit();
        assert_eq!(
            values
                .retained_heap_charge(&mut Preparation::new(1))
                .unwrap(),
            Charge::ZERO
        );
    }
}

//! Request-owned captured source preimage for a body-created field.
use super::super::application_contribution::QueryMemoryReservation;
use worth_foundational::facade::AspectValue;

pub struct PreparedBodySource {
    pub value: AspectValue,
    pub _reservation: QueryMemoryReservation,
}

impl PreparedBodySource {
    pub fn new(value: AspectValue, reservation: QueryMemoryReservation) -> Self {
        Self {
            value,
            _reservation: reservation,
        }
    }

    pub fn into_parts(self) -> (AspectValue, QueryMemoryReservation) {
        (self.value, self._reservation)
    }
}

use worth_foundational::facade::{AspectContract, AspectValue};
use worth_query_installation::facade::WorthQueryArtifactNativeAlignment;

use super::provider_port::value_matches_contract;
use super::WorthQueryArtifactProviderAccessDenial;

pub struct WorthQueryArtifactProjectionSink {
    fields: Vec<AspectContract>,
    max_rows: usize,
    values: Vec<AspectValue>,
    pending_row: Vec<AspectValue>,
}

impl WorthQueryArtifactProjectionSink {
    pub(crate) fn new(
        fields: Vec<AspectContract>,
        max_rows: usize,
        required_alignment: WorthQueryArtifactNativeAlignment,
    ) -> Result<Self, WorthQueryArtifactProviderAccessDenial> {
        let value_capacity = max_rows.saturating_mul(fields.len());
        let sink = Self {
            pending_row: Vec::with_capacity(fields.len()),
            fields,
            max_rows,
            values: Vec::with_capacity(value_capacity),
        };
        if !sink.satisfies_alignment(required_alignment) {
            return Err(WorthQueryArtifactProviderAccessDenial::AlignmentMismatch);
        }
        Ok(sink)
    }

    pub fn push_row(
        &mut self,
        values: impl IntoIterator<Item = AspectValue>,
    ) -> Result<(), WorthQueryArtifactProviderAccessDenial> {
        if self.row_count() >= self.max_rows {
            return Err(WorthQueryArtifactProviderAccessDenial::BoundsExceeded);
        }
        self.pending_row.clear();
        for value in values {
            let Some(contract) = self.fields.get(self.pending_row.len()) else {
                return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            };
            if !value_matches_contract(&value, contract) {
                return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
            }
            self.pending_row.push(value);
        }
        if self.pending_row.len() != self.fields.len() {
            return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
        }
        self.values.append(&mut self.pending_row);
        Ok(())
    }

    pub fn row_count(&self) -> usize {
        if self.fields.is_empty() {
            0
        } else {
            self.values.len() / self.fields.len()
        }
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    pub fn row(&self, row: usize) -> Option<&[AspectValue]> {
        let start = row.checked_mul(self.fields.len())?;
        self.values.get(start..start + self.fields.len())
    }

    pub fn allocated_capacity_bytes(&self) -> usize {
        let outer_capacity = self
            .values
            .capacity()
            .saturating_add(self.pending_row.capacity())
            .saturating_mul(std::mem::size_of::<AspectValue>());
        self.values
            .iter()
            .chain(&self.pending_row)
            .fold(outer_capacity, |capacity, value| {
                capacity.saturating_add(value.owned_allocation_capacity_bytes())
            })
    }

    pub(crate) fn result_semantic_bytes(&self) -> usize {
        self.values
            .iter()
            .map(AspectValue::semantic_byte_width)
            .sum()
    }

    fn satisfies_alignment(&self, required: WorthQueryArtifactNativeAlignment) -> bool {
        let bytes = required.bytes();
        bytes <= std::mem::align_of::<AspectValue>()
            && (self.values.as_ptr() as usize).is_multiple_of(bytes)
            && (self.pending_row.as_ptr() as usize).is_multiple_of(bytes)
    }
}

mod court_doors_for_test;

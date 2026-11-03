use worth_foundational::facade::{AspectKey, AspectShape};

use super::thread_bound::WorthQueryArtifactThreadBound;
use super::{
    WorthQueryArtifactNativeAccessAdmission, WorthQueryArtifactNativeAccessCounters,
    WorthQueryArtifactNativeAccessDenial, WorthQueryArtifactNativeValueView,
    WorthQueryArtifactProviderAccessDenial, WorthQueryArtifactProviderFieldSlice,
};

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryArtifactNativeFieldSlice<'a> {
    values: WorthQueryArtifactProviderFieldSlice<'a>,
    _thread_bound: WorthQueryArtifactThreadBound,
}

impl<'a> WorthQueryArtifactNativeFieldSlice<'a> {
    pub(crate) const fn from_provider(values: WorthQueryArtifactProviderFieldSlice<'a>) -> Self {
        Self {
            values,
            _thread_bound: WorthQueryArtifactThreadBound::new(),
        }
    }

    pub fn len(self) -> usize {
        self.values.len()
    }

    pub fn is_empty(self) -> bool {
        self.values.is_empty()
    }

    pub fn value(self, row: usize) -> Option<WorthQueryArtifactNativeValueView<'a>> {
        Some(WorthQueryArtifactNativeValueView::from_provider(
            self.values.value(row)?,
        ))
    }
}

pub(crate) fn with_borrowed_field<T>(
    admission: &mut WorthQueryArtifactNativeAccessAdmission<'_>,
    start_row: usize,
    max_rows: usize,
    field: &AspectKey,
    consume: impl for<'view> FnOnce(WorthQueryArtifactNativeFieldSlice<'view>) -> T,
) -> Result<T, WorthQueryArtifactNativeAccessDenial> {
    let layout = admission.native_contract().layout().clone();
    let field = field.clone();
    let (value, increment) = admission.with_provider(|provider, session| {
        let values = provider.borrow_field(session, start_row, max_rows, &field)?;
        if values.len() > max_rows {
            return Err(WorthQueryArtifactProviderAccessDenial::BoundsExceeded);
        }
        let Some(contract) = layout
            .fields()
            .iter()
            .find(|contract| contract.aspect().key() == &field)
        else {
            return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
        };
        let matches = match contract.aspect().shape() {
            AspectShape::Scalar(family) => values.matches_scalar_family(*family),
            AspectShape::Struct(_) => values.is_struct(),
            AspectShape::Opaque(_) | AspectShape::Reference(_) => false,
        };
        if !matches {
            return Err(WorthQueryArtifactProviderAccessDenial::ShapeMismatch);
        }
        let increment = WorthQueryArtifactNativeAccessCounters {
            field_slice_contacts: 1,
            rows_exposed: values.len(),
            values_exposed: values.len(),
            source_bytes: values.physical_bytes(),
            ..WorthQueryArtifactNativeAccessCounters::default()
        };
        Ok((
            consume(WorthQueryArtifactNativeFieldSlice::from_provider(values)),
            increment,
        ))
    })?;
    admission.counters_mut().accumulate(increment);
    Ok(value)
}

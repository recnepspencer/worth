use super::thread_bound::WorthQueryArtifactThreadBound;
use super::WorthQueryArtifactProviderValueView;

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryArtifactNativeValueView<'a> {
    value: WorthQueryArtifactProviderValueView<'a>,
    _thread_bound: WorthQueryArtifactThreadBound,
}

impl<'a> WorthQueryArtifactNativeValueView<'a> {
    pub const fn from_provider(value: WorthQueryArtifactProviderValueView<'a>) -> Self {
        Self {
            value,
            _thread_bound: WorthQueryArtifactThreadBound::new(),
        }
    }

    pub fn as_u64(self) -> Option<u64> {
        match self.value {
            WorthQueryArtifactProviderValueView::UInt64(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(self) -> Option<f64> {
        match self.value {
            WorthQueryArtifactProviderValueView::Float64(value) => Some((*value).as_f64()),
            _ => None,
        }
    }
}

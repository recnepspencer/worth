//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryGraphReadStreamAccumulator {
    #[doc(hidden)]
    pub fn admit_chunk_for_test(&mut self, material: WorthQueryGraphReadMaterial) -> usize {
        Self::admit_chunk(self, material)
    }
}

impl WorthQueryGraphReadStreamAccumulator {
    #[doc(hidden)]
    pub fn finish_for_test(
        self,
        call: &WorthQueryGraphProviderCall,
    ) -> WorthQueryExecutionGraphReadStreamEvidence {
        Self::finish(self, call)
    }
}

impl WorthQueryGraphReadStreamAccumulator {
    #[doc(hidden)]
    pub fn new_for_test(_call: &WorthQueryGraphProviderCall) -> Self {
        Self::new(_call)
    }
}

use std::num::NonZeroUsize;

use worth_query_host::facade::application_entry::WorthQueryOutputDemandControls;

pub(super) fn output_controls() -> WorthQueryOutputDemandControls {
    WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
}

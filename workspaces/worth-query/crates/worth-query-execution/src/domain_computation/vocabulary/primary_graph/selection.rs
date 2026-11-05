//! Moved whole from `domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::preimage_retention::selection`: the items below and every impl they own.
use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_query_installation::facade::InstalledPreImageLocus;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::transactions::RecordRef;

pub(crate) struct WorthQueryRetainedPreImageFieldSeal {
    locus: InstalledPreImageLocus,
    value: AspectValue,
    encoded_bytes: usize,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

pub(crate) struct WorthQueryRetainedPreImageSeal {
    fields: Vec<WorthQueryRetainedPreImageFieldSeal>,
    total_encoded_bytes: usize,
}

impl WorthQueryRetainedPreImageSeal {
    pub(crate) fn into_parts(self) -> (Vec<WorthQueryRetainedPreImageFieldSeal>, usize) {
        (self.fields, self.total_encoded_bytes)
    }
}

impl WorthQueryRetainedPreImageFieldSeal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        InstalledPreImageLocus,
        AspectValue,
        usize,
        RecordRef,
        KindId,
        AspectFieldLocator,
    ) {
        (
            self.locus,
            self.value,
            self.encoded_bytes,
            self.target_record,
            self.entity_kind,
            self.locator,
        )
    }
}

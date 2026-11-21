//! Preimage seal constructors on `WorthQueryRetainedPreImage`.

use worth_query_execution_lineage::facade::domain_computation::application_aftermath::retained_preimage::WorthQueryRetainedPreImageField;
use crate::domain_computation::application_aftermath::{
    retained_relations::RetainedRelations, WorthQueryRetainedPreImage,
};
use crate::domain_computation::primary_graph::{
    WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal,
};

pub(crate) trait RetainedPreImageExt: Sized {
    fn from_retention_seal(seal: WorthQueryRetainedPreImageSeal) -> Self;

    fn with_relation_seal(
        self,
        seal: WorthQueryRetainedRelationPreImageSeal,
        bytes: usize,
    ) -> Self;
}

impl RetainedPreImageExt for WorthQueryRetainedPreImage {
    fn from_retention_seal(seal: WorthQueryRetainedPreImageSeal) -> Self {
        let (fields, absences, output_fields, total_encoded_bytes) = seal.into_parts();
        let fields = fields
            .into_iter()
            .map(|field| {
                let (locus, value, encoded_bytes, target_record, entity_kind, locator) =
                    field.into_parts();
                WorthQueryRetainedPreImageField::from_parts(
                    locus,
                    value,
                    encoded_bytes,
                    target_record,
                    entity_kind,
                    locator,
                )
            })
            .collect();
        Self::from_parts(fields, absences, output_fields, total_encoded_bytes)
    }

    fn with_relation_seal(
        self,
        seal: WorthQueryRetainedRelationPreImageSeal,
        bytes: usize,
    ) -> Self {
        let (rows, loci, footprint, memory) = seal.into_parts();
        let relations = RetainedRelations::from_parts(rows, loci, footprint, memory);
        self.with_retained_relations(relations, bytes)
    }
}

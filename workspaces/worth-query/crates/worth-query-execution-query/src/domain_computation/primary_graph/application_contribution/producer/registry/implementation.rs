//! Exact installed producer edition comparison is independent of checkpoint decoding.
use super::*;
impl<Schema: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<Schema> {
    pub(crate) fn implementation_matches(
        &self,
        producer: &str,
        original: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
    ) -> bool {
        self.entries.get(producer).is_some_and(|installed| {
            installed.edition.digest() == original.edition()
                && installed.edition.build_identity().bytes() == original.build_identity()
        })
    }
}

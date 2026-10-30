//! Provider callbacks have their own installed purpose, separate from Query reads.
use super::{
    WorthQueryGraphObligationInstallationDenial,
    WorthQueryInstalledGraphObligationContract as Contract,
    WorthQueryInstalledGraphObligationResourcePosture as Resources,
    WorthQueryInstalledGraphObligationSet,
};
use crate::application_query::WorthQueryInstalledManagedIndexReadMeaning;
use std::sync::Arc;

pub(crate) fn bind_managed_index_obligations(
    meaning: &Arc<WorthQueryInstalledManagedIndexReadMeaning>,
) -> Result<WorthQueryInstalledGraphObligationSet, WorthQueryGraphObligationInstallationDenial> {
    WorthQueryInstalledGraphObligationSet::for_managed_index(
        meaning.binding_identity(),
        meaning.definition().parts().name.clone(),
        meaning.parent_binding().to_owned(),
        vec![Contract::ManagedIndexRead {
            meaning: Arc::clone(meaning),
        }],
        Resources::ApplicationManagedIndex {
            maximum_source_results: meaning.maximum_source_results(),
        },
    )
    .map_err(Into::into)
}

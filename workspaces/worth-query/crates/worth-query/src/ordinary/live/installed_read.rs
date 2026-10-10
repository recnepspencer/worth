use crate::ordinary::read::{current, WorthQueryReadDeclaration};
use crate::runtime::WorthQueryWorkspace;

use super::{WorthQueryLiveDeclaration, WorthQueryLiveOpenOutcome};

/// Open a managed live resource from Query's installation-validated read.
///
/// This seam deliberately accepts no authoring closure. The operation
/// installation remains the sole source of read meaning while the ordinary
/// live path continues to own context admission, planning, lowering, and
/// managed-resource registration.
pub(crate) fn open_installed_read_live(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    resource_name: String,
    read: WorthQueryReadDeclaration,
    workspace: &mut WorthQueryWorkspace,
) -> WorthQueryLiveOpenOutcome {
    WorthQueryLiveDeclaration::from_installed_read(resource_name, read)
        .using(current())
        .open(request_execution, workspace)
}

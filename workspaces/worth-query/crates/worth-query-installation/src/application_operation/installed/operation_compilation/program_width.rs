//! Checked authored declaration width at the first installation owner.

use crate::package::WorthQueryPortableApplicationOperationContractRecord;

use super::{
    operation_denial, WorthQueryApplicationOperationInstallationDenial,
    WorthQueryApplicationOperationInstallationDenialKind,
};

pub(super) fn resolve(
    contract: &WorthQueryPortableApplicationOperationContractRecord,
    operation: &str,
) -> Result<usize, WorthQueryApplicationOperationInstallationDenial> {
    contract.authored_program_width().ok_or_else(|| {
        operation_denial(
            WorthQueryApplicationOperationInstallationDenialKind::InvalidGraphObligationContract,
            operation,
        )
    })
}

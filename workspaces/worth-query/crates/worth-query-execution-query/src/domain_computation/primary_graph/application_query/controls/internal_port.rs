//! Crate-boundary internal port for `WorthQueryAdmittedApplicationQueryControls`.
//! Never exported on `worth-query-execution::facade` or Host.

use super::WorthQueryAdmittedApplicationQueryControls;

pub trait AdmittedApplicationQueryControlsInternalPort {
    fn is_current_product_one_shot(&self) -> bool;
}

impl AdmittedApplicationQueryControlsInternalPort
    for WorthQueryAdmittedApplicationQueryControls<'_>
{
    fn is_current_product_one_shot(&self) -> bool {
        Self::is_current_product_one_shot(self)
    }
}

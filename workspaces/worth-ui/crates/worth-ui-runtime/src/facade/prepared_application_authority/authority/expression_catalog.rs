use std::sync::Arc;

use super::WorthUiPreparedApplicationAuthority;
use crate::runtime::expression::UiExpressionCatalog;

impl WorthUiPreparedApplicationAuthority {
    /// The expressions installed for this prepared generation.
    pub(crate) fn expression_catalog(&self) -> &Arc<UiExpressionCatalog> {
        &self.expression_catalog
    }
}

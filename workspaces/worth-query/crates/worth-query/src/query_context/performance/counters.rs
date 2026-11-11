#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QueryContextCounters {
    query_basis_binding_count: usize,
    historical_basis_lookup_count: usize,
    comparison_basis_lookup_count: usize,
    materialization_path_compatibility_check_count: usize,
    comparison_scope_width: usize,
    comparison_row_width: usize,
    diff_input_breadth: usize,
    diff_change_set_row_width: usize,
    unsupported_basis_denial_count: usize,
    basis_substitution_denial_count: usize,
    comparison_broadening_denial_count: usize,
    historical_broadening_denial_count: usize,
    basis_binding_width: usize,
    historical_lookup_width: usize,
    denial_width: usize,
    basis_rediscovery_count: usize,
    historical_path_rediscovery_count: usize,
    comparison_family_rediscovery_count: usize,
}

impl QueryContextCounters {
    pub(crate) fn for_identity_refusal(historical_lookup: bool) -> Self {
        let mut counters = Self::for_denial(historical_lookup, false);
        counters.unsupported_basis_denial_count = 0;
        counters
    }
    pub fn query_basis_binding_count(&self) -> usize {
        self.query_basis_binding_count
    }

    pub fn historical_basis_lookup_count(&self) -> usize {
        self.historical_basis_lookup_count
    }

    pub fn comparison_basis_lookup_count(&self) -> usize {
        self.comparison_basis_lookup_count
    }

    pub fn materialization_path_compatibility_check_count(&self) -> usize {
        self.materialization_path_compatibility_check_count
    }

    pub fn comparison_scope_width(&self) -> usize {
        self.comparison_scope_width
    }

    pub fn comparison_row_width(&self) -> usize {
        self.comparison_row_width
    }

    pub fn diff_input_breadth(&self) -> usize {
        self.diff_input_breadth
    }

    pub fn diff_change_set_row_width(&self) -> usize {
        self.diff_change_set_row_width
    }

    pub fn unsupported_basis_denial_count(&self) -> usize {
        self.unsupported_basis_denial_count
    }

    pub fn basis_substitution_denial_count(&self) -> usize {
        self.basis_substitution_denial_count
    }

    pub fn comparison_broadening_denial_count(&self) -> usize {
        self.comparison_broadening_denial_count
    }

    pub fn historical_broadening_denial_count(&self) -> usize {
        self.historical_broadening_denial_count
    }

    pub fn basis_binding_width(&self) -> usize {
        self.basis_binding_width
    }

    pub fn historical_lookup_width(&self) -> usize {
        self.historical_lookup_width
    }

    pub fn denial_width(&self) -> usize {
        self.denial_width
    }

    pub fn basis_rediscovery_count(&self) -> usize {
        self.basis_rediscovery_count
    }

    pub fn historical_path_rediscovery_count(&self) -> usize {
        self.historical_path_rediscovery_count
    }

    pub fn comparison_family_rediscovery_count(&self) -> usize {
        self.comparison_family_rediscovery_count
    }

    pub fn basis_context_binding_count(&self) -> usize {
        self.query_basis_binding_count()
    }

    pub(crate) fn for_runtime_basis_binding() -> Self {
        Self {
            query_basis_binding_count: 1,
            historical_basis_lookup_count: 0,
            comparison_basis_lookup_count: 0,
            materialization_path_compatibility_check_count: 0,
            comparison_scope_width: 0,
            comparison_row_width: 0,
            diff_input_breadth: 0,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 0,
            basis_substitution_denial_count: 0,
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 0,
            basis_binding_width: 1,
            historical_lookup_width: 0,
            denial_width: 0,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_historical_basis_binding() -> Self {
        Self {
            query_basis_binding_count: 1,
            historical_basis_lookup_count: 1,
            comparison_basis_lookup_count: 0,
            materialization_path_compatibility_check_count: 1,
            comparison_scope_width: 0,
            comparison_row_width: 0,
            diff_input_breadth: 0,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 0,
            basis_substitution_denial_count: 0,
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 0,
            basis_binding_width: 1,
            historical_lookup_width: 1,
            denial_width: 0,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_preview_basis_binding() -> Self {
        Self {
            query_basis_binding_count: 1,
            historical_basis_lookup_count: 0,
            comparison_basis_lookup_count: 0,
            materialization_path_compatibility_check_count: 0,
            comparison_scope_width: 0,
            comparison_row_width: 0,
            diff_input_breadth: 0,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 0,
            basis_substitution_denial_count: 0,
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 0,
            basis_binding_width: 1,
            historical_lookup_width: 0,
            denial_width: 0,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_denial(historical_lookup: bool, substitution: bool) -> Self {
        Self {
            query_basis_binding_count: 0,
            historical_basis_lookup_count: usize::from(historical_lookup),
            comparison_basis_lookup_count: 0,
            materialization_path_compatibility_check_count: usize::from(historical_lookup),
            comparison_scope_width: 0,
            comparison_row_width: 0,
            diff_input_breadth: 0,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 1,
            basis_substitution_denial_count: usize::from(substitution),
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 0,
            basis_binding_width: 0,
            historical_lookup_width: usize::from(historical_lookup),
            denial_width: 1,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_diff(comparison_row_width: usize) -> Self {
        Self {
            query_basis_binding_count: 0,
            historical_basis_lookup_count: 0,
            comparison_basis_lookup_count: 1,
            materialization_path_compatibility_check_count: 0,
            comparison_scope_width: 2,
            comparison_row_width,
            diff_input_breadth: 2,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 0,
            basis_substitution_denial_count: 0,
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 0,
            basis_binding_width: 0,
            historical_lookup_width: 0,
            denial_width: 0,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_diff_denial(substitution: bool, broadening: bool) -> Self {
        Self {
            query_basis_binding_count: 0,
            historical_basis_lookup_count: 0,
            comparison_basis_lookup_count: 1,
            materialization_path_compatibility_check_count: 0,
            comparison_scope_width: 2,
            comparison_row_width: 0,
            diff_input_breadth: 2,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 1,
            basis_substitution_denial_count: usize::from(substitution),
            comparison_broadening_denial_count: usize::from(broadening),
            historical_broadening_denial_count: 0,
            basis_binding_width: 0,
            historical_lookup_width: 0,
            denial_width: 1,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }

    pub(crate) fn for_historical_broadening_denial() -> Self {
        Self {
            query_basis_binding_count: 0,
            historical_basis_lookup_count: 1,
            comparison_basis_lookup_count: 0,
            materialization_path_compatibility_check_count: 1,
            comparison_scope_width: 0,
            comparison_row_width: 0,
            diff_input_breadth: 0,
            diff_change_set_row_width: 0,
            unsupported_basis_denial_count: 0,
            basis_substitution_denial_count: 0,
            comparison_broadening_denial_count: 0,
            historical_broadening_denial_count: 1,
            basis_binding_width: 0,
            historical_lookup_width: 1,
            denial_width: 1,
            basis_rediscovery_count: 0,
            historical_path_rediscovery_count: 0,
            comparison_family_rediscovery_count: 0,
        }
    }
}

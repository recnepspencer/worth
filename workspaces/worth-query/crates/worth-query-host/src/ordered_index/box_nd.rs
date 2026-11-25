//! Shared domain implementation macro for N-dimensional stock AABB managed indexes.
macro_rules! impl_box_domain {
    (
        $Domain:ident,
        $ProbeTrait:ident,
        $probe_fn:ident,
        $D:literal,
        [$(($min_key:ident, $min_member:ident)),+],
        [$(($max_key:ident, $max_member:ident)),+]
    ) => {
        impl<S, I, $($min_key,)+ $($max_key,)+ C>
            $Domain<S, I, $($min_key,)+ $($max_key,)+ C>
        where
            S: worth_query_declaration::facade::application_schema::ApplicationSchema,
            I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
            C: crate::ordered_index::coord::WorthQueryOrderedCoord,
            $($min_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
            $($max_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
        {
            /// The row's `(min, max)` corners, or `None` when any endpoint is absent.
            fn read_bounds(
                &self,
                row: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                Option<([C; $D], [C; $D])>,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<
                    crate::ordered_index::coord::WorthQueryStockIndexDenial,
                >,
            > {
                Ok(Some((
                    [$(match self.$min_member.read_key(row, control)? {
                        Some(value) => value,
                        None => return Ok(None),
                    }),+],
                    [$(match self.$max_member.read_key(row, control)? {
                        Some(value) => value,
                        None => return Ok(None),
                    }),+],
                )))
            }
        }

        impl<S, I, $($min_key,)+ $($max_key,)+ C>
            crate::ordered_index::WorthQueryOrderedIndexDomain<S, I>
            for $Domain<S, I, $($min_key,)+ $($max_key,)+ C>
        where
            S: worth_query_declaration::facade::application_schema::ApplicationSchema,
            I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
            C: crate::ordered_index::coord::WorthQueryOrderedCoord,
            worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>:
                $ProbeTrait<C>,
            $($min_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
            $($max_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
        {
            type Summary = crate::ordered_index::coord::WorthQueryAxisEnvelope<C, $D>;
            type Denial = crate::ordered_index::coord::WorthQueryStockIndexDenial;
            const IDENTITY: &'static str = I::PROVIDER;
            const VERSION: u32 = I::PROVIDER_VERSION;

            fn compare_rows(
                &self,
                left: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                right: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                std::cmp::Ordering,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                let left = self.read_bounds(&left, control)?;
                let right = self.read_bounds(&right, control)?;
                control.checkpoint(1)?;
                Ok(crate::ordered_index::coord::WorthQueryAxisEnvelope::row_cmp(left, right))
            }

            fn source_summary(
                &self,
                _slot: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceSlot,
                row: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                Self::Summary,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                let bounds = self.read_bounds(&row, control)?;
                control.checkpoint(1)?;
                Ok(crate::ordered_index::coord::WorthQueryAxisEnvelope::from_row_bounds(
                    bounds,
                    self.inclusion,
                ))
            }

            fn combine_summaries(
                &self,
                left: &Self::Summary,
                right: &Self::Summary,
                _sources: &crate::ordered_index::WorthQueryOrderedIndexSources<'_, '_, S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                Self::Summary,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                control.checkpoint(1)?;
                Ok(left.combine(*right))
            }

            fn validate_probe(
                &self,
                probe: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                (),
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                control.checkpoint(1)?;
                let (min, max) = probe.$probe_fn();
                if !crate::ordered_index::coord::WorthQueryAxisEnvelope::bounds_valid(
                    min,
                    max,
                    self.inclusion,
                ) {
                    return Err(
                        worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop::Domain(
                            crate::ordered_index::coord::WorthQueryStockIndexDenial::InvalidProbeBounds,
                        ),
                    );
                }
                Ok(())
            }

            fn summary_may_match(
                &self,
                summary: &Self::Summary,
                _sources: &crate::ordered_index::WorthQueryOrderedIndexSources<'_, '_, S, I>,
                probe: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                bool,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                control.checkpoint(1)?;
                let (min, max) = probe.$probe_fn();
                Ok(summary.may_overlap_bounds(min, max, self.inclusion))
            }

            fn matches_source(
                &self,
                row: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                probe: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                bool,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                let bounds = self.read_bounds(&row, control)?;
                control.checkpoint(1)?;
                let (p_min, p_max) = probe.$probe_fn();
                Ok(bounds.is_some_and(|(r_min, r_max)| {
                    crate::ordered_index::coord::WorthQueryAxisEnvelope::bounds_overlap(
                        r_min,
                        r_max,
                        p_min,
                        p_max,
                        self.inclusion,
                    )
                }))
            }
        }

        impl<S, I, $($min_key,)+ $($max_key,)+ C>
            crate::ordered_index::WorthQueryOrderedIndexPredicateDomain<S, I>
            for $Domain<S, I, $($min_key,)+ $($max_key,)+ C>
        where
            S: worth_query_declaration::facade::application_schema::ApplicationSchema,
            I: worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding<S>,
            C: crate::ordered_index::coord::WorthQueryOrderedCoord,
            worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>:
                $ProbeTrait<C>,
            $($min_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
            $($max_key: crate::ordered_index::key_field::WorthQueryOrderedKeyField<S, I, Value = C>,)+
        {
            type PredicateSummary = crate::ordered_index::coord::WorthQueryAxisEnvelope<C, $D>;

            fn compare_probes(
                &self,
                left: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                right: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                std::cmp::Ordering,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                control.checkpoint(1)?;
                let (l_min, l_max) = left.$probe_fn();
                let (r_min, r_max) = right.$probe_fn();
                Ok(crate::ordered_index::coord::WorthQueryAxisEnvelope::lex_cmp(
                    l_min, l_max, r_min, r_max,
                ))
            }

            fn predicate_summary(
                &self,
                probe: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                Self::PredicateSummary,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                control.checkpoint(1)?;
                let (min, max) = probe.$probe_fn();
                Ok(crate::ordered_index::coord::WorthQueryAxisEnvelope::from_bounds(
                    min,
                    max,
                    self.inclusion,
                ))
            }

            fn predicate_region_may_match(
                &self,
                summary: &Self::PredicateSummary,
                _extrema: [&worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProbeValue<S, I>; 2],
                row: worth_query_execution::facade::primary_graph::WorthQueryManagedIndexSourceRow<'_, S, I>,
                control: &worth_query_execution::facade::primary_graph::WorthQueryManagedIndexControl<'_>,
            ) -> Result<
                bool,
                worth_query_execution::facade::primary_graph::WorthQueryManagedIndexProviderStop<Self::Denial>,
            > {
                let bounds = self.read_bounds(&row, control)?;
                control.checkpoint(1)?;
                Ok(bounds.is_some_and(|(r_min, r_max)| {
                    summary.may_overlap_bounds(r_min, r_max, self.inclusion)
                }))
            }
        }
    };
}
pub(super) use impl_box_domain;

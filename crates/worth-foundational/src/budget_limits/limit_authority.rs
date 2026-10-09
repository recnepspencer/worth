//! The sealed authority of one budget owner: the one way to declare one. It
//! names no proof substrate, so a narrow mechanism crate whose dependencies
//! are exactly foundational plus its format can still own its limits.

/// Expansion support for [`limit_authority!`] only; not vocabulary. It
/// re-exports no proof type, so diagnostics keep naming proof types at home.
#[doc(hidden)]
pub mod __limit_authority {
    pub use worth_proof::authority_marker;
    use worth_proof::{AuthorityWitness, Performed};

    use crate::{BudgetRefused, ExhaustedLimit, LimitCounts, LimitDimension};

    /// Record the refusal under the owner's own witness and mint its limit.
    pub fn refuse<D: LimitDimension>(
        dimension: D,
        witness: &AuthorityWitness<D::Authority>,
        counts: LimitCounts,
    ) -> ExhaustedLimit<D> {
        ExhaustedLimit::refused(
            dimension,
            Performed::<BudgetRefused, _, _>::record(witness, counts),
        )
    }
}

/// Declare a budget owner's sealed limit authority in a *leaf* module.
///
/// Expands to the sealed marker plus one private door,
/// `refuse(dimension, counts)`, which records the refusal under the marker's
/// own witness and mints the [`ExhaustedLimit`](crate::ExhaustedLimit). Like the
/// witness, the door is private to the declaring module and its descendants,
/// and coherence still ties each dimension to exactly one authority.
///
/// ```
/// mod entries_budget {
///     use worth_foundational::{ExhaustedLimit, LimitCounts, LimitDimension};
///     worth_foundational::limit_authority!(pub EntriesAuthority);
///
///     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
///     pub enum EntriesBound { Entries }
///     impl LimitDimension for EntriesBound { type Authority = EntriesAuthority; }
///
///     pub fn admit(count: u64, admitted: u64) -> Result<(), ExhaustedLimit<EntriesBound>> {
///         if count <= admitted { return Ok(()); }
///         let counts = LimitCounts::new(count, admitted);
///         Err(EntriesAuthority::refuse(EntriesBound::Entries, counts))
///     }
/// }
///
/// let limit = entries_budget::admit(5, 4).unwrap_err();
/// assert_eq!((limit.observed(), limit.admitted()), (5, 4));
/// ```
#[macro_export]
macro_rules! limit_authority {
    ($(#[$meta:meta])* $vis:vis $name:ident) => {
        $crate::__limit_authority::authority_marker!($(#[$meta])* $vis $name);

        impl $name {
            /// Record this owner's budget refusal and mint its limit.
            #[allow(dead_code)]
            fn refuse<D>(dimension: D, counts: $crate::LimitCounts) -> $crate::ExhaustedLimit<D>
            where
                D: $crate::LimitDimension<Authority = Self>,
            {
                $crate::__limit_authority::refuse(dimension, &Self::witness(), counts)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    mod owner {
        use crate::{ExhaustedLimit, LimitCounts, LimitDimension};

        crate::limit_authority!(pub OwnerAuthority);

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OwnerBound {
            Entries,
        }

        impl LimitDimension for OwnerBound {
            type Authority = OwnerAuthority;
        }

        pub fn refuse(observed: u64, admitted: u64) -> ExhaustedLimit<OwnerBound> {
            OwnerAuthority::refuse(OwnerBound::Entries, LimitCounts::new(observed, admitted))
        }
    }

    #[test]
    fn the_declared_door_mints_its_own_dimension_with_both_counts() {
        let limit = owner::refuse(9, 4);
        assert_eq!(limit.dimension(), owner::OwnerBound::Entries);
        assert_eq!((limit.observed(), limit.admitted()), (9, 4));
    }
}

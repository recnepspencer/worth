//! Authority-free projection of canonical query meaning for portable records.

use crate::archive_decode_authority::ArchiveDecodeAuthority;
use crate::authoring::{QueryFamily, ResultShapeFamily, RootEntityKey};
use crate::binding::IdentityBindingDescriptor;
use crate::diagnostics::{CanonicalizationCounters, CanonicalizationReport};
use crate::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest};

use super::{
    CanonicalOrderingEntry, CanonicalPredicateEntry, CanonicalProjectionEntry,
    CanonicalQueryBundle, CanonicalResultField, CanonicalTraversalEntry,
};

/// Descriptive canonical query meaning with canonicalization authority removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableCanonicalQueryRecord {
    digest: CanonicalQueryDigest,
    family: QueryFamily,
    root: RootEntityKey,
    projection: Vec<CanonicalProjectionEntry>,
    predicates: Vec<CanonicalPredicateEntry>,
    ordering: Vec<CanonicalOrderingEntry>,
    traversal: Vec<CanonicalTraversalEntry>,
    identity_bindings: Vec<IdentityBindingDescriptor>,
}

impl WorthQueryPortableCanonicalQueryRecord {
    pub fn digest(&self) -> &CanonicalQueryDigest {
        &self.digest
    }

    pub const fn family(&self) -> &QueryFamily {
        &self.family
    }

    pub const fn root(&self) -> &RootEntityKey {
        &self.root
    }

    pub fn projection(&self) -> &[CanonicalProjectionEntry] {
        &self.projection
    }

    pub fn predicates(&self) -> &[CanonicalPredicateEntry] {
        &self.predicates
    }

    pub fn ordering(&self) -> &[CanonicalOrderingEntry] {
        &self.ordering
    }

    pub fn traversal(&self) -> &[CanonicalTraversalEntry] {
        &self.traversal
    }

    pub fn identity_bindings(&self) -> &[IdentityBindingDescriptor] {
        &self.identity_bindings
    }
}

/// Descriptive canonical result-shape meaning carried by a portable record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableCanonicalResultShapeRecord {
    digest: CanonicalResultShapeDigest,
    family: ResultShapeFamily,
    fields: Vec<CanonicalResultField>,
}

impl WorthQueryPortableCanonicalResultShapeRecord {
    pub fn digest(&self) -> &CanonicalResultShapeDigest {
        &self.digest
    }

    pub const fn family(&self) -> &ResultShapeFamily {
        &self.family
    }

    pub fn fields(&self) -> &[CanonicalResultField] {
        &self.fields
    }
}

/// Complete canonical bundle description without canonicalization proof.
///
/// The projection copies every query, result-shape, report, counter, and
/// identity-freeze field, but deliberately has no route back to
/// `QueryCanonicalAuthority`.
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     canonicalization::WorthQueryPortableCanonicalQueryBundleRecord,
/// };
/// fn cannot_recover_authority(record: &WorthQueryPortableCanonicalQueryBundleRecord) {
///     let _ = record.query().authority();
/// }
/// ```
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     canonicalization::WorthQueryPortableCanonicalQueryBundleRecord,
///     identity_authority::QueryCanonicalAuthority,
/// };
/// fn cannot_remint_authority(record: &WorthQueryPortableCanonicalQueryBundleRecord) {
///     let _ = QueryCanonicalAuthority::mint(record.query().digest().clone());
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortableCanonicalQueryBundleRecord {
    query: WorthQueryPortableCanonicalQueryRecord,
    result_shape: WorthQueryPortableCanonicalResultShapeRecord,
    report: CanonicalizationReport,
    counters: CanonicalizationCounters,
}

impl WorthQueryPortableCanonicalQueryBundleRecord {
    /// Constructs an authority-free record from decoded, untrusted fields.
    ///
    /// This performs no validation and mints no canonicalization authority.
    pub fn from_untrusted_parts(
        _: ArchiveDecodeAuthority,
        parts: WorthQueryPortableCanonicalQueryBundleParts,
    ) -> Self {
        Self {
            query: WorthQueryPortableCanonicalQueryRecord {
                digest: parts.query_digest,
                family: parts.query_family,
                root: parts.query_root,
                projection: parts.projection,
                predicates: parts.predicates,
                ordering: parts.ordering,
                traversal: parts.traversal,
                identity_bindings: parts.identity_bindings,
            },
            result_shape: WorthQueryPortableCanonicalResultShapeRecord {
                digest: parts.result_shape_digest,
                family: parts.result_shape_family,
                fields: parts.result_shape_fields,
            },
            report: parts.report,
            counters: parts.counters,
        }
    }

    pub fn project(source: &CanonicalQueryBundle) -> Self {
        let query = source.query();
        let result_shape = source.result_shape();
        Self {
            query: WorthQueryPortableCanonicalQueryRecord {
                digest: query.digest().clone(),
                family: query.family().clone(),
                root: query.root().clone(),
                projection: query.projection().to_vec(),
                predicates: query.predicates().to_vec(),
                ordering: query.ordering().to_vec(),
                traversal: query.traversal().to_vec(),
                identity_bindings: query.identity_bindings().to_vec(),
            },
            result_shape: WorthQueryPortableCanonicalResultShapeRecord {
                digest: result_shape.digest().clone(),
                family: result_shape.family().clone(),
                fields: result_shape.fields().to_vec(),
            },
            report: source.report().clone(),
            counters: source.counters().clone(),
        }
    }

    pub const fn query(&self) -> &WorthQueryPortableCanonicalQueryRecord {
        &self.query
    }

    pub const fn result_shape(&self) -> &WorthQueryPortableCanonicalResultShapeRecord {
        &self.result_shape
    }

    pub const fn report(&self) -> &CanonicalizationReport {
        &self.report
    }

    pub const fn counters(&self) -> &CanonicalizationCounters {
        &self.counters
    }

    pub fn into_parts(self) -> WorthQueryPortableCanonicalQueryBundleParts {
        WorthQueryPortableCanonicalQueryBundleParts {
            query_digest: self.query.digest,
            query_family: self.query.family,
            query_root: self.query.root,
            projection: self.query.projection,
            predicates: self.query.predicates,
            ordering: self.query.ordering,
            traversal: self.query.traversal,
            identity_bindings: self.query.identity_bindings,
            result_shape_digest: self.result_shape.digest,
            result_shape_family: self.result_shape.family,
            result_shape_fields: self.result_shape.fields,
            report: self.report,
            counters: self.counters,
        }
    }
}

/// Decoded fields for one authority-free portable canonical-query record.
pub struct WorthQueryPortableCanonicalQueryBundleParts {
    pub query_digest: CanonicalQueryDigest,
    pub query_family: QueryFamily,
    pub query_root: RootEntityKey,
    pub projection: Vec<CanonicalProjectionEntry>,
    pub predicates: Vec<CanonicalPredicateEntry>,
    pub ordering: Vec<CanonicalOrderingEntry>,
    pub traversal: Vec<CanonicalTraversalEntry>,
    pub identity_bindings: Vec<IdentityBindingDescriptor>,
    pub result_shape_digest: CanonicalResultShapeDigest,
    pub result_shape_family: ResultShapeFamily,
    pub result_shape_fields: Vec<CanonicalResultField>,
    pub report: CanonicalizationReport,
    pub counters: CanonicalizationCounters,
}

#[cfg(test)]
pub(super) enum WorthQueryPortableCanonicalQueryTestMutation {
    QueryDigest(CanonicalQueryDigest),
    ResultShapeDigest(CanonicalResultShapeDigest),
    DuplicateProjection,
    DuplicateResultField,
    ReplaceFirstResultField(CanonicalResultField),
    AliasSecondResultToFirst,
    ResultShapeFamily(ResultShapeFamily),
    IncrementWarningCount,
    FirstTraversalDepth(u8),
}

#[cfg(test)]
impl WorthQueryPortableCanonicalQueryBundleRecord {
    pub(super) fn project_for_test(
        source: &CanonicalQueryBundle,
        mutation: WorthQueryPortableCanonicalQueryTestMutation,
    ) -> Self {
        let mut record = Self::project(source);
        match mutation {
            WorthQueryPortableCanonicalQueryTestMutation::QueryDigest(digest) => {
                record.query.digest = digest;
            }
            WorthQueryPortableCanonicalQueryTestMutation::ResultShapeDigest(digest) => {
                record.result_shape.digest = digest;
            }
            WorthQueryPortableCanonicalQueryTestMutation::DuplicateProjection => record
                .query
                .projection
                .push(record.query.projection[0].clone()),
            WorthQueryPortableCanonicalQueryTestMutation::DuplicateResultField => record
                .result_shape
                .fields
                .push(record.result_shape.fields[0].clone()),
            WorthQueryPortableCanonicalQueryTestMutation::ReplaceFirstResultField(field) => {
                record.result_shape.fields[0] = field;
            }
            WorthQueryPortableCanonicalQueryTestMutation::AliasSecondResultToFirst => {
                record.result_shape.fields[1].delivered_name =
                    record.result_shape.fields[0].delivered_name.clone();
            }
            WorthQueryPortableCanonicalQueryTestMutation::ResultShapeFamily(family) => {
                record.result_shape.family = family;
            }
            WorthQueryPortableCanonicalQueryTestMutation::IncrementWarningCount => {
                record.counters.canonicalization_warning_count += 1;
            }
            WorthQueryPortableCanonicalQueryTestMutation::FirstTraversalDepth(depth) => {
                record.query.traversal[0].depth = depth;
            }
        }
        record
    }
}

#[cfg(test)]
mod tests {
    use crate::authoring::{
        AspectFieldSelector, AuthoredResultShapeField, EqualityPredicate, GuidedAuthoringPath,
        OrderingSelector, RawAuthoredQuery, RawAuthoredResultShape, RootEntityKey,
        TraversalSelector,
    };
    use crate::binding::{
        IdentityBindingDescriptor, QueryBindingDescriptor, QueryBindingSlot, QueryBindingSubject,
    };

    use super::{CanonicalQueryBundle, WorthQueryPortableCanonicalQueryBundleRecord};

    #[test]
    fn projection_retains_each_populated_descriptive_query_family() {
        let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("task").unwrap())
            .project(AspectFieldSelector::new("profile", "name").unwrap())
            .where_equal(EqualityPredicate::new("profile", "name", "Ada").unwrap())
            .order_by(OrderingSelector::ascending("profile", "name").unwrap())
            .traverse(TraversalSelector::bounded("owner", 2).unwrap())
            .build()
            .unwrap();
        let shape = RawAuthoredResultShape::detail_builder()
            .field(AuthoredResultShapeField::new("profile", "name", "name").unwrap())
            .build()
            .unwrap();
        let bindings = QueryBindingDescriptor::new().with_identity(IdentityBindingDescriptor::new(
            QueryBindingSlot::new("root").unwrap(),
            QueryBindingSubject::RootEntity,
        ));
        let source = crate::canonicalization::canonicalize_request(
            GuidedAuthoringPath::pair_detail_with_bindings(query, shape, bindings).unwrap(),
        )
        .unwrap();
        assert_every_query_collection_is_populated(&source);

        let projected = WorthQueryPortableCanonicalQueryBundleRecord::project(&source);
        let query = projected.query();
        assert_eq!(query.digest(), source.query().digest());
        assert_eq!(query.family(), source.query().family());
        assert_eq!(query.root(), source.query().root());
        assert_eq!(query.projection(), source.query().projection());
        assert_eq!(query.predicates(), source.query().predicates());
        assert_eq!(query.ordering(), source.query().ordering());
        assert_eq!(query.traversal(), source.query().traversal());
        assert_eq!(
            query.identity_bindings(),
            source.query().identity_bindings()
        );
        assert_eq!(
            projected.result_shape().digest(),
            source.result_shape().digest()
        );
        assert_eq!(
            projected.result_shape().family(),
            source.result_shape().family()
        );
        assert_eq!(
            projected.result_shape().fields(),
            source.result_shape().fields()
        );
        assert_eq!(projected.report(), source.report());
        assert_eq!(projected.counters(), source.counters());
        assert_eq!(
            projected.portable_record_logical_bytes(),
            source.portable_record_logical_bytes()
        );
    }

    fn assert_every_query_collection_is_populated(source: &CanonicalQueryBundle) {
        assert!(!source.query().projection().is_empty());
        assert!(!source.query().predicates().is_empty());
        assert!(!source.query().ordering().is_empty());
        assert!(!source.query().traversal().is_empty());
        assert!(!source.query().identity_bindings().is_empty());
        assert!(!source.result_shape().fields().is_empty());
        assert!(!source.report().events().is_empty());
    }
}

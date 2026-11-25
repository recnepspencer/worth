use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
mod diagnostics;
mod query;

use worth_query_declaration::facade::canonicalization::{
    WorthQueryPortableCanonicalQueryBundleParts, WorthQueryPortableCanonicalQueryBundleRecord,
};

use crate::binary_encoding::BinaryEncodingSink;
use crate::binary_input::BinaryInput;
use crate::denial::WorthQueryPackageArchiveDenial as Denial;
use crate::record::decode_budget::RecordDecodeAttempt;

pub(super) fn write_bundle(
    output: &mut dyn BinaryEncodingSink,
    bundle: &WorthQueryPortableCanonicalQueryBundleRecord,
) -> Result<(), Denial> {
    query::write_query(output, bundle.query())?;
    query::write_result_shape(output, bundle.result_shape())?;
    diagnostics::write_report(output, bundle.report())?;
    diagnostics::write_counters(output, bundle.counters())
}

pub(super) fn decode_bundle(
    input: &mut BinaryInput<'_>,
    budget: &mut RecordDecodeAttempt,
) -> Result<WorthQueryPortableCanonicalQueryBundleRecord, Denial> {
    let query = query::decode_query(input, budget)?;
    let result_shape = query::decode_result_shape(input, budget)?;
    let report = diagnostics::decode_report(input, budget)?;
    let counters = diagnostics::decode_counters(input)?;
    Ok(
        WorthQueryPortableCanonicalQueryBundleRecord::from_untrusted_parts(
            ARCHIVE_DECODE,
            WorthQueryPortableCanonicalQueryBundleParts {
                query_digest: query.digest,
                query_family: query.family,
                query_root: query.root,
                projection: query.projection,
                predicates: query.predicates,
                ordering: query.ordering,
                traversal: query.traversal,
                identity_bindings: query.identity_bindings,
                result_shape_digest: result_shape.digest,
                result_shape_family: result_shape.family,
                result_shape_fields: result_shape.fields,
                report,
                counters,
            },
        ),
    )
}

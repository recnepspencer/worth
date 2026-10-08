//! Fresh schema commitments sharing the package's complete canonical allowance.
use worth_foundational::facade::{
    CanonicalDigestDerivationDenial as Denial, CanonicalDigestId, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_schema::ErasedApplicationSchemaDeclaration;

use crate::application_schema::derive_installed_schema_identity_with_budget;
use crate::canonical_work::WorthQueryCanonicalWorkEvidence as Work;

pub(super) struct SchemaCommitments {
    digests: Vec<CanonicalDigestId>,
    work: Work,
    budget: CanonicalDigestWorkBudget,
}

pub(super) fn admit(
    schemas: &[ErasedApplicationSchemaDeclaration],
    budget: CanonicalDigestWorkBudget,
) -> Result<SchemaCommitments, Denial> {
    // The caller preflights all retained schema entries before package member
    // sorting. A child cannot obtain a fresh independent installation budget.
    let mut commitments = SchemaCommitments {
        digests: Vec::with_capacity(schemas.len()),
        work: Work::zero(),
        budget,
    };
    for schema in schemas {
        let remaining = commitments.remaining_budget()?;
        let (digest, work) =
            derive_installed_schema_identity_with_budget(schema.identity(), remaining)
                .map_err(|denial| commitments.aggregate_denial(denial))?;
        commitments.work = commitments.work.combine(work);
        commitments.digests.push(digest);
    }
    Ok(commitments)
}

impl SchemaCommitments {
    pub(super) fn digests(&self) -> &[CanonicalDigestId] {
        &self.digests
    }

    pub(super) fn work(&self) -> Work {
        self.work
    }

    pub(super) fn remaining_budget(&self) -> Result<CanonicalDigestWorkBudget, Denial> {
        let bytes = self
            .budget
            .maximum_encoded_bytes()
            .saturating_sub(self.work.canonical_encoded_bytes());
        if bytes == 0 {
            return Err(Denial::EncodedByteLimitExceeded {
                maximum: self.budget.maximum_encoded_bytes(),
                attempted: self.work.canonical_encoded_bytes().saturating_add(1),
            });
        }
        let entries = self
            .budget
            .maximum_entry_count()
            .saturating_sub(self.work.canonical_entries());
        CanonicalDigestWorkBudget::new(entries, bytes).ok_or(Denial::EntryLimitExceeded {
            maximum: self.budget.maximum_entry_count(),
            actual: self.work.canonical_entries().saturating_add(1),
        })
    }

    pub(super) fn aggregate_denial(&self, denial: Denial) -> Denial {
        match denial {
            Denial::EncodedByteLimitExceeded { attempted, .. } => {
                Denial::EncodedByteLimitExceeded {
                    maximum: self.budget.maximum_encoded_bytes(),
                    attempted: self
                        .work
                        .canonical_encoded_bytes()
                        .saturating_add(attempted),
                }
            }
            Denial::EntryLimitExceeded { actual, .. } => Denial::EntryLimitExceeded {
                maximum: self.budget.maximum_entry_count(),
                actual: self.work.canonical_entries().saturating_add(actual),
            },
            other => other,
        }
    }
}

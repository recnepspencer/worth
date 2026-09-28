//! Canonical program identity over the foundational canonical-basis and
//! digest APIs. The basis records meaning only: operator and type tags, exact
//! literals, slot bindings, unit catalog meaning, referenced declarations, and
//! the function closure. Spans, binder names, and formatting are excluded.

use std::sync::Arc;

use worth_proof::TransitionOutcome;

use super::basis::{literal_value, nominal_declarations, op_tag};
use crate::canonicalization::{
    admit_canonical_sequence_digest_derivation_with_budget, derive_canonical_digest,
    prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisSequence, CanonicalBasisValue,
    CanonicalDerivedDigest, CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial,
    CanonicalDigestWorkBudget, CanonicalIntegerWidth, CanonicalSingleSequenceDigestAlgorithmSlot,
    CanonicalizationRuleVersion,
};
use crate::expressions::denial::{ExpressionDenial, ExpressionResource, ExpressionResult};
use crate::expressions::functions::InstalledExpressionFunction;
use crate::expressions::profile::ExpressionProfile;
use crate::expressions::program::ExpressionProgram;
use crate::expressions::types::units::UNIT_CATALOG_VERSION;
use crate::expressions::types::{ExpressionSchema, ExpressionType};

/// The V1 language semantic version recorded in every identity.
pub(crate) const LANGUAGE_VERSION: &str = "worth-expression-v1";
const RULE_VERSION: &str = "worth-expression-identity-v1";
const DOMAIN: CanonicalBasisDomain = CanonicalBasisDomain::Future(LANGUAGE_VERSION);

/// Exact canonical meaning of an admitted program.
///
/// Equality compares the digest, the complete basis, and every closure
/// function's identity; a digest collision alone never makes two programs
/// equal.
#[derive(Debug, Clone)]
pub struct ExpressionProgramIdentity {
    basis: CanonicalBasisSequence,
    digest: CanonicalDerivedDigest,
    functions: Box<[Arc<ExpressionProgramIdentity>]>,
}

impl ExpressionProgramIdentity {
    pub fn basis(&self) -> &CanonicalBasisSequence {
        &self.basis
    }

    pub fn digest(&self) -> &CanonicalDerivedDigest {
        &self.digest
    }

    pub(crate) fn digest_bytes(&self) -> &[u8; 32] {
        self.digest.value().bytes()
    }

    /// Replaces the digest with another identity's, keeping this basis, so
    /// tests can prove that equal digests over different bases stay unequal.
    #[cfg(test)]
    pub(crate) fn with_digest_of(mut self, other: &Self) -> Self {
        self.digest = other.digest.clone();
        self
    }
}

impl PartialEq for ExpressionProgramIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.digest == other.digest
            && self.basis.version() == other.basis.version()
            && self.basis.domain() == other.basis.domain()
            && self.basis.entries() == other.basis.entries()
            && self.functions == other.functions
    }
}

impl Eq for ExpressionProgramIdentity {}

/// The facts identity covers beyond the program nodes.
pub(crate) struct IdentityInputs<'a> {
    pub(crate) program: &'a ExpressionProgram,
    /// Slot bindings in slot order. Operand names are semantic; function
    /// parameter names are binders and pass `None`.
    pub(crate) slots: Vec<(Option<&'a str>, &'a ExpressionType)>,
    pub(crate) closure: &'a [Arc<InstalledExpressionFunction>],
    pub(crate) schema: &'a ExpressionSchema,
    pub(crate) profile: &'a ExpressionProfile,
}

pub(crate) fn derive_identity(
    inputs: IdentityInputs<'_>,
) -> ExpressionResult<ExpressionProgramIdentity> {
    let entries = basis_entries(&inputs);
    let limit = inputs.profile.limit(ExpressionResource::CanonicalBytes);
    let exceeded = || ExpressionDenial::resource(ExpressionResource::CanonicalBytes, limit);
    let entry_count = u32::try_from(entries.len()).map_err(|_| exceeded())?;
    let budget =
        CanonicalDigestWorkBudget::new(entry_count, usize::try_from(limit).unwrap_or(usize::MAX))
            .ok_or_else(exceeded)?;
    let version = CanonicalizationRuleVersion::new(RULE_VERSION).expect("rule version is nonempty");
    let ready = match prepare_canonical_basis_sequence(version.clone(), DOMAIN, entries) {
        TransitionOutcome::Success(ready) => ready,
        TransitionOutcome::Denied(denial) => {
            unreachable!("expression bases are nonempty with unique loci: {denial:?}")
        }
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::Stale(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => unreachable!("basis preparation uses only denied"),
    };
    let basis = ready.payload().clone();
    let slot = CanonicalSingleSequenceDigestAlgorithmSlot::single_sequence(
        CanonicalDigestAlgorithmId::sha256(),
        DOMAIN,
        version,
    );
    let derivation =
        match admit_canonical_sequence_digest_derivation_with_budget(ready, slot, budget) {
            TransitionOutcome::Success(derivation) => derivation,
            TransitionOutcome::Denied(
                CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. }
                | CanonicalDigestDerivationDenial::EntryLimitExceeded { .. },
            ) => return Err(exceeded()),
            TransitionOutcome::Denied(denial) => {
                unreachable!("the slot matches the basis domain and version: {denial:?}")
            }
            TransitionOutcome::Deferred(_)
            | TransitionOutcome::Stale(_)
            | TransitionOutcome::RebindRequired(_)
            | TransitionOutcome::Failed(_) => unreachable!("digest admission uses only denied"),
        };
    Ok(ExpressionProgramIdentity {
        basis,
        digest: derive_canonical_digest(derivation),
        functions: inputs
            .closure
            .iter()
            .map(|function| function.identity().clone())
            .collect(),
    })
}

fn basis_entries(inputs: &IdentityInputs<'_>) -> Vec<CanonicalBasisEntry> {
    let text = |value: String| CanonicalBasisValue::ExactText(value.into());
    let mut entries = vec![
        named("language", "version", text(LANGUAGE_VERSION.to_string())),
        named(
            "result",
            "type",
            text(inputs.program.result_type().to_string()),
        ),
        named("units", "catalog", text(UNIT_CATALOG_VERSION.to_string())),
    ];
    for (index, (name, ty)) in inputs.slots.iter().enumerate() {
        let locus = format!("slot:{index}");
        entries.push(named(&locus, "type", text(ty.to_string())));
        if let Some(name) = name {
            entries.push(named(&locus, "name", text((*name).to_string())));
        }
    }
    let types = inputs
        .program
        .nodes()
        .iter()
        .map(|node| &node.ty)
        .chain(inputs.slots.iter().map(|(_, ty)| *ty));
    for (name, declaration) in nominal_declarations(inputs.schema, types) {
        entries.push(named(
            &format!("nominal:{name}"),
            "declaration",
            text(declaration),
        ));
    }
    for (index, function) in inputs.closure.iter().enumerate() {
        let locus = format!("function:{index}");
        entries.push(named(
            &locus,
            "name",
            text(format!("{}@{}", function.name(), function.version())),
        ));
        entries.push(named(
            &locus,
            "digest",
            text(hex(function.identity().digest_bytes())),
        ));
    }
    for (ordinal, node) in inputs.program.nodes().iter().enumerate() {
        let ordinal = ordinal as u32;
        entries.push(node_entry(ordinal, "op", text(op_tag(&node.op))));
        entries.push(node_entry(ordinal, "type", text(node.ty.to_string())));
        let arity = CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits32,
            value: node.children.len() as u128,
        };
        entries.push(node_entry(ordinal, "arity", arity));
        if let Some(value) = literal_value(&node.op) {
            entries.push(node_entry(ordinal, "literal", value));
        }
    }
    entries
}

fn named(locus: &str, kind: &'static str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    let locus = CanonicalBasisLocus::Named(locus.to_string().into());
    CanonicalBasisEntry::new(DOMAIN, locus, CanonicalBasisEntryKind::Future(kind), value)
}

fn node_entry(ordinal: u32, kind: &'static str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    let locus = CanonicalBasisLocus::EntryOrdinal(ordinal);
    CanonicalBasisEntry::new(DOMAIN, locus, CanonicalBasisEntryKind::Future(kind), value)
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use crate::expression_api::{
        expressions, ExpressionFunctionCatalog, ExpressionProfile, ExpressionSchema, ExpressionType,
    };

    fn identities(sources: [&str; 2]) -> [super::ExpressionProgramIdentity; 2] {
        let schema = ExpressionSchema::builder()
            .operand("x", ExpressionType::Float64)
            .and_then(|builder| builder.operand("y", ExpressionType::Float64))
            .expect("operands are valid")
            .build();
        let catalog =
            ExpressionFunctionCatalog::builder(&schema, ExpressionProfile::interactive()).build();
        sources.map(|source| {
            expressions()
                .parse(source)
                .and_then(|draft| draft.admit(&schema, &catalog, ExpressionProfile::interactive()))
                .expect("fixture admits")
                .identity()
                .clone()
        })
    }

    #[test]
    fn equal_digests_over_different_bases_are_unequal() {
        let [x, y] = identities(["x + 1.0", "y + 1.0"]);
        let forged = y.clone().with_digest_of(&x);
        assert_eq!(forged.digest(), x.digest());
        assert_ne!(
            forged, x,
            "a digest collision never merges distinct meaning"
        );
        assert_ne!(forged, y);
    }

    #[test]
    fn equal_meaning_is_equal_identity() {
        let [first, second] = identities(["x+1.0", "x + 1.00"]);
        assert_eq!(first, second);
    }
}

use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, canonicalization, compare_canonical_basis,
    prepare_canonical_basis_sequence, prepare_canonical_comparison, AspectValue,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisReadyArtifact, CanonicalBasisValue, CanonicalComparisonOutcome,
    CanonicalDigestAlgorithmId, CanonicalDigestDerivationDenial, CanonicalDigestId,
    CanonicalDigestWorkBudget, CanonicalEquivalenceBasis, CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

const PARAMETER_DOMAIN: &str = "worth-query.application-query-parameters";
const PARAMETER_RULE: &str = "worth-query-application-query-parameters-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationParameterCanonicalArtifact {
    basis: CanonicalBasisReadyArtifact,
    digest: CanonicalDigestId,
    work: WorthQueryCanonicalWorkEvidence,
}

impl WorthQueryApplicationParameterCanonicalArtifact {
    pub fn basis(&self) -> &CanonicalBasisReadyArtifact {
        &self.basis
    }

    pub const fn identity(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }

    pub const fn work(&self) -> WorthQueryCanonicalWorkEvidence {
        self.work
    }

    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        let comparison = prepare_canonical_comparison(
            CanonicalEquivalenceBasis::ExactCanonicalBasis,
            self.basis.clone(),
            other.basis.clone(),
        )
        .into_result()
        .expect("application-query parameter artifacts share one comparison family");
        matches!(
            compare_canonical_basis(&comparison),
            CanonicalComparisonOutcome::Equivalent(_)
        )
    }
}

pub(super) fn prepare_parameter_basis(
    bindings: &[(&'static str, AspectValue)],
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryApplicationParameterCanonicalArtifact, CanonicalDigestDerivationDenial> {
    let domain = CanonicalBasisDomain::Future(PARAMETER_DOMAIN);
    let entries = prepared_entries(bindings, &mut |_, _| Ok::<_, std::convert::Infallible>(()))
        .expect("an ordinary parameter basis has no resource refusal");
    let version = CanonicalizationRuleVersion::new(PARAMETER_RULE)
        .expect("the fixed application-query parameter rule is valid");
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .expect("the application-query parameter basis always has its count entry");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis.clone(), CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()?;
    let derived = canonicalization().digest().derive(ready);
    Ok(WorthQueryApplicationParameterCanonicalArtifact {
        basis,
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn entry(
    domain: CanonicalBasisDomain,
    locus: String,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Field,
        value,
    )
}

mod preparation;
pub(super) use preparation::{prepare_parameter_basis_admitted, ParameterBasisPreparationStop};

fn prepared_entries<Stop>(
    bindings: &[(&'static str, AspectValue)],
    admit: &mut impl FnMut(usize, usize) -> Result<(), Stop>,
) -> Result<Vec<CanonicalBasisEntry>, ParameterBasisPreparationStop<Stop>> {
    use ParameterBasisPreparationStop::{AccountingOverflow, Admission};
    admit(1, 0).map_err(Admission)?;
    let count = bindings
        .len()
        .checked_mul(3)
        .and_then(|n| n.checked_add(1))
        .ok_or(AccountingOverflow)?;
    let bytes = count
        .checked_mul(std::mem::size_of::<CanonicalBasisEntry>())
        .and_then(|n| n.checked_add("binding-count".len()))
        .ok_or(AccountingOverflow)?;
    admit(count, bytes).map_err(Admission)?;
    let domain = CanonicalBasisDomain::Future(PARAMETER_DOMAIN);
    let mut entries = Vec::with_capacity(count);
    entries.push(entry(
        domain,
        "binding-count".to_owned(),
        CanonicalBasisValue::UnsignedInteger {
            width: worth_foundational::facade::CanonicalIntegerWidth::Bits64,
            value: bindings.len() as u128,
        },
    ));
    for (index, (name, value)) in bindings.iter().enumerate() {
        admit(3, 0).map_err(Admission)?;
        let digits = if index == 0 {
            1
        } else {
            index.ilog10() as usize + 1
        };
        let prefix = "parameter[]"
            .len()
            .checked_add(digits)
            .ok_or(AccountingOverflow)?;
        let loci = prefix
            .checked_mul(3)
            .and_then(|n| n.checked_add(".name.scalar.value".len()))
            .ok_or(AccountingOverflow)?;
        let scalar = value.value_family().canonical_name();
        let bytes = loci
            .checked_add(name.len())
            .and_then(|n| n.checked_add(scalar.len()))
            .and_then(|n| n.checked_add(value.owned_allocation_capacity_bytes()))
            .ok_or(AccountingOverflow)?;
        let work = loci
            .checked_add(name.len())
            .and_then(|n| n.checked_add(scalar.len()))
            .and_then(|n| n.checked_add(value.semantic_byte_width()))
            .ok_or(AccountingOverflow)?;
        admit(work, bytes).map_err(Admission)?;
        entries.extend([
            entry(
                domain,
                parameter_locus(index, digits, ".name"),
                CanonicalBasisValue::ExactText((*name).into()),
            ),
            entry(
                domain,
                parameter_locus(index, digits, ".scalar"),
                CanonicalBasisValue::ExactText(scalar.into()),
            ),
            entry(
                domain,
                parameter_locus(index, digits, ".value"),
                canonical_basis_value_for_aspect_value(value),
            ),
        ]);
    }
    Ok(entries)
}

fn parameter_locus(index: usize, digits: usize, suffix: &str) -> String {
    use std::fmt::Write;
    let mut value = String::with_capacity("parameter[]".len() + digits + suffix.len());
    write!(&mut value, "parameter[{index}]{suffix}")
        .expect("writing a parameter locus into its sized String");
    value
}

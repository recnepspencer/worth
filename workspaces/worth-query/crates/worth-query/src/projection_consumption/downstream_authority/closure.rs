#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DownstreamAuthorityClosureRole {
    Authoritative,
    OrderedAuthority,
    DerivedEvidence,
    DeletionObligation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DownstreamAuthorityClosureRow {
    component: &'static str,
    owner: &'static str,
    role: DownstreamAuthorityClosureRole,
}

impl DownstreamAuthorityClosureRow {
    pub fn component(&self) -> &'static str {
        self.component
    }

    pub fn owner(&self) -> &'static str {
        self.owner
    }

    pub fn role(&self) -> DownstreamAuthorityClosureRole {
        self.role
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DownstreamAuthorityClosureContract {
    rows: &'static [DownstreamAuthorityClosureRow],
}

impl DownstreamAuthorityClosureContract {
    pub fn rows(&self) -> &'static [DownstreamAuthorityClosureRow] {
        self.rows
    }

    pub fn authoritative_width(&self) -> usize {
        self.rows
            .iter()
            .filter(|row| {
                matches!(
                    row.role,
                    DownstreamAuthorityClosureRole::Authoritative
                        | DownstreamAuthorityClosureRole::OrderedAuthority
                )
            })
            .count()
    }

    pub fn deletion_obligations(&self) -> impl Iterator<Item = &'static str> + use<> {
        self.rows.iter().filter_map(|row| {
            (row.role == DownstreamAuthorityClosureRole::DeletionObligation)
                .then_some(row.component)
        })
    }
}

const ROWS: &[DownstreamAuthorityClosureRow] = &[
    row(
        "scoped_basis",
        "worth-query basis lifecycle",
        DownstreamAuthorityClosureRole::Authoritative,
    ),
    row(
        "projection_contract",
        "worth-query projection consumption",
        DownstreamAuthorityClosureRole::Authoritative,
    ),
    row(
        "consumption_receipt",
        "worth-query projection consumption",
        DownstreamAuthorityClosureRole::Authoritative,
    ),
    row(
        "source_identity",
        "worth-query source adapter",
        DownstreamAuthorityClosureRole::Authoritative,
    ),
    row(
        "source_reference_order",
        "worth-query source adapter",
        DownstreamAuthorityClosureRole::OrderedAuthority,
    ),
    row(
        "settlement_posture",
        "worth-query projection consumption",
        DownstreamAuthorityClosureRole::Authoritative,
    ),
    row(
        "consumed_fact_order",
        "worth-query projection consumption",
        DownstreamAuthorityClosureRole::OrderedAuthority,
    ),
    row(
        "evidence_identity",
        "worth-query inspection",
        DownstreamAuthorityClosureRole::DerivedEvidence,
    ),
    row(
        "independently_pairable_completed_parts",
        "worth-query legacy DX",
        DownstreamAuthorityClosureRole::DeletionObligation,
    ),
    row(
        "consumer_basis_compatibility_scan",
        "downstream consumer residue",
        DownstreamAuthorityClosureRole::DeletionObligation,
    ),
    row(
        "digest_to_authority_promotion",
        "downstream consumer residue",
        DownstreamAuthorityClosureRole::DeletionObligation,
    ),
    row(
        "raw_source_identity_reentry",
        "downstream consumer residue",
        DownstreamAuthorityClosureRole::DeletionObligation,
    ),
];

const fn row(
    component: &'static str,
    owner: &'static str,
    role: DownstreamAuthorityClosureRole,
) -> DownstreamAuthorityClosureRow {
    DownstreamAuthorityClosureRow {
        component,
        owner,
        role,
    }
}

pub fn downstream_authority_closure_contract() -> DownstreamAuthorityClosureContract {
    DownstreamAuthorityClosureContract { rows: ROWS }
}

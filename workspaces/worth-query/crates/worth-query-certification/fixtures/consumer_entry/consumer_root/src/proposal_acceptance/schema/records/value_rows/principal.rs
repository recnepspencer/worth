//! The proposal principal's field declaration.
use super::*;
pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<ProposalRecords, Principal, Self> {
        aspect(ProposalRecords, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity: u64,
}
impl fragment::Record<PrincipalFactsValue> for ProposalRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentity", |row: &PrincipalFactsValue| {
            row.principal_identity
        })
        .equality_queryable(true)
        .read_only()
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalFactsValue { principal_identity })
        })
    }
}

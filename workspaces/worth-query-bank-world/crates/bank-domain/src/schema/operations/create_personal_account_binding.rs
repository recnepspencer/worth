use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalizationRuleVersion,
};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputRoleDescriptor,
        WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, AccountName, BankPrincipalId, InstitutionId};
use crate::proposals::{BankIdempotencyKey, BankProposalDenial};

use super::{
    CreatePersonalAccount, CreatePersonalAccountInputBinding, CreatePersonalAccountOperation,
};
use crate::schema::{
    Account, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
    Institution, InstitutionIdentity, InstitutionIdentityField, Principal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreatePersonalAccountResult {
    pub account: AccountId,
}

worth_query_structured_value_binding!(
    pub CreatePersonalAccountResultBinding for CreatePersonalAccountResult {
        identity: "bank.operation.create-personal-account.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub CreatePersonalAccountDenialBinding for BankProposalDenial {
        identity: "bank.operation.create-personal-account.denial.v1"
    }
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePersonalAccountDecision {
    account: AccountId,
    institution: InstitutionId,
    owner: BankPrincipalId,
    display_name: AccountName,
}

pub struct CreatePersonalAccountOutputs;

/// The personal account the operation creates.
pub struct CreatedPersonalAccountOutput;

impl WorthQueryApplicationOutputRole for CreatedPersonalAccountOutput {
    type Schema = BankSchema;
    type Contract = CreatePersonalAccountOutputs;
    type Entity = Account;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "account";
}

impl ApplicationMutationOutputContract<BankSchema> for CreatePersonalAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<CreatedPersonalAccountOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}

impl CreatePersonalAccountDecision {
    /// Derives the new account's identity from Query's canonical identities of
    /// the admitted client key and input, so a retry of the same request names
    /// the same account.
    pub fn from_admitted_identities(
        key_identity: &[u8; 32],
        input_identity: &[u8; 32],
        input: &CreatePersonalAccount,
    ) -> Self {
        let operation_identity = derive_identity(
            CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
            CREATE_PERSONAL_ACCOUNT_IDENTITY_RULE_VERSION,
            [
                digest_entry(
                    CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
                    "client-key",
                    *key_identity,
                ),
                digest_entry(
                    CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
                    "input",
                    *input_identity,
                ),
            ],
        );
        Self {
            account: AccountId::from_operation(operation_identity, 0),
            institution: input.institution,
            owner: input.owner,
            display_name: input.display_name.clone(),
        }
    }

    pub fn into_parts(self) -> (AccountId, InstitutionId, BankPrincipalId, AccountName) {
        (
            self.account,
            self.institution,
            self.owner,
            self.display_name,
        )
    }
}

const CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.create-personal-account-identity");
const CREATE_PERSONAL_ACCOUNT_IDENTITY_RULE_VERSION: &str =
    "worth-bank-create-personal-account-identity-v1";

fn institution_scope(input: &CreatePersonalAccount) -> InstitutionId {
    input.institution
}

worth_query_mutation_binding!(
    pub CreatePersonalAccountMutationBinding for CreatePersonalAccount, schema BankSchema,
    identity "bank.operation.create-personal-account.mutation-binding.v1",
    input CreatePersonalAccountInputBinding,
    operation CreatePersonalAccountOperation,
    result CreatePersonalAccountResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
    decision CreatePersonalAccountDecision,
    denial CreatePersonalAccountDenialBinding,
    handler identity "bank.operation.create-personal-account.handler.v1",
    program required,
    outputs CreatePersonalAccountOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Institution,
        InstitutionIdentity,
        InstitutionIdentityField,
        InstitutionId,
        ReadOnly,
        NoApplicationUnit,
    field InstitutionIdentityField::reference(),
    value institution_scope
);

fn derive_identity<const N: usize>(
    domain: CanonicalBasisDomain,
    rule_version: &'static str,
    entries: [CanonicalBasisEntry; N],
) -> [u8; 32] {
    let version = CanonicalizationRuleVersion::new(rule_version)
        .expect("the fixed bank application mutation identity rule is valid");
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .expect("bank application mutation identity fields have unique typed loci");
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .expect("SHA-256 admits the typed bank application mutation identity basis");
    *canonicalization().digest().derive(ready).value().bytes()
}

fn digest_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: [u8; 32],
) -> CanonicalBasisEntry {
    identity_entry(
        domain,
        locus,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(value)),
    )
}

fn identity_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}

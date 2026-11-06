use super::*;

pub type Binary = FieldSelection<NumericModel, Number, WriteFacts, BinaryCounter>;
pub type Canonical = FieldSelection<NumericModel, Number, WriteFacts, CanonicalCounter>;
pub type Delegated = FieldSelection<NumericModel, Number, WriteFacts, DelegatedCounter>;
pub type ExternalIdentityField = FieldSelection<
    NumericModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryExternalPrincipalIdentity,
>;
pub type MappingStatusField = FieldSelection<
    NumericModel,
    ExternalMapping,
    ExternalIdentity,
    WorthQueryPrincipalMappingStatus,
>;
pub type PrincipalIdentityField = FieldSelection<NumericModel, Principal, PrincipalFacts, u64>;
pub type Key = FieldSelection<NumericModel, Number, NumberFacts, String>;
pub type Primary = FieldSelection<NumericModel, Number, NumberFacts, Counter>;
pub type Secondary = FieldSelection<NumericModel, Number, NumberFacts, u64>;
pub type ReadEnabled = FieldSelection<NumericModel, Number, NumberFacts, bool>;
pub type Ceiling = FieldSelection<NumericModel, Number, NumberFacts, u64>;
pub type Calculated = FieldSelection<NumericModel, Number, NumberFacts, u64>;
pub type Operand = FieldSelection<NumericModel, Number, NumberFacts, u64>;
pub type Value = FieldSelection<NumericModel, Number, ScalarFacts, u64>;
pub type Total = FieldSelection<NumericModel, Number, ScalarResult, u64>;
pub type DefinitionRevision = FieldSelection<NumericModel, Number, DefinitionRevisionFacts, u64>;
pub type Disclosure = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type Bias = FieldSelection<NumericModel, Number, InputFacts, u64>;
pub type Sibling = FieldSelection<NumericModel, Number, InputFacts, u64>;
pub type GrantIdentity = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type GrantAction = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type GrantPurpose = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type GrantStatus = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type GrantScopeKey = FieldSelection<NumericModel, Grant, GrantFacts, String>;
pub type GrantNotBefore = FieldSelection<NumericModel, Grant, GrantFacts, u64>;
pub type GrantNotAfter = FieldSelection<NumericModel, Grant, GrantFacts, u64>;
pub type GrantDepth = FieldSelection<NumericModel, Grant, GrantFacts, u64>;

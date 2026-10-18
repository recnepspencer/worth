//! Stable companion meaning, unchanged across the codec cutover.
use super::super::schema::CanonicalSchema;
use worth_query_host::facade::{declaration::application_schema::*, *};
worth_query_entity!(pub Companion for CanonicalSchema);
worth_query_aspect!(pub CompanionFacts for CanonicalSchema, Companion; identity = AspectIdentity(0x91768205), revision = AspectContractRevision(1));
worth_query_field!(pub CompanionIdentityField for CanonicalSchema, Companion, CompanionFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub CompanionValueField for CanonicalSchema, Companion, CompanionFacts: String => StringApplicationValueBinding, read_write, equality);
worth_query_relation!(pub CompanionLink in CanonicalSchema, Companion => Companion; integrity = same_context_unbounded_retain_dangling);

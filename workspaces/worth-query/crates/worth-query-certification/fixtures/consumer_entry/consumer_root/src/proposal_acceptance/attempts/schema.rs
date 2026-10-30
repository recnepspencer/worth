//! The model links issued claim and result occurrences on its stable storage product.
use super::{super::schema::*, data::*};
use worth_query_decl::facade::{application_schema::*, *};
worth_query_entity!(pub AcceptanceClaim for Schema);
worth_query_entity!(pub AcceptanceResult for Schema);
worth_query_aspect!(pub ClaimFacts for Schema,AcceptanceClaim;identity=AspectIdentity(0x91811709),revision=AspectContractRevision(1));
worth_query_aspect!(pub ResultFacts for Schema,AcceptanceResult;identity=AspectIdentity(0x9181170a),revision=AspectContractRevision(1));
worth_query_field!(pub ClaimName for Schema,AcceptanceClaim,ClaimFacts:String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub ClaimData for Schema,AcceptanceClaim,ClaimFacts:StoredClaim=>ClaimDataBinding,read_only,equality);
worth_query_field!(pub ResultName for Schema,AcceptanceResult,ResultFacts:String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub ResultData for Schema,AcceptanceResult,ResultFacts:StoredResult=>ResultDataBinding,read_only,equality);
worth_query_relation!(pub ModelClaim in Schema,Model=>AcceptanceClaim;integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub ModelResult in Schema,Model=>AcceptanceResult;integrity=same_context_unbounded_retain_dangling);
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
        .entity(AcceptanceClaim::reference())
        .entity(AcceptanceResult::reference())
        .aspect(AcceptanceClaim::reference(), ClaimFacts::reference())
        .aspect(AcceptanceResult::reference(), ResultFacts::reference())
        .field(AcceptanceClaim::reference(), ClaimName::reference())
        .field(AcceptanceClaim::reference(), ClaimData::reference())
        .field(AcceptanceResult::reference(), ResultName::reference())
        .field(AcceptanceResult::reference(), ResultData::reference())
        .relation(
            ModelClaim::reference(),
            Model::reference(),
            AcceptanceClaim::reference(),
        )
        .relation(
            ModelResult::reference(),
            Model::reference(),
            AcceptanceResult::reference(),
        )
}

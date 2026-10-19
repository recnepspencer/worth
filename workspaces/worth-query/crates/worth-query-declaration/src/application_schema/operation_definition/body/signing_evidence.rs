//! A requirement reference derived from one signed producer, never an approval grant.
use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial as Denial,
    ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyProgram as Body,
    ApplicationOperationBodySigningLifecycleSpec,
};
use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationScalarValueBinding, ApplicationSchema,
    ApplicationSchemaMember as Member, ApplicationSigningSubjectProvenanceBinding,
};
use std::marker::PhantomData;

/// Portable selector meaning. Fresh schema validation must resolve its exact producer.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationSigningEvidenceSource {
    pub mutation_binding: String,
    pub operation: String,
    pub evidence_entity: String,
    pub provenance: ApplicationFieldBindingLocus,
    pub purpose: String,
    pub producer_meaning: String,
    pub lifecycle: ApplicationOperationBodySigningLifecycleSpec,
}

/// Typed descriptive reference to a completed body; possession grants no authority.
pub struct ApplicationSigningEvidenceRef<Schema, Binding> {
    pub(super) source: ApplicationSigningEvidenceSource,
    marker: PhantomData<fn() -> (Schema, Binding)>,
}

impl<
        Schema: ApplicationSchema,
        Operation: crate::application_schema::ApplicationOperationMarkerIdentity<Schema>,
        Input,
    > ApplicationOperationBody<Schema, Operation, Input>
{
    pub fn signed_evidence<Binding>(
        &self,
    ) -> Result<ApplicationSigningEvidenceRef<Schema, Binding>, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        let source = ApplicationSigningEvidenceSource::from_body(
            Binding::IDENTITY,
            Binding::Operation::IDENTIFIER,
            &self.program,
        )
        .ok_or(Denial::InvalidSigningLifecycle)?;
        Ok(ApplicationSigningEvidenceRef {
            source,
            marker: PhantomData,
        })
    }
}

impl ApplicationSigningEvidenceSource {
    fn from_body(binding: &str, operation: &str, body: &Body) -> Option<Self> {
        let creation = body.creation()?;
        let signing = creation.signing()?;
        let lifecycle = creation.signing_lifecycle()?;
        if body.scope_entity() != Some(lifecycle.source_entity.as_str())
            || !creation.fields().iter().any(|field| matches!(field, Field::SigningProvenance { field, binding }
                if field == signing.provenance() && *binding == ApplicationSigningSubjectProvenanceBinding::IDENTITY))
            || !creation.fields().iter().any(|field| matches!(field, Field::PrincipalIdentity { field, mutation_binding, .. }
                if field == signing.actor() && mutation_binding == binding)) { return None; }
        // Use the schema owner's canonical body encoding, never a second body serializer.
        let producer_basis = crate::application_schema::canonical_identity::canonical_identity(
            crate::application_schema::canonical_identity::ApplicationSchemaCanonicalHeader {
                owner: "worth.query",
                name: "signed-evidence-producer",
                major: 1,
                minor: 0,
            },
            &[Member::OperationBody {
                operation: operation.into(),
                body: body.clone(),
            }],
            &[],
        );
        use worth_foundational::facade::{
            canonicalization, CanonicalDigestAlgorithmId, CanonicalDigestWorkBudget,
        };
        let ready = canonicalization()
            .digest()
            .for_sequence_with_budget(
                producer_basis.canonical_basis().clone(),
                CanonicalDigestAlgorithmId::sha256(),
                CanonicalDigestWorkBudget::new(u32::MAX, usize::MAX)?,
            )
            .into_result()
            .ok()?;
        let digest = canonicalization().digest().derive(ready);
        let mut producer_meaning = String::with_capacity(64);
        for byte in digest.value().bytes() {
            use std::fmt::Write;
            write!(&mut producer_meaning, "{byte:02x}").expect("String write");
        }
        Some(Self {
            mutation_binding: binding.into(),
            operation: operation.into(),
            evidence_entity: creation.entity().into(),
            provenance: signing.provenance().clone(),
            purpose: signing.purpose().into(),
            producer_meaning,
            lifecycle: lifecycle.clone(),
        })
    }
    pub fn is_declared(&self, members: &[Member]) -> bool {
        members.iter().any(|member| matches!(member, Member::ApplicationMutation { description }
            if description.binding_identity().as_str() == self.mutation_binding && description.operation() == self.operation
                && description.scope().entity == self.lifecycle.source_entity))
        && members.iter().any(|member| matches!(member, Member::OperationBody { operation, body }
            if operation == &self.operation && Self::from_body(&self.mutation_binding, operation, body).as_ref() == Some(self)))
    }
    pub fn decision_reads(
        &self,
    ) -> Vec<crate::application_schema::ApplicationOperationDecisionReadTarget> {
        use crate::application_schema::ApplicationOperationDecisionReadTarget as Read;
        vec![
            Read::Relation {
                relation: self.lifecycle.current_relation.clone(),
                from: self.lifecycle.source_entity.clone(),
                to: self.evidence_entity.clone(),
            },
            Read::Entity {
                entity: self.lifecycle.source_entity.clone(),
            },
            Read::Entity {
                entity: self.evidence_entity.clone(),
            },
            Read::Field {
                entity: self.provenance.entity().into(),
                aspect: self.provenance.aspect().into(),
                field: self.provenance.field().into(),
            },
        ]
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut output = String::from("current-signed-evidence-v1;");
        for value in [
            &self.mutation_binding,
            &self.operation,
            &self.evidence_entity,
            self.provenance.entity(),
            self.provenance.aspect(),
            self.provenance.field(),
            &self.purpose,
            &self.producer_meaning,
            &self.lifecycle.canonical_record(),
        ] {
            write!(&mut output, "{}:{value};", value.len()).expect("String write");
        }
        output
    }
    /// Retained owned strings in a finding; quoting must not allocate before admission.
    pub fn retained_string_bytes(&self) -> Option<usize> {
        [
            &self.mutation_binding,
            &self.operation,
            &self.evidence_entity,
            self.provenance.entity(),
            self.provenance.aspect(),
            self.provenance.field(),
            &self.purpose,
            &self.producer_meaning,
            &self.lifecycle.history_relation,
            &self.lifecycle.current_relation,
            &self.lifecycle.source_entity,
            &self.lifecycle.consequence_relation,
            self.lifecycle.consequence_field.entity(),
            self.lifecycle.consequence_field.aspect(),
            self.lifecycle.consequence_field.field(),
        ]
        .into_iter()
        .try_fold(0usize, |total, value| total.checked_add(value.len()))
    }
}

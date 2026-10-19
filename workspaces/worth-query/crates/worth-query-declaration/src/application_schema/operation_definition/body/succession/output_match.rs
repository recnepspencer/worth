//! Binding-affine identity equality, not a caller-selected predecessor target.
use super::{
    ApplicationOperationBodyBuilder, ApplicationOperationBodyProgram, ApplicationSchema, Denial,
};
use crate::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationOutputPosture as Posture,
        ApplicationMutationOutputRoleCardinality as Cardinality,
        ApplicationMutationOutputRoleDescription, WorthQueryApplicationDeclaredOutputRole,
        WorthQueryApplicationOutputRole, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
    },
    application_schema::{ApplicationOperationBodyOutputTarget, ApplicationSchemaMember},
};

/// Pure selector: latest performed occurrence of the named producer binding.
/// It does not assert that this producer is the workflow's final operation.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodySuccessionOutputMatch {
    selected_role: String,
    producer_binding: String,
    producer_role: ApplicationMutationOutputRoleDescription,
}
impl ApplicationOperationBodySuccessionOutputMatch {
    pub fn from_untrusted(
        selected_role: String,
        producer_binding: String,
        producer_role: ApplicationMutationOutputRoleDescription,
    ) -> Self {
        Self {
            selected_role,
            producer_binding,
            producer_role,
        }
    }
    pub fn selected_role(&self) -> &str {
        &self.selected_role
    }
    pub fn producer_binding(&self) -> &str {
        &self.producer_binding
    }
    pub fn producer_role(&self) -> &ApplicationMutationOutputRoleDescription {
        &self.producer_role
    }
    pub fn canonical_record(&self) -> String {
        use std::fmt::Write;
        let mut record = String::from("succession-output-match-v1;");
        for part in [
            &self.selected_role,
            &self.producer_binding,
            &self.producer_role.name,
            &self.producer_role.entity,
        ] {
            write!(&mut record, "{}:{part};", part.len()).expect("String write");
        }
        let posture = match self.producer_role.posture {
            Posture::Preserve => 1,
            Posture::Create => 2,
            Posture::Retire => 3,
        };
        let cardinality = match self.producer_role.cardinality {
            Cardinality::ExactlyOne => 1,
            Cardinality::AtMostOne => 2,
        };
        write!(&mut record, "{posture};{cardinality};").expect("String write");
        record
    }
}
impl<Schema: ApplicationSchema, Operation, Input>
    ApplicationOperationBodyBuilder<Schema, Operation, Input>
{
    /// Compare an authored Preserve output with the latest performed producer output.
    /// Both roles must have the same entity and exact-one cardinality. Native history,
    /// fresh capability admission and commit currentness remain mandatory.
    ///
    /// A created replacement cannot masquerade as the selected preimage:
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn created<S, B, P, R, O>(body: ApplicationOperationBodyBuilder<S,B::Operation,B::Input>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>, P: ApplicationMutationBinding<S>,
    /// R: WorthQueryApplicationOutputRole<Schema=S, Contract=B::Output, Action=WorthQueryCreateOutput, Cardinality=WorthQueryExactlyOneOutput>,
    /// O: WorthQueryApplicationOutputRole<Schema=S, Contract=P::Output, Entity=R::Entity, Cardinality=WorthQueryExactlyOneOutput> {
    ///     let _ = body.require_predecessor_output::<B,R,P,O>();
    /// }
    /// ```
    ///
    /// Unrelated entity markers cannot be joined even if names happen to match:
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn unrelated<S, B, P, R, O>(body: ApplicationOperationBodyBuilder<S,B::Operation,B::Input>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>, P: ApplicationMutationBinding<S>,
    /// R: WorthQueryApplicationOutputRole<Schema=S, Contract=B::Output, Action=WorthQueryPreserveOutput, Cardinality=WorthQueryExactlyOneOutput>,
    /// O: WorthQueryApplicationOutputRole<Schema=S, Contract=P::Output, Cardinality=WorthQueryExactlyOneOutput> {
    ///     let _ = body.require_predecessor_output::<B,R,P,O>();
    /// }
    /// ```
    pub fn require_predecessor_output<Binding, Selected, Producer, Produced>(
        mut self,
    ) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Producer: ApplicationMutationBinding<Schema>,
        Selected: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Produced: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Producer::Output,
            Entity = Selected::Entity,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
    {
        const { <Selected as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        const { <Produced as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
        let succession = self
            .succession
            .as_mut()
            .ok_or(Denial::InvalidWorkflowSuccession)?;
        if succession.binding() != Binding::IDENTITY || succession.output_match().is_some() {
            return Err(Denial::InvalidWorkflowSuccession);
        }
        let role = <Produced as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
        succession.output_match = Some(
            ApplicationOperationBodySuccessionOutputMatch::from_untrusted(
                Selected::NAME.into(),
                Producer::IDENTITY.into(),
                ApplicationMutationOutputRoleDescription {
                    name: role.name().into(),
                    entity: role.entity().into(),
                    posture: role.posture(),
                    cardinality: role.cardinality(),
                },
            ),
        );
        Ok(self)
    }
}

pub(super) fn valid(body: &ApplicationOperationBodyProgram) -> bool {
    let Some(succession) = body.succession() else {
        return true;
    };
    let Some(matching) = succession.output_match() else {
        return true;
    };
    if [
        &matching.selected_role,
        &matching.producer_binding,
        &matching.producer_role.name,
        &matching.producer_role.entity,
    ]
    .iter()
    .any(|text| text.is_empty() || text.trim() != text.as_str())
        || matching.producer_role.cardinality != Cardinality::ExactlyOne
    {
        return false;
    }
    body.outputs().iter().any(|output| {
        output.binding == succession.binding()
            && output.role.name == matching.selected_role
            && output.role.entity == matching.producer_role.entity
            && output.role.posture == Posture::Preserve
            && output.role.cardinality == Cardinality::ExactlyOne
            && !matches!(
                output.target,
                ApplicationOperationBodyOutputTarget::Creation { .. }
            )
    })
}

pub(in crate::application_schema::operation_definition::body) fn readmit(
    body: &ApplicationOperationBodyProgram,
    members: &[ApplicationSchemaMember],
) -> bool {
    let Some(matching) = body.succession().and_then(|s| s.output_match()) else {
        return true;
    };
    valid(body)
        && members.iter().any(|member| {
            matches!(member,
        ApplicationSchemaMember::ApplicationMutation { description }
        if description.binding_identity().as_str() == matching.producer_binding()
            && description.output_roles().contains(matching.producer_role()))
        })
}

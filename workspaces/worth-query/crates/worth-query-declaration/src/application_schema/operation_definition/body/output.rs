//! Fixed output uses bind the authored operation to its exact mutation contract.
use super::{
    ApplicationOperationBodyAuthoringDenial as Denial, ApplicationOperationBodyBuilder,
    ApplicationOperationBodyCreationBuilder,
};
use crate::application_operation::{
    ApplicationMutationBinding, ApplicationMutationOutputRoleDescription,
    ApplicationMutationScopeBinding, WorthQueryApplicationDeclaredOutputRole,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryPreserveOutput,
};
use crate::application_schema::ApplicationSchema;
mod related;

/// Descriptive target within this body; never a caller-selected entity identity.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyOutputTarget {
    Scope,
    Creation {
        key: String,
    },
    Related {
        relation: String,
        from: String,
        to: String,
    },
}

/// Portable meaning. Fresh readmission and native installation confer authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyOutput {
    pub binding: String,
    pub role: ApplicationMutationOutputRoleDescription,
    pub target: ApplicationOperationBodyOutputTarget,
}

impl ApplicationOperationBodyOutput {
    /// Collision-free descriptive meaning shared by schema and program identity.
    pub fn canonical_record(&self) -> String {
        use crate::application_operation::{
            ApplicationMutationOutputPosture as Posture,
            ApplicationMutationOutputRoleCardinality as Cardinality,
        };
        let (target, key) = match &self.target {
            ApplicationOperationBodyOutputTarget::Scope => ("scope", ""),
            ApplicationOperationBodyOutputTarget::Creation { key } => ("creation", key.as_str()),
            ApplicationOperationBodyOutputTarget::Related { relation, .. } => {
                ("related", relation.as_str())
            }
        };
        let parts = [
            self.binding.as_str(),
            &self.role.name,
            &self.role.entity,
            match self.role.posture {
                Posture::Preserve => "preserve",
                Posture::Create => "create",
                Posture::Retire => "retire",
            },
            match self.role.cardinality {
                Cardinality::ExactlyOne => "exactly-one",
                Cardinality::AtMostOne => "at-most-one",
            },
            target,
            key,
        ];
        let mut record = String::from(
            if matches!(
                self.target,
                ApplicationOperationBodyOutputTarget::Related { .. }
            ) {
                "body-output-v2;"
            } else {
                "body-output-v1;"
            },
        );
        for part in parts {
            use std::fmt::Write;
            write!(record, "{}:{part};", part.len()).expect("String write");
        }
        if let ApplicationOperationBodyOutputTarget::Related { from, to, .. } = &self.target {
            for part in [from, to] {
                use std::fmt::Write;
                write!(&mut record, "{}:{part};", part.len()).expect("String write");
            }
        }
        record
    }
}

impl ApplicationOperationBodyOutputTarget {
    /// Exact native outgoing selection required by a related correspondence.
    pub fn selection_read(
        &self,
    ) -> Option<crate::application_schema::ApplicationOperationDecisionReadTarget> {
        match self {
            Self::Related { relation, from, to } => Some(
                crate::application_schema::ApplicationOperationDecisionReadTarget::Relation {
                    relation: relation.clone(),
                    from: from.clone(),
                    to: to.clone(),
                },
            ),
            _ => None,
        }
    }
    /// Registration and cold readmission share the same dependency closure.
    pub fn decision_reads(
        &self,
    ) -> Vec<crate::application_schema::ApplicationOperationDecisionReadTarget> {
        use crate::application_schema::ApplicationOperationDecisionReadTarget as Read;
        match self {
            Self::Related { from, to, .. } => vec![
                self.selection_read().expect("related selection"),
                Read::Entity {
                    entity: from.clone(),
                },
                Read::Entity { entity: to.clone() },
            ],
            _ => Vec::new(),
        }
    }
}

fn fixed<Schema, Binding, Role>(
    target: ApplicationOperationBodyOutputTarget,
) -> ApplicationOperationBodyOutput
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = Binding::Output>,
{
    const { <Role as WorthQueryApplicationDeclaredOutputRole>::DECLARED };
    let role = <Role as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR;
    ApplicationOperationBodyOutput {
        binding: Binding::IDENTITY.to_owned(),
        role: ApplicationMutationOutputRoleDescription {
            name: role.name().to_owned(),
            entity: role.entity().to_owned(),
            posture: role.posture(),
            cardinality: role.cardinality(),
        },
        target,
    }
}

impl<Schema: ApplicationSchema, Operation, Input>
    ApplicationOperationBodyBuilder<Schema, Operation, Input>
{
    /// Preserve the admitted scope under an exact declared fixed output role.
    /// A Create role cannot be reinterpreted as Preserve.
    ///
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn wrong_action<S, B, R>(body: ApplicationOperationBodyBuilder<S, B::Operation, B::Input>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     B::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = R::Entity>,
    ///     R: WorthQueryApplicationOutputRole<Schema = S, Contract = B::Output, Action = WorthQueryCreateOutput>,
    /// {
    ///     let _ = body.preserve_scope_output::<B, R>();
    /// }
    /// ```
    /// Same-named roles from another contract are not interchangeable.
    ///
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn foreign_contract<S, B, R>(body: ApplicationOperationBodyBuilder<S, B::Operation, B::Input>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     B::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = R::Entity>,
    ///     R: WorthQueryApplicationOutputRole<Schema = S, Action = WorthQueryPreserveOutput>,
    /// {
    ///     let _ = body.preserve_scope_output::<B, R>();
    /// }
    /// ```
    pub fn preserve_scope_output<Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Binding::ScopeBinding: ApplicationMutationScopeBinding<Schema, Scope = Role::Entity>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
    {
        let output = fixed::<Schema, Binding, Role>(ApplicationOperationBodyOutputTarget::Scope);
        self.add_output(output)?;
        Ok(self)
    }
}

impl<Schema: ApplicationSchema, Operation, Input, Entity>
    ApplicationOperationBodyCreationBuilder<Schema, Operation, Input, Entity>
{
    /// Bind this creation slot, not its business key, under an exact Create role.
    /// A role must name the exact entity marker of the creation builder.
    ///
    /// ```compile_fail,E0271
    /// use worth_query_declaration::facade::{application_operation::*, application_schema::*};
    /// fn foreign_entity<S, B, R, E>(body: ApplicationOperationBodyCreationBuilder<S, B::Operation, B::Input, E>)
    /// where S: ApplicationSchema, B: ApplicationMutationBinding<S>,
    ///     R: WorthQueryApplicationOutputRole<Schema = S, Contract = B::Output, Action = WorthQueryCreateOutput>,
    /// {
    ///     let _ = body.create_output::<B, R>();
    /// }
    /// ```
    pub fn create_output<Binding, Role>(mut self) -> Result<Self, Denial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Entity = Entity,
            Action = WorthQueryCreateOutput,
        >,
    {
        let output =
            fixed::<Schema, Binding, Role>(ApplicationOperationBodyOutputTarget::Creation {
                key: self.key.clone(),
            });
        self.parent.add_output(output)?;
        Ok(self)
    }
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    fn add_output(&mut self, output: ApplicationOperationBodyOutput) -> Result<(), Denial> {
        if self.outputs.iter().any(|existing| {
            existing.binding != output.binding || existing.role.name == output.role.name
        }) {
            return Err(Denial::IncompatibleOutput);
        }
        self.outputs.push(output);
        Ok(())
    }
}

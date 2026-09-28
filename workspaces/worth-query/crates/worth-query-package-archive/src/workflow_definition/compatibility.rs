//! Rebuilding typed meaning from an untrusted draft against the vocabulary a
//! workflow spec installed. Every member resolves to the ref a typed author
//! would have declared, or the draft is refused naming the node.

use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAuthoringDenial,
    ApplicationWorkflowCommandAdapter, ApplicationWorkflowCondition,
    ApplicationWorkflowConditionOperand, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowNodeKind, ApplicationWorkflowSpec, ApplicationWorkflowSubjectSelector,
    AuthoredWorkflowDefinition,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaDeclaration,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

use super::{
    DraftCondition, DraftConditionOperand, DraftConnection, DraftMember, DraftNode,
    WorthQueryUntrustedWorkflowDefinitionDraft,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDefinitionDraftDenialKind {
    /// The draft was authored under another workflow spec.
    ForeignSpec,
    /// A node names an operation, binding, query or capability the spec did
    /// not install.
    UnknownMember,
    /// A node names an installed member whose portable types differ from
    /// those the draft was authored against.
    ChangedMember,
    UnknownEvidenceJoinPolicy,
    InvalidSubject,
    /// An assessment applies on a relation the schema does not declare.
    UndeclaredRelation,
    SchemaUnavailable,
    /// The rebuilt nodes and connections do not author a definition.
    Authoring,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowDefinitionDraftDenial {
    kind: WorthQueryWorkflowDefinitionDraftDenialKind,
    subject: String,
    authoring: Option<ApplicationWorkflowAuthoringDenial>,
}

impl WorthQueryWorkflowDefinitionDraftDenial {
    pub const fn kind(&self) -> WorthQueryWorkflowDefinitionDraftDenialKind {
        self.kind
    }

    /// The spec, node or connection the denial names.
    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub const fn authoring(&self) -> Option<&ApplicationWorkflowAuthoringDenial> {
        self.authoring.as_ref()
    }
}

impl std::fmt::Display for WorthQueryWorkflowDefinitionDraftDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "workflow definition draft refused: {:?} ({})",
            self.kind, self.subject
        )?;
        if let Some(authoring) = &self.authoring {
            write!(formatter, ": {authoring:?}")?;
        }
        Ok(())
    }
}

impl std::error::Error for WorthQueryWorkflowDefinitionDraftDenial {}

type Kind = WorthQueryWorkflowDefinitionDraftDenialKind;
type Denial = WorthQueryWorkflowDefinitionDraftDenial;

impl WorthQueryUntrustedWorkflowDefinitionDraft {
    /// Rebuilds the draft as a typed definition against `installed`. The
    /// result is authored, not validated: it still validates, binds and
    /// publishes through the ordinary path.
    pub fn author<Schema, Spec>(
        &self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    ) -> Result<AuthoredWorkflowDefinition<Spec>, Denial>
    where
        Schema: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if self.spec != Spec::IDENTITY.as_str() {
            return Err(denial(Kind::ForeignSpec, &self.spec));
        }
        let mut schema = None;
        let mut commands = Vec::with_capacity(1 + self.nodes.len() + self.connections.len());
        for node in &self.nodes {
            let kind = resolve(node, installed, &mut schema)?;
            let identity = ApplicationWorkflowNodeIdentity::new(node.identity.as_str()).map_err(
                |invalid| {
                    authoring(
                        &node.identity,
                        ApplicationWorkflowAuthoringDenial::InvalidIdentity(invalid),
                    )
                },
            )?;
            commands.push(ApplicationWorkflowAuthoringCommand::Node(identity, kind));
        }
        commands.push(
            ApplicationWorkflowAuthoringCommand::start(self.start.as_str())
                .map_err(|refused| authoring(&self.start, refused))?,
        );
        for connection in &self.connections {
            commands.push(
                command(connection).map_err(|refused| authoring(&connection.source, refused))?,
            );
        }
        ApplicationWorkflowCommandAdapter::author::<Spec>(
            self.identity.as_str(),
            self.limits,
            commands,
        )
        .map_err(|refused| authoring(&self.identity, refused))
    }
}

fn resolve<Schema, Spec>(
    node: &DraftNode,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
    schema: &mut Option<ApplicationSchemaDeclaration<Schema>>,
) -> Result<ApplicationWorkflowNodeKind, Denial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    let subject = node.identity.as_str();
    let unknown = || denial(Kind::UnknownMember, subject);
    let changed = || denial(Kind::ChangedMember, subject);
    Ok(match &node.member {
        DraftMember::Operation {
            identifier,
            input_type,
            binding,
            requires_workflow_authority,
        } => {
            let operation = installed
                .draft_operation(identifier, binding.as_deref())
                .ok_or_else(unknown)?;
            if operation.input_type().as_str() != input_type {
                return Err(changed());
            }
            ApplicationWorkflowNodeKind::Operation {
                operation,
                requires_workflow_authority: *requires_workflow_authority,
            }
        }
        DraftMember::Assessment {
            identifier,
            parameter_type,
            result_type,
            subject: selector,
            related_relation,
        } => {
            let assessment = installed.draft_assessment(identifier).ok_or_else(unknown)?;
            if assessment.parameter_type().as_str() != parameter_type
                || assessment.result_type().as_str() != result_type
            {
                return Err(changed());
            }
            let selector = ApplicationWorkflowSubjectSelector::from_persistence_identity(selector)
                .ok_or_else(|| denial(Kind::InvalidSubject, subject))?;
            let assessment = assessment.for_subject(selector.clone());
            ApplicationWorkflowNodeKind::Assessment(match related_relation {
                None => assessment,
                // A related relation always reads the related subject.
                Some(_) if selector != ApplicationWorkflowSubjectSelector::Related => {
                    return Err(denial(Kind::InvalidSubject, subject))
                }
                Some([relation, from, to]) => {
                    if schema.is_none() {
                        *schema = Some(
                            Schema::declaration()
                                .map_err(|_| denial(Kind::SchemaUnavailable, subject))?,
                        );
                    }
                    let declaration = schema
                        .as_ref()
                        .expect("the schema declaration was just read");
                    assessment
                        .when_related_relation_declared_in(declaration, relation, from, to)
                        .ok_or_else(|| denial(Kind::UndeclaredRelation, subject))?
                }
            })
        }
        DraftMember::Condition(condition) => ApplicationWorkflowNodeKind::Condition(
            resolve_condition(subject, condition, installed)?,
        ),
        DraftMember::Approval {
            identifier,
            capability_type,
        } => {
            let approval = installed.draft_approval(identifier).ok_or_else(unknown)?;
            if approval.capability_type().as_str() != capability_type {
                return Err(changed());
            }
            ApplicationWorkflowNodeKind::Approval(approval)
        }
        DraftMember::EvidenceJoin { policy } => ApplicationWorkflowNodeKind::EvidenceJoin(
            ApplicationWorkflowEvidenceJoinPolicy::from_identity(policy)
                .ok_or_else(|| denial(Kind::UnknownEvidenceJoinPolicy, subject))?,
        ),
        DraftMember::Terminal => ApplicationWorkflowNodeKind::Terminal,
    })
}

/// Every operand resolves to an installed query with its authored types, then
/// the expression readmits over them. A version-1 condition readmits as the
/// migrated expression over its one query.
fn resolve_condition<Schema, Spec>(
    subject: &str,
    condition: &DraftCondition,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
) -> Result<ApplicationWorkflowCondition, Denial>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    let resolve = |operand: &DraftConditionOperand| {
        let query = installed
            .draft_condition_operand(&operand.identifier)
            .ok_or_else(|| denial(Kind::UnknownMember, subject))?;
        if query.parameter_type().as_str() != operand.parameter_type
            || query.result_type().as_str() != operand.result_type
        {
            return Err(denial(Kind::ChangedMember, subject));
        }
        Ok(ApplicationWorkflowConditionOperand::new(
            operand.name.as_str(),
            query,
        ))
    };
    let admitted = match condition {
        DraftCondition::Expression { draft, operands } => ApplicationWorkflowCondition::decode(
            draft,
            operands.iter().map(resolve).collect::<Result<_, _>>()?,
        ),
        DraftCondition::Migrated(operand) => {
            ApplicationWorkflowCondition::migrated(resolve(operand)?.query().clone())
        }
    };
    admitted.map_err(|refused| {
        authoring(
            subject,
            ApplicationWorkflowAuthoringDenial::Condition(refused),
        )
    })
}

fn command(
    connection: &DraftConnection,
) -> Result<ApplicationWorkflowAuthoringCommand, ApplicationWorkflowAuthoringDenial> {
    let (source, target) = (connection.source.as_str(), connection.target.as_str());
    match connection.kind.clone() {
        ApplicationWorkflowConnectionKind::Control(outcome) => {
            ApplicationWorkflowAuthoringCommand::control(source, outcome, target)
        }
        ApplicationWorkflowConnectionKind::Data(flow) => {
            ApplicationWorkflowAuthoringCommand::data(source, flow, target)
        }
        ApplicationWorkflowConnectionKind::Retry(retry) => {
            ApplicationWorkflowAuthoringCommand::retry(source, retry, target)
        }
    }
}

fn denial(kind: Kind, subject: &str) -> Denial {
    Denial {
        kind,
        subject: subject.to_owned(),
        authoring: None,
    }
}

fn authoring(subject: &str, refused: ApplicationWorkflowAuthoringDenial) -> Denial {
    Denial {
        kind: Kind::Authoring,
        subject: subject.to_owned(),
        authoring: Some(refused),
    }
}

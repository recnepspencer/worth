//! A condition node's persisted meaning.
//!
//! An expression condition records its encoded draft, in hex, as the node
//! member and its operands, name-ordered, in one framed field. A record with
//! no operands field is a version-1 condition: its member names the one Bool
//! query it reads. It compiles as the migrated expression over that query
//! and keeps its original node identity, so live instances published before
//! expressions still settle under the identities they were admitted with.

/// The fields one operand record frames, in order.
const OPERAND_FIELDS: usize = 5;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum CompiledWorkflowConditionExpression {
    /// A version-1 record: the one operand's result is the whole decision.
    Migrated,
    /// The encoded expression draft, readmitted at every evaluation.
    Draft(Box<[u8]>),
}

/// One query result a condition reads: the operand name the expression uses,
/// the installed query, its parameter and result types, and the binding whose
/// published result supplies the value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowConditionOperand {
    pub(in crate::domain_computation::primary_graph) name: String,
    pub(in crate::domain_computation::primary_graph) query: String,
    pub(in crate::domain_computation::primary_graph) parameter_type: String,
    pub(in crate::domain_computation::primary_graph) result_type: String,
    pub(in crate::domain_computation::primary_graph) binding: String,
}

impl WorkflowConditionOperand {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn parameter_type(&self) -> &str {
        &self.parameter_type
    }
    pub fn result_type(&self) -> &str {
        &self.result_type
    }
    pub fn binding(&self) -> &str {
        &self.binding
    }

    fn fields(&self) -> [&str; OPERAND_FIELDS] {
        [
            &self.name,
            &self.query,
            &self.parameter_type,
            &self.result_type,
            &self.binding,
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct CompiledWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) expression:
        CompiledWorkflowConditionExpression,
    /// Name-ordered; names are unique.
    pub(in crate::domain_computation::primary_graph) operands: Box<[WorkflowConditionOperand]>,
}

impl CompiledWorkflowCondition {
    /// Reads a persisted condition. `operands` is absent exactly for a
    /// version-1 record, which alone carries the query's types and binding in
    /// their own fields.
    pub(in crate::domain_computation::primary_graph) fn from_record(
        member: String,
        parameter_type: Option<String>,
        result_type: Option<String>,
        binding: Option<String>,
        operands: Option<String>,
    ) -> Option<Self> {
        match (parameter_type, result_type, binding, operands) {
            (Some(parameter_type), Some(result_type), Some(binding), None) => Some(Self {
                expression: CompiledWorkflowConditionExpression::Migrated,
                operands: Box::new([WorkflowConditionOperand {
                    name: worth_query_declaration::facade::application_program::MIGRATED_WORKFLOW_CONDITION_OPERAND.to_owned(),
                    query: member,
                    parameter_type,
                    result_type,
                    binding,
                }]),
            }),
            (None, None, None, Some(operands)) => Some(Self {
                expression: CompiledWorkflowConditionExpression::Draft(decode_draft(&member)?),
                operands: decode_operands(&operands)?,
            }),
            _ => None,
        }
    }

    /// Node identity fields after the node tag. A migrated condition keeps
    /// the version-1 fields exactly; an expression condition always has more
    /// fields, so the two cannot share a material.
    pub(in crate::domain_computation::primary_graph) fn identity_fields(&self) -> Vec<String> {
        match &self.expression {
            CompiledWorkflowConditionExpression::Migrated => self.operands[0].fields()[1..]
                .iter()
                .map(|field| (*field).to_owned())
                .collect(),
            CompiledWorkflowConditionExpression::Draft(draft) => {
                ["expression".to_owned(), encode_draft(draft)]
                    .into_iter()
                    .chain(
                        self.operands
                            .iter()
                            .flat_map(|operand| operand.fields().map(str::to_owned)),
                    )
                    .collect()
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn retained_string_bytes(&self) -> usize {
        let draft = match &self.expression {
            CompiledWorkflowConditionExpression::Migrated => 0,
            CompiledWorkflowConditionExpression::Draft(draft) => draft.len(),
        };
        self.operands
            .iter()
            .flat_map(|operand| operand.fields().map(str::len))
            .fold(draft, usize::saturating_add)
    }
}

/// The member an expression condition persists: its encoded draft in hex.
pub(in crate::domain_computation::primary_graph) fn encode_draft(draft: &[u8]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(draft.len() * 2);
    for byte in draft {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

fn decode_draft(member: &str) -> Option<Box<[u8]>> {
    if member.is_empty() || member.len() % 2 != 0 {
        return None;
    }
    member
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            };
            Some(digit(pair[0])? << 4 | digit(pair[1])?)
        })
        .collect()
}

/// Frames each operand's fields as `length:field`, operands in name order.
pub(in crate::domain_computation::primary_graph) fn encode_operands<'a>(
    operands: impl IntoIterator<Item = [&'a str; OPERAND_FIELDS]>,
) -> String {
    use std::fmt::Write;

    let mut text = String::new();
    for field in operands.into_iter().flatten() {
        write!(&mut text, "{}:{field}", field.len()).expect("writing to a String cannot fail");
    }
    text
}

/// The exact inverse of [`encode_operands`]: canonical lengths, nonempty
/// fields, whole operands, and strictly ascending names.
fn decode_operands(mut text: &str) -> Option<Box<[WorkflowConditionOperand]>> {
    let mut fields = Vec::new();
    while !text.is_empty() {
        let (length, rest) = text.split_once(':')?;
        if length.starts_with('0') || !length.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        let length = length.parse::<usize>().ok()?;
        fields.push(rest.get(..length)?.to_owned());
        text = rest.get(length..)?;
    }
    if fields.is_empty() || fields.len() % OPERAND_FIELDS != 0 {
        return None;
    }
    let operands = fields
        .chunks_exact(OPERAND_FIELDS)
        .map(|operand| WorkflowConditionOperand {
            name: operand[0].clone(),
            query: operand[1].clone(),
            parameter_type: operand[2].clone(),
            result_type: operand[3].clone(),
            binding: operand[4].clone(),
        })
        .collect::<Box<[_]>>();
    operands
        .windows(2)
        .all(|pair| pair[0].name < pair[1].name)
        .then_some(operands)
}

#[cfg(test)]
#[path = "condition/tests.rs"]
mod tests;

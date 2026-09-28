//! Canonical linearization: checked nodes become one post-order program with
//! canonical operand slots and function-closure indices, then the expanded
//! instruction count and call depth are checked before anything is flattened.

use std::collections::{BTreeMap, BTreeSet};

use crate::expressions::denial::{ExpressionResource, ExpressionResult};
use crate::expressions::profile::check_limit;
use crate::expressions::program::{ExpressionProgram, Op, ProgramNode};
use crate::expressions::types::ExpressionType;

use super::{AdmissionContext, CheckedProgram, Checker, Scope};

pub(super) fn finish(checker: Checker<'_>, root: u32) -> ExpressionResult<CheckedProgram> {
    let Checker {
        context,
        scope,
        meter,
        nodes,
        operands,
        functions,
        ..
    } = checker;
    let (slot_of, slots) = slot_mapping(context, scope, &operands);
    let catalog = context.catalog;
    let mut closure: Vec<usize> = functions.into_iter().collect();
    closure.sort_by(|a, b| {
        catalog
            .function(*a)
            .closure_key()
            .cmp(&catalog.function(*b).closure_key())
    });
    let call_of = ranks(closure.iter().copied());

    let order = post_order(&nodes, root);
    let mut renumbered = vec![u32::MAX; nodes.len()];
    let mut pending: Vec<Option<ProgramNode>> = nodes.into_iter().map(Some).collect();
    let mut linear = Vec::with_capacity(order.len());
    let (mut expanded, mut call_depth, mut bit_width) = (0_u64, 0_u32, 0_u32);
    let profile = context.profile;
    for old in order {
        let mut node = pending[old].take().expect("checked nodes form a tree");
        node.children = node
            .children
            .iter()
            .map(|child| renumbered[*child as usize])
            .collect();
        match &mut node.op {
            Op::Operand(index) => *index = slot_of[&(*index as usize)],
            Op::Call(index) => {
                let callee = catalog.function(*index as usize);
                expanded = expanded.saturating_add(callee.expanded_instructions());
                call_depth = call_depth.max(callee.call_depth() + 1);
                bit_width = bit_width.max(callee.bit_width());
                *index = call_of[&(*index as usize)];
            }
            _ => {}
        }
        bit_width = bit_width.max(context.schema.widest_bus(&node.ty));
        renumbered[old] = linear.len() as u32;
        linear.push(node);
    }
    expanded = expanded.saturating_add(linear.len() as u64);
    check_limit(profile, ExpressionResource::ExpandedInstructions, expanded)?;
    check_limit(
        profile,
        ExpressionResource::CallDepth,
        u64::from(call_depth),
    )?;
    check_limit(profile, ExpressionResource::BitWidth, u64::from(bit_width))?;
    Ok(CheckedProgram {
        program: ExpressionProgram::new(linear),
        slots,
        functions: closure,
        work: meter.used(),
        expanded_instructions: expanded,
        call_depth,
        bit_width,
    })
}

/// Operand slots are the used operands in canonical schema order; parameter
/// slots are every parameter in declared order.
fn slot_mapping(
    context: &AdmissionContext<'_>,
    scope: Scope<'_>,
    operands: &BTreeSet<usize>,
) -> (BTreeMap<usize, u32>, Vec<(Box<str>, ExpressionType)>) {
    match scope {
        Scope::Operands => {
            let slots = operands
                .iter()
                .map(|index| {
                    let (name, ty) = context
                        .schema
                        .operand_at(*index)
                        .expect("checked operand index");
                    (Box::from(name), ty.clone())
                })
                .collect();
            (ranks(operands.iter().copied()), slots)
        }
        Scope::Parameters(parameters) => (ranks(0..parameters.len()), parameters.to_vec()),
    }
}

/// Maps each value to its rank in iteration order.
fn ranks(values: impl Iterator<Item = usize>) -> BTreeMap<usize, u32> {
    values
        .enumerate()
        .map(|(rank, value)| (value, rank as u32))
        .collect()
}

/// Children in evaluation order, each before its parent; the root is last.
fn post_order(nodes: &[ProgramNode], root: u32) -> Vec<usize> {
    let mut order = Vec::with_capacity(nodes.len());
    let mut stack = vec![(root as usize, 0_usize)];
    while let Some(&(node, next)) = stack.last() {
        match nodes[node].children.get(next) {
            Some(child) => {
                let top = stack.len() - 1;
                stack[top].1 += 1;
                stack.push((*child as usize, 0));
            }
            None => {
                order.push(node);
                stack.pop();
            }
        }
    }
    order
}

"""Destructuring doors: `into_parts_for_test(self)` returns every field, in declaration order.

A test that moved several fields out of one owned value (partial moves) cannot do so through
by-value field doors, each of which consumes the whole value. The struct's parts door gives
all fields at once; the site destructures it (that site change is a hand fix).
"""
from module_tree import read_tree
from src_door_members_text import DOC, from_child, impl_header, self_type, struct_shape
from src_door_writes import Plan, apply_plan, struct_declaration

LIB = 'workspaces/worth-query/crates/worth-query-execution/src/lib.rs'


def parts_door(shape):
    fields = shape['fields']
    types = ', '.join(from_child(t) for t in fields.values())
    values = ', '.join(f'self.{name}' for name in fields)
    return (impl_header(shape['generics'], self_type(shape), shape['where']) + DOC +
            f'    pub fn into_parts_for_test(self) -> ({types},) {{\n        ({values},)\n    }}\n}}')


def parts(root, names, dry_run=False):
    """Write the parts door of each named struct; returns (written, problems)."""
    tree = read_tree(root / LIB)
    index = {node.names: node for node in tree.walk()}
    plan = Plan(index)
    for name in names:
        found = struct_declaration(index, name)
        if found is None:
            return 0, [f'no single struct named {name}']
        plan.door(found[0].names, ('parts', 'into_parts', name), parts_door(struct_shape(found[1])))
    problems = apply_plan(plan, dry_run)
    return len(names), problems

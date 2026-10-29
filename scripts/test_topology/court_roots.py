"""Generate one binary and category-first module tree for each product frontier."""
from collections import defaultdict
import json
import re

from court_partition import CATEGORIES, INTERIM
from module_tree import without_tests
from source_rewrite import relpath


def area_names(names):
    return tuple(n for i, n in enumerate(names) if not (i == 0 and n in (*INTERIM, *CATEGORIES)))


def origin_name(owner):
    return owner['cargo']['name'].removeprefix('worth-').replace('-', '_')


def target_area(unit):
    target = unit['owner']['targets'][unit['target_index']]['cargo']['name']
    package = unit['owner']['cargo']['name'].replace('-', '_')
    return () if target == package else (target,)


def module_path(unit):
    category = tuple(unit['category'].split('::')) if unit['category'] else ()
    return category + (origin_name(unit['owner']),) + target_area(unit) + area_names(unit['node'].names)


def prologue(text):
    # Context comes from existing ancestor scopes. Generated files cannot put an
    # inner doc comment after another ancestor's items.
    return re.sub(r'(?m)^(\s*)//!', r'\1//', text).strip() + '\n'


def render(court, workspace, assignments, aliases, effects, unit_text, context_text):
    name = court['name']
    base = workspace / 'crates' / name
    binary = base / 'tests' / (name.replace('-', '_') + '.rs')
    directory = binary.with_suffix('')
    scopes, mounts, support = defaultdict(set), {}, {}
    for unit in court['units']:
        node, path = unit['node'], module_path(unit)
        root = unit['owner']['targets'][unit['target_index']]['tree']
        origin_path = (tuple(unit['category'].split('::')) if unit['category'] else ()) + (origin_name(unit['owner']),) + target_area(unit)
        if root is not node or not unit['own_only']:
            context_path = origin_path if target_area(unit) else ()
            scopes[context_path].add(prologue(context_text(without_tests(root.own_text()), unit, root.source)))
        for ancestor in unit['ancestors'][1:]:
            ancestor_path = tuple(unit['category'].split('::')) if unit['category'] else ()
            ancestor_path += (origin_name(unit['owner']),) + target_area(unit) + area_names(ancestor.names)
            scopes[ancestor_path].add(prologue(context_text(without_tests(ancestor.own_text()), unit, ancestor.source)))
        scopes[origin_path].add('#[allow(unused_imports)]\nuse crate::*;\n')
        for shared in unit['support']:
            if len(shared.names) == 1:
                key = (origin_path if target_area(unit) else (), shared.names[-1])
                if key in support and support[key].source != shared.source:
                    raise ValueError('root support alias collision: ' + shared.names[-1])
            else:
                parent = origin_path + area_names(shared.names[:-1])
                key = (parent, shared.names[-1])
            support[key] = shared
        if unit['own_only']:
            target = directory.joinpath(*path, 'mod.rs')
            scopes[path].add(prologue(unit_text(unit, target)))
        else:
            if path in mounts and mounts[path][0].source != node.source:
                raise ValueError('module path collision: ' + '::'.join(path))
            mounts[path] = (node, unit)
    all_paths = set(scopes) | set(mounts) | {path for path, _alias in support}
    for path in list(all_paths):
        all_paths.update(path[:i] for i in range(len(path)))
    source_for = {path: directory.joinpath(*path, 'mod.rs') if path else binary for path in all_paths}
    for (parent, alias), node in sorted(support.items()):
        attrs = re.sub(r'#\s*\[\s*path\s*=\s*(?:r#*".*?"#*|"(?:\\.|[^"\\])*")\s*\]\s*', '', node.declaration.attributes, flags=re.S) if node.declaration else ''
        assignment = assignments.get(node.source)
        if assignment and '-testkit' in assignment['owner']:
            line = attrs + 'pub(crate) use ' + assignment['owner'].replace('-', '_') + '::' + aliases[node.source] + ' as ' + alias + ';\n'
        else:
            target = assignment['destination'] if assignment else node.source
            line = attrs + '#[path = ' + json.dumps(relpath(target, source_for[parent].parent)) + ']\npub(crate) mod ' + alias + ';\n'
        scopes[parent].add(line)
    for path in sorted(all_paths, key=lambda p: (len(p), p)):
        if path in mounts:
            node, unit = mounts[path]
            # A source unit already includes its descendants. Its physical source
            # is mounted directly; wrappers exist only above the selected unit.
            continue
        source = source_for[path]
        text = ''.join(sorted(scopes[path]))
        children = sorted(p for p in all_paths if len(p) == len(path) + 1 and p[:-1] == path)
        for child_path in children:
            attrs = ''
            if child_path in mounts:
                node, unit = mounts[child_path]
                assignment = assignments.get(node.source)
                target = assignment['destination'] if assignment else node.source
                if node.offset:
                    target = directory.joinpath(*child_path).with_suffix('.rs')
                    effects.write(target, unit_text(unit, target))
                if node.declaration:
                    attrs = re.sub(r'#\s*\[\s*path\s*=\s*(?:r#*".*?"#*|"(?:\\.|[^"\\])*")\s*\]\s*', '', node.declaration.attributes, flags=re.S)
            else:
                target = source_for[child_path]
            text += attrs + '#[path = ' + json.dumps(relpath(target, source.parent)) + ']\npub(crate) mod ' + child_path[-1] + ';\n'
        effects.write(source, text)
    return binary

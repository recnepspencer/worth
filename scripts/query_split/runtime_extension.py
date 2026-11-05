#!/usr/bin/env python3
"""Turn one module's inherent impl blocks of the application runtime into extension traits.

Usage:
    python scripts/query_split/runtime_extension.py <lib.rs> <module::path> --host <HostTrait>
        [--facade-file <path>] [--report DIR] [--dry-run]

For every production impl block of `WorthQueryPrimaryGraphApplicationRuntime` inside the module
(its test doors included), each group of methods sharing a file and a visibility becomes one
extension trait in that file, with the original visibility, over the module's host trait:

    <vis> trait <Stem>Ops<Schema: Bounds>: <Host><Schema> { <methods, visibility dropped> }
    impl<Schema: Bounds, H: <Host><Schema> + ?Sized> <Stem>Ops<Schema> for H {}

Inside the moved bodies `self.f` (a runtime field) and `self.runtime.g` (a core field) become
host accessors `self.f()` / `self.g()`; `self.runtime.m(..)` (a core method) becomes the host
method `self.m(..)`; calls to runtime methods defined outside the module become host methods.
The report lists every accessor, core method and foreign method the host trait must declare
(with the foreign method's signature), and every construct left for a hand fix. Public
non-test methods also get a facade method on the runtime (`--facade-file`), which only calls
the extension trait (ruling A). The host trait and its implementation for the runtime are
written by hand from the report.
"""
import argparse
from collections import defaultdict
import json
from pathlib import Path
import re
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from item_mentions import lookup  # noqa: E402
from module_graph import LibraryModel  # noqa: E402
from ownership_units import header_parts  # noqa: E402
from rust_source import pairs, tokens  # noqa: E402
from src_families import test_gated  # noqa: E402
from src_scope import scope_items  # noqa: E402

RUNTIME = ('domain_computation', 'primary_graph', 'application_runtime')
RUNTIME_NAME = 'WorthQueryPrimaryGraphApplicationRuntime'


def camel(stem):
    return ''.join(part[:1].upper() + part[1:] for part in stem.split('_') if part)


def is_call(stream, k):
    if k >= len(stream):
        return False
    return stream[k].value == '(' or (stream[k].value == ':' and k + 1 < len(stream)
                                      and stream[k + 1].value == ':')


def split_header(body):
    """(generic declaration text, where text) of an impl header."""
    stream = tokens(body)
    matching = pairs(stream)
    first = next(k for k, t in enumerate(stream) if t.value == 'impl')
    generics = ''
    k = first + 1
    if k < len(stream) and stream[k].value == '<':
        depth, j = 0, k
        while True:
            if stream[j].value == '<':
                depth += 1
            elif stream[j].value == '>' and stream[j - 1].value != '-':
                depth -= 1
                if depth == 0:
                    break
            j += 1
        generics = body[stream[k].start:stream[j].end]
    opening = next(k for k, t in enumerate(stream) if t.value == '{' and
                   not any(a < k <= b for a, b in matching.items() if stream[a].value in '(['))
    head = body[:stream[opening].start]
    where = ''
    w = re.search(r'\bwhere\b', head)
    if w:
        where = head[w.start():].strip()
    return generics, where, stream[opening].end, stream[matching[opening]].start


def visibility_of(item):
    m = re.match(r'\s*((?:#\[[^\]]*\]\s*|///[^\n]*\n\s*|//[^\n]*\n\s*)*)\s*(pub(?:\s*\([^)]*\))?)?', item)
    vis = ' '.join((m.group(2) or '').split())
    return vis.replace('pub (', 'pub(').replace('( ', '(').replace(' )', ')')


def runtime_blocks(model, key, prefix):
    """Every runtime impl block under `prefix` (test doors included, test-gated excluded)."""
    found = []
    for names in sorted(model.views):
        if names[:len(prefix)] != prefix or 'tests' in names:
            continue
        node = model.index[names]
        text = node.text
        for attributes, _vis, keyword, _name, start, end in scope_items(text):
            if keyword != 'impl' or test_gated(attributes):
                continue
            body = text[start:end]
            generic_names, trait_path, self_path, _h = header_parts(body)
            if trait_path or not self_path:
                continue
            target = lookup(model, names, self_path)
            if target is None or target[0] != 'item' or target[1] != key:
                continue
            generics, where, inner_start, inner_end = split_header(body)
            inner = body[inner_start:inner_end]
            methods, others = [], []
            for attrs, vis, kw, name, s, e in scope_items(inner):
                if kw != 'fn':
                    others.append((kw, name))
                    continue
                methods.append({'name': name, 'item': inner[s:e], 'vis': visibility_of(inner[s:e]),
                                'gated': test_gated(attrs)})
            found.append({'names': names, 'node': node, 'start': node.offset + start,
                          'end': node.offset + end, 'generics': generics, 'where': where,
                          'generic_names': sorted(generic_names), 'methods': methods,
                          'others': others})
    return found


def rewrite_body(item, schema, local, core_fields, top_fields, needs, receiver='self'):
    """Host accessors for fields, host methods for core and foreign calls; edits in place."""
    stream = tokens(item)
    edits = []
    for k, t in enumerate(stream):
        if t.kind != 'ident' or t.value != receiver or k + 2 >= len(stream) or stream[k + 1].value != '.':
            continue
        if k >= 1 and (stream[k - 1].value == '.' or (stream[k - 1].value == ':' and k >= 2
                                                       and stream[k - 2].value == ':')):
            continue
        name_tok = stream[k + 2]
        if name_tok.kind != 'ident':
            continue
        name = name_tok.value
        if name == 'runtime':
            if k + 4 < len(stream) and stream[k + 3].value == '.' and stream[k + 4].kind == 'ident':
                sub = stream[k + 4].value
                if is_call(stream, k + 5):
                    needs['core_methods'].add(sub)
                    edits.append((stream[k + 2].start, stream[k + 4].start, ''))
                elif sub in core_fields:
                    needs['core_fields'].add(sub)
                    edits.append((stream[k + 2].start, stream[k + 4].end, sub + '()'))
                    whole = not (k + 5 < len(stream) and stream[k + 5].value in ('.', '['))
                    if whole and k >= 1 and stream[k - 1].value == '&' and \
                            not (k >= 2 and stream[k - 2].value == '&'):
                        edits.append((stream[k - 1].start, stream[k - 1].end, ''))
                else:
                    needs['hand'].append(f'self.runtime.{sub}')
            else:
                needs['hand'].append('self.runtime (whole core)')
            continue
        if is_call(stream, k + 3):
            if name not in local:
                needs['foreign'].add(name)
            continue
        if name in top_fields:
            needs['top_fields'].add(name)
            edits.append((name_tok.start, name_tok.end, name + '()'))
            whole = not (k + 3 < len(stream) and stream[k + 3].value in ('.', '['))
            if whole and k >= 1 and stream[k - 1].value == '&' and not (k >= 2 and stream[k - 2].value == '&'):
                edits.append((stream[k - 1].start, stream[k - 1].end, ''))
        else:
            needs['hand'].append(f'self.{name}')
    for k, t in enumerate(stream):
        if t.value == 'Self' and k + 3 < len(stream) and stream[k + 1].value == ':' and \
                stream[k + 2].value == ':' and stream[k + 3].kind == 'ident':
            target = stream[k + 3].value
            if target not in local and receiver == 'self':
                needs['foreign'].add(target)
        elif receiver == 'self' and t.value == 'Self' and not (k + 1 < len(stream) and stream[k + 1].value == ':'):
            needs['hand'].append('Self used as a type')
    for a, b, r in sorted(edits, reverse=True):
        item = item[:a] + r + item[b:]
    return item


RUNTIME_TYPE = re.compile(r"&\s*('\w+\s+)?(?:(?:crate|super|self)(?:::\w+)*::)?" + RUNTIME_NAME +
                          r"\s*<\s*(\w+)\s*,?\s*>")


def rewrite_parameters(text, host_path, local, core_fields, top_fields, needs, skip):
    """`name: &Runtime<S>` parameters become `&dyn Host<S>`; their uses become host calls."""
    stream = tokens(text)
    matching = pairs(stream)
    edits = []
    for k, t in enumerate(stream):
        if t.value != 'fn' or t.kind != 'ident' or any(a <= t.start < b for a, b in skip):
            continue
        j = k + 2
        while j < len(stream) and stream[j].value != '(':
            if stream[j].value == '<':
                depth = 0
                while True:
                    if stream[j].value == '<':
                        depth += 1
                    elif stream[j].value == '>' and stream[j - 1].value != '-':
                        depth -= 1
                        if depth == 0:
                            break
                    j += 1
            j += 1
        if j >= len(stream):
            continue
        close = matching[j]
        params = text[stream[j].end:stream[close].start]
        found = []
        for m in re.finditer(r'(\w+)\s*:\s*' + RUNTIME_TYPE.pattern, params):
            found.append((m.group(1), stream[j].end + m.start(0) + len(m.group(1)), stream[j].end + m.end(0),
                          m.group(2) or '', m.group(3)))
        if not found:
            continue
        b = close + 1
        while b < len(stream) and stream[b].value not in ('{', ';'):
            b = matching[b] + 1 if stream[b].value in '([' else b + 1
        if b >= len(stream) or stream[b].value != '{':
            continue
        body_start, body_end = stream[b].start, stream[matching[b]].end
        body = text[body_start:body_end]
        for name, a, e, lifetime, schema in found:
            body = rewrite_body(body, schema, set(), core_fields, top_fields, needs, receiver=name)
        edits.append((body_start, body_end, body))
        for name, a, e, lifetime, schema in found:
            colon = text.index(':', a)
            edits.append((colon + 1, e, f' &{lifetime}dyn {host_path}<{schema}>'))
    for a, b, r in sorted(edits, reverse=True):
        text = text[:a] + r + text[b:]
    return text, len([1 for e in edits])


def flatten_use(tree, base=''):
    """`a::{b::{C, D}, E as F}` -> [('a::b::C', None), ('a::b::D', None), ('a::E', 'F')]."""
    tree = tree.strip()
    depth, brace = 0, None
    for i, ch in enumerate(tree):
        if ch == '{' and depth == 0:
            brace = i
            break
        depth += ch in '<'
        depth -= ch in '>'
    if brace is None:
        alias = None
        if ' as ' in tree:
            tree, alias = [x.strip() for x in tree.split(' as ', 1)]
        path = (base + tree).rstrip(':')
        if path.endswith('::self'):
            path = path[:-len('::self')]
        return [(path, alias)]
    head = base + tree[:brace]
    inner = tree[brace + 1:tree.rindex('}')]
    parts, depth, cur = [], 0, ''
    for ch in inner:
        if ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
        if ch == ',' and depth == 0:
            parts.append(cur)
            cur = ''
        else:
            cur += ch
    parts.append(cur)
    out = []
    for part in parts:
        if part.strip():
            out.extend(flatten_use(part, head))
    return out


def file_uses(source, names, file_module):
    """The file's top-level `use` items, written from `crate::` so another module can hold them."""
    text = source.read_text(encoding='utf-8')
    module = file_module.get(source, names)
    found = []
    for attributes, vis, keyword, _name, start, end in scope_items(text):
        if keyword != 'use' or vis:
            continue
        item = ' '.join(text[start:end].split())
        body = item[len('use '):].rstrip(';')
        if body.startswith('super::') or body.startswith('self::') or body.startswith('{'):
            base = list(module)
            while body.startswith('super::'):
                base.pop()
                body = body[len('super::'):]
            if body.startswith('self::'):
                body = body[len('self::'):]
            body = '::'.join(['crate'] + base) + '::' + body
        for path, alias in flatten_use(body):
            found.append((path, alias))
    return found


def facade_imports(uses):
    bound, lines = {'ApplicationSchema'}, []
    for path, alias in uses:
        name = alias or path.rsplit('::', 1)[-1]
        if name == '*':
            line = f'use {path};'
        elif name in bound:
            continue
        else:
            bound.add(name)
            line = f'use {path} as {alias};' if alias else f'use {path};'
        if line not in lines:
            lines.append(line)
    return ''.join(line + '\n' for line in lines)


def rewrite_holders(text, host_path, core_fields, top_fields, needs):
    """Struct fields and accessor returns typed `&Runtime<S>` become `&dyn Host<S>`; uses through
    them (`.f.runtime.g`, `.f().installed_schema`) become host calls."""
    holders = set()
    edits = []
    for m in re.finditer(r"(?m)^(\s*(?:pub(?:\([^)]*\))?\s+)?)(\w+)\s*:\s*" + RUNTIME_TYPE.pattern + r"\s*,", text):
        holders.add(m.group(2))
        a = text.index(':', m.start(2)) + 1
        b = m.end() - 1
        edits.append((a, b, f" &{m.group(3) or ''}dyn {host_path}<{m.group(4)}>"))
    for m in re.finditer(r"fn\s+(\w+)\s*(<[^(]*>)?\s*\(\s*&\s*self\s*\)\s*->\s*" + RUNTIME_TYPE.pattern, text):
        if re.search(r'(?<![\w(])pub\s+(const\s+)?$', text[max(0, m.start() - 20):m.start()]):
            continue  # a public accessor keeps handing out the runtime (consumer API)
        holders.add(m.group(1))
        start = m.start(0) + m.group(0).index('->') + 2
        edits.append((start, m.end(0), f" &{m.group(3) or ''}dyn {host_path}<{m.group(4)}>"))
    for a, b, r in sorted(edits, reverse=True):
        text = text[:a] + r + text[b:]
    if not holders:
        return text, holders
    stream = tokens(text)
    edits = []
    for k, t in enumerate(stream):
        if t.value != '.' or k + 1 >= len(stream) or stream[k + 1].value not in holders:
            continue
        j = k + 2
        if j + 1 < len(stream) and stream[j].value == '(' and stream[j + 1].value == ')':
            j += 2
        if j + 2 < len(stream) and stream[j].value == '.' and stream[j + 1].value == 'runtime' \
                and stream[j + 2].value == '.' and j + 3 < len(stream) and stream[j + 3].kind == 'ident':
            sub = stream[j + 3].value
            if is_call(stream, j + 4):
                needs['core_methods'].add(sub)
                edits.append((stream[j + 1].start, stream[j + 3].start, ''))
            elif sub in core_fields:
                needs['core_fields'].add(sub)
                edits.append((stream[j + 1].start, stream[j + 3].end, sub + '()'))
        elif j + 1 < len(stream) and stream[j].value == '.' and stream[j + 1].value in top_fields \
                and not is_call(stream, j + 2):
            needs['top_fields'].add(stream[j + 1].value)
            edits.append((stream[j + 1].end, stream[j + 1].end, '()'))
    for a, b, r in sorted(edits, reverse=True):
        text = text[:a] + r + text[b:]
    return text, holders


def drop_visibility(item):
    return re.sub(r'(?m)^(\s*)pub(?:\s*\([^)]*\))?\s+(?=(?:const\s+|async\s+|unsafe\s+)*fn\b)', r'\1', item, count=1)


def struct_fields(text, name):
    m = re.search(r'pub struct ' + name + r'(<[^{]*>)?\s*\{', text)
    stream = tokens(text[m.end() - 1:])
    matching = pairs(stream)
    inner = text[m.end():m.end() - 1 + stream[matching[0]].start]
    return set(re.findall(r'(?m)^\s*(?:pub(?:\([^)]*\))?\s+)?(\w+)\s*:', inner))


def facade_args(signature):
    """The forwarded argument list, or None when a parameter is not a plain binding."""
    stream = tokens(signature)
    matching = pairs(stream)
    fn_at = next(k for k, t in enumerate(stream) if t.value == 'fn' and t.kind == 'ident')
    open_paren = next(k for k, t in enumerate(stream) if k > fn_at and t.value == '(')
    inner = signature[stream[open_paren].end:stream[matching[open_paren]].start]
    depth, parts, cur = 0, [], ''
    for ch in inner:
        if ch in '<([{':
            depth += 1
        elif ch in '>)]}':
            depth -= 1
        if ch == ',' and depth == 0:
            parts.append(cur)
            cur = ''
        else:
            cur += ch
    parts.append(cur)
    args = []
    for part in parts:
        part = part.strip()
        if not part or part.endswith('self') or part in ('&self', '&mut self', 'self', 'mut self'):
            continue
        pattern = part.split(':', 1)[0].strip()
        pattern = pattern.removeprefix('mut ').strip()
        if not re.fullmatch(r'[a-z_][a-z0-9_]*', pattern) or pattern == '_':
            return None
        args.append(pattern)
    return args


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('lib')
    parser.add_argument('module')
    parser.add_argument('--host', required=True)
    parser.add_argument('--host-path', required=True, help='crate path of the host trait')
    parser.add_argument('--facade-file')
    parser.add_argument('--report', type=Path)
    parser.add_argument('--root', type=Path, default=Path('.'))
    parser.add_argument('--dry-run', action='store_true')
    parser.add_argument('--params-only', action='store_true')
    args = parser.parse_args(argv)
    root = args.root.resolve()
    model = LibraryModel(args.lib, root)
    key = lookup(model, RUNTIME, (RUNTIME_NAME,))[1]
    prefix = tuple(args.module.split('::'))
    blocks = [] if args.params_only else runtime_blocks(model, key, prefix)
    file_module = {}
    for names, node in model.index.items():
        if getattr(node, 'offset', 0) == 0 and node.source is not None:
            src = Path(node.source)
            if src not in file_module or len(names) > len(file_module[src]):
                file_module[src] = names
    lib_dir = Path(args.lib).resolve().parent
    runtime_text = (lib_dir / 'domain_computation/primary_graph/application_runtime.rs').read_text()
    core_text = (lib_dir / 'domain_computation/primary_graph/application_runtime_core.rs').read_text()
    top_fields = struct_fields(runtime_text, RUNTIME_NAME) - {'runtime'}
    core_fields = struct_fields(core_text, 'WorthQueryApplicationRuntimeCore')
    local = {m['name'] for b in blocks for m in b['methods']}
    needs = {'core_fields': set(), 'top_fields': set(), 'core_methods': set(), 'foreign': set(),
             'hand': []}
    file_edits = defaultdict(list)
    facades, traits = [], []
    facade_uses = []
    used_names = defaultdict(set)
    module_names = set()
    reexports = []
    for block in blocks:
        source = Path(block['node'].source)
        generic_decl = block['generics'] or '<Schema>'
        params = [p.strip() for p in generic_decl[1:-1].split(',')]
        schema = next((p.split(':')[0].strip() for p in params if not p.startswith("'")), 'Schema')
        if len([p for p in params if not p.startswith("'")]) != 1 or any(p.startswith("'") for p in params):
            needs['hand'].append(f'{source}: impl generics {generic_decl}')
        bound = generic_decl[1:-1]
        groups = defaultdict(list)
        for m in block['methods']:
            if m['gated']:
                needs['hand'].append(f'{source}: test-gated method {m["name"]}')
            groups[m['vis']].append(m)
        texts = []
        for vis, methods in groups.items():
            stem = camel(source.stem if source.stem not in ('mod', 'court_doors_for_test') else
                         (source.parent.name if source.stem == 'mod' else source.parent.name + '_doors'))
            tag = '' if vis == 'pub' else {'': 'Own', 'pub(super)': 'Parent'}.get(vis, 'Scoped')
            name = stem + tag + 'Ops'
            while name in module_names:
                name = camel(source.parent.name) + name
            module_names.add(name)
            used_names[source].add(name)
            if vis.startswith('pub(in ') or vis in ('pub', 'pub(crate)'):
                owner = block['names']
                if owner != prefix:
                    reexports.append((vis, '::'.join(('crate',) + owner) + '::' + name))
            bodies = []
            for m in methods:
                if re.search(r'\bconst\s+fn\b', m['item']):
                    needs['hand'].append(f'{source}: const fn {m["name"]}')
                item = rewrite_body(m['item'], schema, local, core_fields, top_fields, needs)
                item = re.sub(r'\bconst\s+fn\b', 'fn', drop_visibility(item))
                bodies.append(item.strip('\n'))
                public = vis == 'pub' and not m['name'].endswith('_for_test')
                if public and args.facade_file:
                    facade_uses.extend(file_uses(source, block['names'], file_module))
                    sig = m['item'][:m['item'].index('{')] if '{' in m['item'] else m['item']
                    fargs = facade_args(sig)
                    if fargs is None:
                        needs['hand'].append(f'facade arguments of {m["name"]}')
                        continue
                    path = '::'.join(('crate',) + prefix) + '::' + name
                    facades.append((schema, sig.rstrip() + ' {\n        <Self as ' + path + f'<{schema}>>::' +
                                   m['name'] + '(self' + ''.join(', ' + a for a in fargs) + ')\n    }'))
            trait_vis = (vis + ' ') if vis else ''
            where = (' ' + block['where']) if block['where'] else ''
            if source in file_module and len(block['names']) < len(file_module[source]):
                needs['hand'].append(f'{source}: module path {block["names"]} vs file {file_module[source]}')
            texts.append(f"{trait_vis}trait {name}<{bound}>: {args.host_path}<{schema}>{where} {{\n" +
                         '\n\n'.join(bodies) + '\n}\n\n' +
                         f"impl<{bound}, HostRuntime: {args.host_path}<{schema}> + ?Sized> {name}<{schema}>"
                         f" for HostRuntime{where} {{}}")
            traits.append({'file': str(source), 'trait': name, 'visibility': vis or 'private',
                           'methods': [m['name'] for m in methods]})
        if block['others']:
            needs['hand'].append(f'{source}: non-fn items {block["others"]}')
        file_edits[source].append((block['start'], block['end'], '\n\n'.join(texts)))
    report = {'module': args.module, 'blocks': len(blocks), 'methods': len(local),
              'traits': traits, 'facades': len(facades),
              'host': {k: sorted(v) if isinstance(v, set) else v for k, v in needs.items()}}
    if args.report:
        args.report.mkdir(parents=True, exist_ok=True)
        (args.report / f'runtime-extension-{prefix[-1]}.json').write_text(json.dumps(report, indent=1) + '\n')
    print(json.dumps({k: (len(v) if isinstance(v, (list, dict)) else v) for k, v in report.items()}))
    print(json.dumps(report['host'], indent=1))
    if args.dry_run:
        return 0
    module_files = {Path(model.index[n].source) for n in model.views if n[:len(prefix)] == prefix}
    for path in module_files - set(file_edits):
        file_edits[path] = []
    for path, edits in file_edits.items():
        text = path.read_text(encoding='utf-8')
        if args.params_only:
            text, _n = rewrite_parameters(text, args.host_path, local, core_fields, top_fields, needs,
                                          [(a, b) for a, b, _r in edits])
            text, _h = rewrite_holders(text, args.host_path, core_fields, top_fields, needs)
        for a, b, r in sorted(edits, reverse=True):
            text = text[:a] + r + text[b:]
        tokens(text)
        pairs(tokens(text))
        path.write_text(text, encoding='utf-8', newline='\n')
    if reexports:
        additions = defaultdict(list)
        for vis, path in reexports:
            segments = path.split('::')[1:]
            name, owner = segments[-1], tuple(segments[:-1])
            for depth in range(len(owner) - 1, len(prefix) - 1, -1):
                ancestor, child = owner[:depth], owner[depth]
                node = model.index[ancestor]
                if node.declaration is not None and node.declaration.body:
                    needs['hand'].append(f're-export of {name} through inline module {ancestor}')
                    break
                additions[Path(node.source)].append(f'{vis} use {child}::{name};')
        for target, lines in additions.items():
            text = target.read_text(encoding='utf-8').rstrip('\n') + '\n'
            for line in lines:
                if line not in text:
                    text += line + '\n'
            target.write_text(text, encoding='utf-8', newline='\n')
    if facades and args.facade_file:
        Path(args.facade_file).write_text(
            f'//! The runtime\'s consumer methods over `{args.module}`; each only calls its subsystem.\n'
            'use worth_query_installation::facade::ApplicationSchema;\n' +
            facade_imports(facade_uses) + '\n'
            ''.join(f'impl<{p}: ApplicationSchema> super::super::{RUNTIME_NAME}<{p}> {{\n    ' +
                    '\n\n    '.join(t for q, t in facades if q == p) + '\n}\n\n'
                    for p in sorted({q for q, _t in facades})), encoding='utf-8', newline='\n')
    return 0


if __name__ == '__main__':
    sys.exit(main())

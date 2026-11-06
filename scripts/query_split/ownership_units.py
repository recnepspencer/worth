"""Ownership units: a type with every inherent impl and every trait impl the crate holds for it.

A unit only moves whole, and must survive a later crate split, so an inherent impl, and an
impl of a trait from outside the crate, sit with the type. An impl of one of this crate's
traits may sit with the trait instead; it joins the type's unit only when the trait lives at
or below the place the type would move to. A trait's own unit takes its declaration (default
bodies included), the impls for types from outside the crate and the blanket impls.

`units(model)` reads every production impl block once; `classify` judges, against a target
stack, each type referenced from below its owner:
- movable: the declaration and all its impls depend only on modules at or below the
  destination (the lowest module referencing it from below), or on types that can move
  there with it;
- behaviour-coupled: some impl (or the declaration) needs a non-type item above the
  destination; the impl and the items are named;
- pinned: no declaration to move (macro-made, or an impl's type unresolved).
"""
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'test_topology'))

from item_mentions import declarations, function_mentions, generic_parameters, lookup, mention_paths  # noqa: E402
from rust_source import pairs, tokens  # noqa: E402
from src_families import test_gated  # noqa: E402
from src_scope import scope_items  # noqa: E402

TYPE_KINDS = ('struct', 'enum', 'trait', 'type', 'union')
SKIP = {'&', 'mut', 'dyn', 'impl', '(', ')', '[', ']', '*', 'const'}


@dataclass
class Impl:
    module: tuple
    header: str
    self_key: object          # (module, name) of a crate type, or None
    self_generic: bool
    trait: object             # ('item', key), ('external', path) or None for an inherent impl
    mentions: list = field(default_factory=list)   # resolved (module, name, kind)
    lines: int = 0


@dataclass
class Unit:
    key: tuple
    keyword: str
    declaration: object = None
    impls: list = field(default_factory=list)

    @property
    def label(self):
        return '::'.join(self.key[0] + (self.key[1],))


def type_path(stream, start, end):
    """Segments of the first path between two token indices, past references and `dyn`."""
    i = start
    while i < end and (stream[i].value in SKIP or stream[i].value == "'" or
                       (i and stream[i - 1].value == "'")):
        i += 1
    segments = []
    while i < end and stream[i].kind == 'ident':
        segments.append(stream[i].value)
        if i + 2 < end and stream[i + 1].value == ':' and stream[i + 2].value == ':':
            i += 3
        else:
            break
    return tuple(segments)


def header_parts(text):
    """(generic names, trait segments or None, self segments, header text) of an impl block."""
    stream = tokens(text)
    matching = pairs(stream)
    first = next((k for k, t in enumerate(stream) if t.value == 'impl' and t.kind == 'ident'), 0)
    stream, i, generics = stream[first:], 1, set()
    matching = {a - first: b - first for a, b in matching.items() if a >= first}
    if i < len(stream) and stream[i].value == '<':
        generics = generic_parameters(stream, matching, i)
        depth = 0
        while i < len(stream):
            if stream[i].value == '<':
                depth += 1
            elif stream[i].value == '>' and stream[i - 1].value != '-':
                depth -= 1
                if depth == 0:
                    i += 1
                    break
            i += 1
    start, depth, split, end = i, 0, None, i
    while end < len(stream):
        value = stream[end].value
        if value in ('(', '['):
            end = matching[end] + 1
            continue
        if value == '<':
            depth += 1
        elif value == '>' and stream[end - 1].value != '-':
            depth -= 1
        elif depth == 0 and value in ('{', 'where'):
            break
        elif depth == 0 and value == 'for' and split is None:
            split = end
        end += 1
    header = text[stream[0].start:stream[end - 1].end] if end > 0 else ''
    if split is None:
        return generics, None, type_path(stream, start, end), header
    trait_start = start + (1 if stream[start].value in ('!', '?') else 0)
    return generics, type_path(stream, trait_start, split), type_path(stream, split + 1, end), header


def resolved_mentions(model, module, mentions):
    result = set()
    for segments, _start, _end in mentions:
        found = lookup(model, module, segments)
        if found is None or found[0] != 'item':
            continue
        target, name = found[1]
        scope = model.surface.scopes.get(target)
        kind = scope.kinds.get(name) if scope is not None else None
        result.add((target, name, kind))
    return sorted(result, key=lambda m: (m[0], m[1], m[2] or ''))


def units(model):
    """{type key: Unit} for every production type of the crate, with all its impls."""
    found = {}
    for declaration in declarations(model, (), ('struct', 'enum', 'union', 'type', 'trait')):
        unit = Unit(declaration.key, declaration.keyword, declaration)
        unit.declaration_mentions = resolved_mentions(model, declaration.module, declaration.mentions)
        found[declaration.key] = unit
    orphans = []
    for names in sorted(model.views):
        node = model.index[names]
        text = node.text
        for attributes, _visibility, keyword, _name, start, end in scope_items(text):
            if keyword != 'impl' or test_gated(attributes):
                continue
            body = text[start:end]
            generics, trait_path, self_path, header = header_parts(body)
            trait = None
            if trait_path:
                trait = lookup(model, names, trait_path) or ('external', '::'.join(trait_path))
                if trait[0] == 'module':
                    trait = ('external', '::'.join(trait_path))
            self_generic = bool(self_path) and self_path[0] in generics and len(self_path) == 1
            self_key = None
            if self_path and not self_generic:
                target = lookup(model, names, self_path)
                if target is not None and target[0] == 'item':
                    self_key = target[1]
            impl = Impl(names, ' '.join(header.split())[:160], self_key, self_generic, trait)
            mentions = mention_paths(body, '', frozenset(generics))
            mentions = function_mentions(model, names, body, '', frozenset(generics), mentions)
            impl.mentions = resolved_mentions(model, names, mentions)
            impl.lines = body.count('\n') + 1
            owner = attach(impl, found)
            if owner is None:
                orphans.append(impl)
            else:
                owner.impls.append(impl)
    return found, orphans


def attach(impl, found):
    """The unit an impl belongs with by the crate-split rule (type first, else its trait)."""
    trait_key = impl.trait[1] if impl.trait and impl.trait[0] == 'item' else None
    if impl.self_key in found:
        return found[impl.self_key]
    if trait_key in found:
        return found[trait_key]
    return None


def node_of(module, parent):
    """The parent's top-level child holding `module`, or None outside the parent's children."""
    if module[:len(parent)] != parent or len(module) == len(parent):
        return None
    return module[len(parent)]


class Classifier:
    """Each unit referenced against a stack, classified at its destination."""

    def __init__(self, model, parent, order, all_units):
        self.model, self.parent = model, parent
        self.position = {name: i for i, name in enumerate(order)}
        self.units = all_units
        self._memo = {}
        self.roots = {}     # (key, destination) -> non-type items above, the coupling's roots

    def dependencies(self, unit, destination):
        """(impl header or 'declaration', (node, item label, kind)) the unit needs above its
        destination; a crate trait's impl that may stay with its higher trait is left out."""
        limit = self.position[destination]
        own = unit.key
        result = []
        sources = [('declaration', unit.declaration.module, getattr(unit, 'declaration_mentions', []))]
        for impl in unit.impls:
            trait_key = impl.trait[1] if impl.trait and impl.trait[0] == 'item' else None
            if trait_key is not None and trait_key != own and impl.self_key == own:
                trait_node = node_of(trait_key[0], self.parent)
                if trait_node is not None and self.position.get(trait_node, -1) > limit:
                    continue
            sources.append((impl.header, impl.module, impl.mentions))
        for label, _module, mentions in sources:
            for target, name, kind in mentions:
                if (target, name) == own:
                    continue
                node = node_of(target, self.parent)
                if node is None or self.position.get(node, -1) <= limit:
                    continue
                result.append((label, node, '::'.join(target + (name,)), kind, (target, name)))
        return result

    def judge(self, key, destination, active=frozenset()):
        """('movable', co-moved keys) / ('behaviour-coupled', [(impl, node, item, kind)]) /
        ('pinned', reason) for moving `key` whole to `destination`."""
        memo = (key, destination)
        if memo in self._memo:
            return self._memo[memo]
        unit = self.units.get(key)
        if unit is None or unit.declaration is None:
            result = ('pinned', 'no declaration in the crate (macro-made or external)')
            self._memo[memo] = result
            return result
        if key in active:
            return ('movable', set())
        active = active | {key}
        coupled, carried, roots = [], set(), set()
        for label, node, item, kind, target in self.dependencies(unit, destination):
            if kind in TYPE_KINDS:
                verdict = self.judge(target, destination, active)
                if verdict[0] == 'movable':
                    carried |= {target} | verdict[1]
                    continue
                coupled.append((label, node, item, kind + ' (' + verdict[0] + ')'))
                roots |= self.roots.get((target, destination), set())
            else:
                coupled.append((label, node, item, kind or 'unresolved'))
                roots.add(item)
        if coupled:
            self.roots[memo] = roots
            result = ('behaviour-coupled', coupled)
        else:
            result = ('movable', carried)
        self._memo[memo] = result
        return result


def stack_order(stack):
    """Bottom-to-top module order and {module: layer name} from a stack file's layers."""
    order, layer_of = [], {}
    for layer in stack['layers']:
        for module, _purpose in layer['modules']:
            order.append(module)
            layer_of[module] = layer['name']
    return order, layer_of


def against_stack(edges, kinds, order):
    """{item key: {'kind', 'refs', 'from': {module: refs}, 'owner'}} for items referenced
    from a module below the one defining them."""
    position = {name: i for i, name in enumerate(order)}
    result = {}
    for (low, high), refs in edges.items():
        if position[low] >= position[high]:
            continue
        for _source, key in refs:
            entry = result.setdefault(key, {'kind': kinds[key], 'refs': 0, 'from': defaultdict(int),
                                            'owner': high})
            entry['refs'] += 1
            entry['from'][low] += 1
    return result


def stray_inherent_impls(model, parent):
    """Inherent impls of a crate type that sit in another top-level child of `parent` than the
    type itself: [(impl module, type key, header)]. A crate split refuses them, so each belongs
    with its type, or becomes an extension trait owned where it sits, or a free function."""
    found = []
    for names in sorted(model.views):
        impl_node = node_of(names, parent)
        if impl_node is None:
            continue
        text = model.index[names].text
        for attributes, _visibility, keyword, _name, start, end in scope_items(text):
            if keyword != 'impl' or test_gated(attributes):
                continue
            generics, trait_path, self_path, header = header_parts(text[start:end])
            if trait_path or not self_path or (self_path[0] in generics and len(self_path) == 1):
                continue
            target = lookup(model, names, self_path)
            if target is None or target[0] != 'item':
                continue
            type_node = node_of(target[1][0], parent)
            if type_node is not None and type_node != impl_node:
                found.append((names, target[1], ' '.join(header.split())[:120]))
    return found

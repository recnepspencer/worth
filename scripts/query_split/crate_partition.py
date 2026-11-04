"""A candidate partition of one library's module nodes into crates, and its crate graph.

Candidate JSON: {"name", "description", "default": <crate>, "crates": {<crate>: [<module path>...]}}.
A node joins the crate that lists its longest module-path prefix; the rest join `default`.
An optional "hypothetical_order" (crate names, top to bottom) drops every edge from a lower
crate to a higher one and counts its references: the score a broken cycle would earn.
"""
from pathlib import PurePosixPath

from coupling_graph import components


class Partition:
    def __init__(self, candidate, nodes, edges, lines):
        self.default = candidate['default']
        listed = {}
        for crate, modules in candidate['crates'].items():
            for module in modules:
                listed[tuple(m for m in module.split('::') if m)] = crate
        self.listed = listed
        self.crate_of = {node: self.assign(node) for node in nodes}
        self.names = sorted(set(self.crate_of.values()) | {self.default})
        self.lines = {name: 0 for name in self.names}
        for node, count in lines.items():
            self.lines[self.crate_of[node]] += count
        self.depends = {name: set() for name in self.names}
        order = {name: i for i, name in enumerate(candidate.get('hypothetical_order', []))}
        self.hypothetical = bool(order)
        self.broken_references = 0
        for (a, b), refs in edges.items():
            source, target = self.crate_of[a], self.crate_of[b]
            if not refs or source == target:
                continue
            if order and order[source] > order[target]:
                self.broken_references += len(refs)
                continue
            self.depends[source].add(target)
        weights = {(a, b): 1 for a in self.names for b in self.depends[a]}
        self.cycles = [group for group in components(self.names, weights) if len(group) > 1]

    def assign(self, node):
        best = None
        for module, crate in self.listed.items():
            if node[:len(module)] == module and (best is None or len(module) > len(best[0])):
                best = (module, crate)
        return best[1] if best else self.default

    def reverse_closure(self, touched):
        """Touched crates and every crate that depends on one of them, transitively."""
        result, frontier = set(touched), list(touched)
        while frontier:
            crate = frontier.pop()
            for other in self.names:
                if crate in self.depends[other] and other not in result:
                    result.add(other)
                    frontier.append(other)
        return result

    def crates_for_file(self, path, mapping, crate_root):
        """Crates a changed path recompiles; a manifest or config file touches every crate."""
        if not path.startswith(crate_root + '/'):
            return set()
        if not path.endswith('.rs'):
            return set(self.names)
        names = mapping.get(path)
        if names is None:
            names = path_module(path, crate_root)
            if names is None:
                return None
        return {self.assign(owner_node(names, self.crate_of))}

    def crates_for_unknown(self):
        return set(self.names)

    def summary(self):
        rows = []
        for name in self.names:
            dependents = len(self.reverse_closure({name})) - 1
            rows.append({'name': name, 'lines': self.lines[name], 'dependents': dependents,
                         'fan_in_size': dependents * self.lines[name],
                         'depends_on': sorted(self.depends[name])})
        return rows

    def parallel_width(self):
        """Most crates on one longest-path level: crates that can compile at the same time."""
        if self.cycles:
            return 0
        level = {}

        def depth(name):
            if name not in level:
                level[name] = 1 + max((depth(d) for d in self.depends[name]), default=0)
            return level[name]
        counts = {}
        for name in self.names:
            counts[depth(name)] = counts.get(depth(name), 0) + 1
        return max(counts.values())


def owner_node(names, crate_of):
    best = ()
    for node in crate_of:
        if names[:len(node)] == node and len(node) >= len(best):
            best = node
    return best


def path_module(path, crate_root):
    """Module path implied by a file's location under `src/`, for files no longer in the tree."""
    relative = PurePosixPath(path).relative_to(crate_root)
    if relative.parts[:1] != ('src',):
        return None
    parts = list(relative.parts[1:])
    if parts == ['lib.rs']:
        return ()
    parts[-1] = parts[-1][:-3]
    if parts[-1] == 'mod':
        parts.pop()
    return tuple(parts)


def file_modules(model, crate_root):
    """Each library source file mapped to the production module that owns it."""
    result = {}
    for names, node in model.index.items():
        if node.declaration is not None and node.declaration.body is not None:
            continue
        owner = names
        while model.surface.in_family(owner):
            owner = owner[:-1]
        result[model.relative(node.source)] = owner
    return result

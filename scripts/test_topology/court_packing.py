"""First-fit decreasing of whole module areas, descending only oversized areas."""
from collections import defaultdict


def assign_file_weights(units):
    from support_readers import has_meaning
    consumers, compiled, programs = defaultdict(list), set(), defaultdict(list)
    for unit in units:
        record = unit['scope']
        unit['packing_lines'] = len(record['test'].splitlines()) + len(record['attributes'].splitlines()) + 12
        compiled.add(record['node'].source)
        for source in unit.get('support_inputs', ()):
            compiled.add(source['node'].source)
            if has_meaning(source):
                consumers[id(source)].append((source, unit))
    for readers in consumers.values():
        record, unit = min(readers, key=lambda pair: (pair[1]['owner_area'], pair[1]['key']))
        unit['packing_lines'] += len(record['helper'].splitlines()) + len(record['attributes'].splitlines()) + 12
    for unit in units:
        for path in unit['footprint'] - compiled:
            if path.suffix == '.rs':
                programs[path].append(unit)
    for path, readers in programs.items():
        unit = min(readers, key=lambda u: (u['owner_area'], u['key']))
        unit['packing_lines'] += len(path.read_bytes().splitlines())


def whole_areas(population, ceiling):
    result = []

    def descend(members, prefix):
        size = sum(u['packing_lines'] for u in members)
        if size <= ceiling:
            result.append({'path': prefix, 'lines': size, 'units': members})
            return
        groups = defaultdict(list)
        for unit in members:
            path = tuple(unit['owner_area'].split('/'))
            key = path[:len(prefix) + 1] if len(path) > len(prefix) else path
            groups[key].append(unit)
        if len(groups) == 1 and next(iter(groups)) == prefix:
            raise ValueError('indivisible area exceeds ceiling: ' + '/'.join(prefix))
        for key, group in sorted(groups.items()):
            descend(group, key)

    origins = defaultdict(list)
    for unit in population:
        origins[(unit['owner_area'].split('/')[0],)].append(unit)
    for prefix, members in sorted(origins.items()):
        descend(members, prefix)
    return result


def first_fit_decreasing(areas, ceiling):
    pieces = []
    for area in sorted(areas, key=lambda a: (-a['lines'], a['path'])):
        piece = next((p for p in pieces if p['lines'] + area['lines'] <= ceiling), None)
        if piece is None:
            piece = {'lines': 0, 'areas': []}
            pieces.append(piece)
        piece['areas'].append(area)
        piece['lines'] += area['lines']
    return pieces


def area_suffix(path, surface, frontier_stems=()):
    parts = []
    stems = sorted({surface, *frontier_stems}, key=lambda s: (-len(s), s))
    for part in path:
        part = part.replace('_', '-')
        for stem in stems:
            if part == stem:
                part = ''
                break
            if part.startswith(stem + '-'):
                part = part[len(stem) + 1:]
                break
        if part:
            parts.append(part)
    return '-'.join(parts) or 'entry'

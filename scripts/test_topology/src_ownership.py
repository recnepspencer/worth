"""Which moving module scopes become testkit support, court tests, or both.

A module scope's disposition:
- `court`: it holds tests and nothing outside its court unit reads it;
- `testkit`: it holds no tests and something outside its unit reads it (or a testkit part does);
- `split`: it holds tests and is read from outside its unit; its non-test items become the
  testkit part and its tests stay in the court part.
The testkit never depends on a court, so every moving scope a testkit part reads gains a
testkit part too, to a fixpoint. Scopes with no tests that nothing outside reads stay with
their unit's court as local support.
"""


def dispositions(model, unit_of):
    """Map module names -> 'court' | 'testkit' | 'split' | 'local'."""
    supported = set()
    for names, record in model.records.items():
        reader_unit = unit_of(names)
        for _reference, origin in record.uses:
            if origin is None or origin[0] == '<external>':
                continue
            target = defining_record(model, origin[0])
            if target is None or target == names:
                continue
            if unit_of(target) != reader_unit:
                supported.add(target)
    supported |= {names for names in model.records if model.records[names].family in root_support(model)}
    supported |= {names for names, record in model.records.items() if record.extracted or record.doors}
    frontier = list(supported)
    while frontier:
        names = frontier.pop()
        for _reference, origin in model.records[names].uses:
            if origin is None or origin[0] == '<external>':
                continue
            target = defining_record(model, origin[0])
            if target is not None and target != names and target not in supported:
                supported.add(target)
                frontier.append(target)
        for ancestor in ancestors_in_family(model, names):
            if ancestor not in supported and not model.records[ancestor].tests:
                # A support scope's non-test ancestors carry its module path; they hold no tests.
                pass
    result = {}
    for names, record in model.records.items():
        has_tests = bool(record.tests) or bool(macro_tests(record))
        if names in supported:
            result[names] = 'split' if has_tests else 'testkit'
        else:
            result[names] = 'court' if has_tests else 'local'
    return result


def defining_record(model, module):
    """The moving module scope that defines `module`'s items (itself when it is moving)."""
    return module if module in model.records else None


def root_support(model):
    """Moving families with no tests at all are support wherever they are read from."""
    return {family for family in model.moving if family in model.families
            if not any(model.records[n.names].tests or macro_tests(model.records[n.names])
                       for n in model.families[family].node.walk())}


def ancestors_in_family(model, names):
    family = model.records[names].family
    return [names[:i] for i in range(len(family), len(names))]


def macro_tests(record):
    from src_families import macro_generated
    return macro_generated(record.node.own_text())


def sizes(model, result):
    """Lines that land in the testkit and in courts, by disposition."""
    totals = {'testkit': 0, 'court': 0}
    for names, disposition in result.items():
        text = model.records[names].node.own_text()
        lines = text.count('\n') + 1
        if disposition == 'testkit':
            totals['testkit'] += lines
        elif disposition in ('court', 'local'):
            totals['court'] += lines
        else:
            record = model.records[names]
            test_lines = sum(text.count('\n', i.start, i.end) + 1 for i in record.items if i.test)
            totals['court'] += test_lines
            totals['testkit'] += max(lines - test_lines, 0)
    return totals

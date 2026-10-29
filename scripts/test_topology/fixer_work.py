"""Record source-local unresolved semantics for disjoint court fixer ownership."""
from collections import defaultdict
import re

from module_tree import private_context, test_names
from rust_source import code
from test_macros import opaque_items


def collect(courts, assignments, repository):
    result = {court['name']: [] for court in courts}
    seen = defaultdict(set)

    def add(court, kind, source, reason, symbols=()):
        row = {'kind': kind, 'source': source.relative_to(repository).as_posix(),
               'reason': reason, 'symbols': sorted(set(symbols))}
        key = (kind, row['source'], reason, tuple(row['symbols']))
        if key not in seen[court]:
            result[court].append(row)
            seen[court].add(key)

    for court in courts:
        for unit in court['units']:
            node = unit['node']
            for question in unit['questions']:
                add(court['name'], 'classification', node.source, question)
            nodes = list(node.walk()) + [n for support in unit['support'] for n in support.walk()]
            for source in nodes:
                text, context = source.own_text(), private_context(source)
                value = code(text)
                assignment = assignments.get(source.source)
                if assignment and '-testkit' in assignment['owner'] and context['restricted_visibility']:
                    symbols = re.findall(r'\bpub\s*\(\s*(?:crate|super)\s*\)\s+(?:async\s+)?(?:fn|struct|enum|trait|type|const|static|mod|use)\s+(\w+)', text)
                    add(court['name'], 'testkit-visibility', source.source,
                        'Shared support keeps its original restricted visibility; grant only the necessary testkit facade exports.', symbols)
                if re.search(r'\bmacro_rules\s*!|\$\s*crate\b', value):
                    add(court['name'], 'macro-context', source.source,
                        'Check macro expansion against its new crate/module owner; no expansion was guessed.')
                macro_names = opaque_items(text)
                if macro_names:
                    add(court['name'], 'macro-expansion', source.source,
                        'Opaque item macros remain mounted as test code. Check expansion assumptions at the new owner.', macro_names)
                if re.search(r'env!\s*\(\s*"CARGO_PKG_NAME"|module_path!\s*\(', text):
                    add(court['name'], 'identity-assertion', source.source,
                        'Package/module identity changed; review assertions and protocols using these macros.')
                if 'CARGO_BIN_EXE_' in text:
                    add(court['name'], 'executable-location', source.source,
                        'Product executable env belongs to its original package; supply the actual built executable path.')
                if 'CARGO_MANIFEST_DIR' in text:
                    add(court['name'], 'manifest-context', source.source,
                        'Literal tests/ paths are rewritten; inspect dynamic paths and nested Cargo subprocesses for old-package assumptions.')
                if re.search(r'\bfeature\s*=\s*"', text):
                    add(court['name'], 'feature-configuration', source.source,
                        'cfg(feature) belongs to the old product package. Courts declare no features; replace this conditional through the owned control/observer contract before certification.')
                if any(name in context['crate_paths'] for name in ('fast', 'exhaustive', 'scale', 'serial', 'compile_fail')):
                    add(court['name'], 'cross-category-support', source.source,
                        'Module paths are rebased. Shared helpers exported by another category may require extraction to a testkit; courts cannot depend on each other.', context['crate_paths'])
                names = [name for name in test_names(text) if re.search(
                    r'(?:^|_)(?:(?:m|d|r|phase|milestone|decision|ticket|req)\d+|(?:phase|milestone|decision|ticket)_(?:\d+|one|two|three|four|five|six|seven|eight|nine|ten))(?:_|$)', name)
                         or name.startswith(('test_', 'it_'))]
                if names:
                    add(court['name'], 'behaviour-name', source.source,
                        'v3 behaviour names need an explicit traceability-preserving rename; codemod keeps all test names.', names)
                if assignment and '-testkit' in assignment['owner'] and context['crate_paths']:
                    add(court['name'], 'testkit-crate-context', source.source,
                        'Crate-relative support imports must refer to the owning testkit facade.', context['crate_paths'])
    return {name: sorted(rows, key=lambda r: (r['kind'], r['source'], r['reason']))
            for name, rows in sorted(result.items())}

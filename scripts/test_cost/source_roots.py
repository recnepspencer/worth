"""Resolve Cargo-declared and autodiscovered Rust target source roots."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'test_topology'))
from filesystem_path import filesystem_path


FAMILIES = (('bin', 'src/bin', 'autobins'), ('test', 'tests', 'autotests'),
            ('bench', 'benches', 'autobenches'), ('example', 'examples', 'autoexamples'))


def discovered(root, folder):
    directory = root / folder
    io_directory = filesystem_path(directory)
    paths = sorted(io_directory.glob('*.rs')) + sorted(io_directory.glob('*/main.rs'))
    return [(path.stem if len(path.relative_to(io_directory).parts) == 1 else path.parent.name,
             directory / path.relative_to(io_directory)) for path in paths]


def target_source(root, family, folder, name, target, package_name):
    if 'path' in target:
        candidates = [root / target['path']]
    elif family == 'lib':
        candidates = [root / 'src/lib.rs']
    elif family == 'bin' and name == package_name and filesystem_path(root / 'src/main.rs').is_file():
        candidates = [root / 'src/main.rs']
    else:
        candidates = [root / folder / (name + '.rs'), root / folder / name / 'main.rs']
    existing = [path.resolve() for path in candidates if filesystem_path(path).is_file()]
    if len(existing) != 1:
        raise ValueError(f'declared {family} has no unique source: {name}: {candidates}')
    return existing[0]


def roots(root, declaration, inherited_edition=None):
    root = Path(root).resolve()
    package = declaration['package']
    edition = package.get('edition', '2015')
    if isinstance(edition, dict):
        if inherited_edition is None:
            raise ValueError('workspace-inherited package edition is unavailable')
        edition = inherited_edition
    result = []
    library = declaration.get('lib', {})
    if 'lib' in declaration or (package.get('autolib', True) and filesystem_path(root / 'src/lib.rs').is_file()):
        name = library.get('name', package['name'].replace('-', '_'))
        result.append(dict(family='lib', name=name,
                           source=target_source(root, 'lib', 'src', name, library, package['name'])))
    for family, folder, automatic in FAMILIES:
        explicit = declaration.get(family, [])
        targets, owned_paths = {}, set()
        for target in explicit:
            name = target.get('name')
            if not isinstance(name, str) or not name:
                raise ValueError(f'declared {family} requires a target name')
            if name in targets:
                raise ValueError(f'duplicate declared {family} target: {name}')
            source = target_source(root, family, folder, name, target, package['name'])
            targets[name] = source
            owned_paths.add(source)
        if package.get(automatic, edition != '2015' or not explicit):
            candidates = discovered(root, folder)
            if family == 'bin' and filesystem_path(root / 'src/main.rs').is_file():
                candidates.insert(0, (package['name'], root / 'src/main.rs'))
            explicit_names, automatic_names = set(targets), set()
            for name, source in candidates:
                source = source.resolve()
                if name in explicit_names or source in owned_paths:
                    continue
                if name in automatic_names:
                    raise ValueError(f'duplicate autodiscovered {family} target: {name}')
                automatic_names.add(name)
                targets[name] = source
        result.extend(dict(family=family, name=name, source=source) for name, source in sorted(targets.items()))
    return result

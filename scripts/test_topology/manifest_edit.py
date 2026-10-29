"""Render Cargo declarations and edit named TOML tables without third-party writers."""
import json
import os
from pathlib import Path
import re
import tomllib


def quote(value):
    return json.dumps(str(value), ensure_ascii=False)


def toml_value(value):
    if isinstance(value, str):
        return quote(value)
    if isinstance(value, bool):
        return str(value).lower()
    if isinstance(value, list):
        return '[' + ', '.join(toml_value(v) for v in value) + ']'
    if isinstance(value, dict):
        return '{ ' + ', '.join(quote(k) + ' = ' + toml_value(v) for k, v in value.items()) + ' }'
    if isinstance(value, (int, float)):
        return str(value)
    raise ValueError('unsupported TOML value: ' + repr(value))


def tables(text):
    headers = list(re.finditer(r'^\s*(\[\[?[^\n]+?\]\]?)\s*(?:#.*)?$', text, re.M))
    return [(match[1], match.start(), headers[i + 1].start() if i + 1 < len(headers) else len(text))
            for i, match in enumerate(headers)]


def set_table(text, header, values):
    rendered = header + '\n' + ''.join(quote(k) + ' = ' + toml_value(v) + '\n' for k, v in values.items()) + '\n'
    found = next(((start, end) for name, start, end in tables(text) if name == header), None)
    if found:
        start, end = found
        return text[:start] + rendered + text[end:]
    return text.rstrip() + '\n\n' + rendered


def product_manifest(text, drops, no_unit_tests):
    value = tomllib.loads(text)
    for header, start, end in reversed(tables(text)):
        if header == '[[test]]':
            text = text[:start] + text[end:]
        elif header.endswith('dev-dependencies]'):
            body = text[start:end]
            parsed = tomllib.loads(body)
            table = parsed
            # Descend only through header containers, never into a dependency spec.
            while 'dev-dependencies' not in table:
                children = [v for v in table.values() if isinstance(v, dict)]
                if len(children) != 1:
                    raise ValueError('ambiguous dev-dependency table: ' + header)
                table = children[0]
            table = table['dev-dependencies']
            remaining = {k: v for k, v in table.items() if k not in drops}
            replacement = header + '\n' + ''.join(quote(k) + ' = ' + toml_value(v) + '\n'
                                                  for k, v in remaining.items()) + '\n' if remaining else ''
            text = text[:start] + replacement + text[end:]
    package = dict(value['package'])
    for header, _start, _end in tables(text):
        if header.startswith('[package.'):
            child = header[len('[package.'):].split('.', 1)[0].rstrip(']')
            package.pop(child, None)
    package['autotests'] = False
    text = set_table(text, '[package]', package)
    if no_unit_tests:
        lib = dict(value.get('lib', {}), test=False)
        text = set_table(text, '[lib]', lib)
    return text


def workspace_manifest(text, paths):
    value = tomllib.loads(text)
    workspace = value['workspace']
    workspace['members'] = sorted(set(workspace.get('members', [])) | set(paths))
    # Only rewrite the workspace header table, preserving child tables and profiles.
    return set_table(text, '[workspace]', {k: v for k, v in workspace.items()
                                         if k not in ('package', 'dependencies', 'lints', 'metadata')})


def dependencies(rows, destination):
    grouped = {}
    for row in rows:
        key = (row.get('target'), row['alias'])
        if key not in grouped:
            grouped[key] = dict(row, features=list(row.get('features', [])))
            continue
        old = grouped[key]
        if (old.get('path'), old.get('requirement'), old['package']) != (row.get('path'), row.get('requirement'), row['package']):
            raise ValueError('dependency alias collision: ' + row['alias'])
        old['features'] = sorted(set(old['features']) | set(row.get('features', [])))
        old['default_features'] |= row['default_features']
    sections = {}
    for (target, alias), row in sorted(grouped.items(), key=lambda item: str(item[0])):
        spec = {}
        if row.get('path'):
            spec['path'] = Path(os.path.relpath(Path(row['path']).resolve(), destination.resolve())).as_posix()
        elif (row.get('source') or '').startswith('git+'):
            source = row['source'][4:].split('#', 1)
            spec['git'] = source[0].split('?', 1)[0]
            if len(source) > 1:
                spec['rev'] = source[1]
        else:
            spec['version'] = row['requirement']
        if alias != row['package']:
            spec['package'] = row['package']
        if row.get('features'):
            spec['features'] = sorted(row['features'])
        if not row.get('default_features', True):
            spec['default-features'] = False
        header = '[dependencies]' if not target else '[target.' + quote(target) + '.dependencies]'
        sections.setdefault(header, []).append(quote(alias) + ' = ' + toml_value(spec))
    return '\n'.join(header + '\n' + '\n'.join(lines) + '\n' for header, lines in sorted(sections.items()))


def crate_manifest(name, rows, destination, workspace_value, testkit=False):
    inherited = workspace_value.get('workspace', {}).get('package', {})
    fields = {'name': name, 'autotests': False, 'publish': False}
    for key, default in [('edition', '2024'), ('rust-version', '1.94'), ('version', '0.1.0'), ('license', 'BUSL-1.1')]:
        fields[key] = {'workspace': True} if key in inherited else default
    text = '[package]\n' + ''.join(quote(k) + ' = ' + toml_value(v) + '\n' for k, v in fields.items()) + '\n'
    text += dependencies(rows, destination)
    if testkit:
        text += '\n[lib]\ntest = false\ndoctest = false\n'
    else:
        snake = name.replace('-', '_')
        text += '\n[[test]]\nname = ' + quote(snake) + '\npath = ' + quote('tests/' + snake + '.rs') + '\n'
    if 'lints' in workspace_value.get('workspace', {}):
        text += '\n[lints]\nworkspace = true\n'
    return text

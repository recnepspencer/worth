"""A reviewable, preflighted move/write program with idempotent replay."""
import hashlib
from pathlib import Path
import subprocess
from filesystem_path import filesystem_path, command_path


def digest(data):
    return hashlib.sha256(data).hexdigest()


def relative(path, root):
    path = path.resolve()
    if not path.is_relative_to(root.resolve()):
        raise ValueError('path outside repository: ' + str(path))
    return path.relative_to(root.resolve()).as_posix()


class Effects:
    def __init__(self, root):
        self.root = root.resolve()
        self.moves = []
        self.writes = {}
        self.removals = []

    def move(self, source, destination):
        if source.resolve() == destination.resolve():
            return
        self.moves.append({'source': relative(source, self.root),
                           'destination': relative(destination, self.root),
                           'sha256': digest(source.read_bytes())})

    def write(self, path, content, before=None):
        name = relative(path, self.root)
        if name in self.writes and self.writes[name]['content'] != content:
            raise ValueError('conflicting generated writes: ' + name)
        if before is None:
            before = path.read_bytes() if path.exists() else b''
        self.writes[name] = {'path': name, 'before_sha256': digest(before), 'content': content}

    def retire_scaffold(self, path):
        self.removals.append({'path': relative(path, self.root), 'sha256': digest(path.read_bytes()),
                              'reason': 'v3 replaces this module-only aggregation with generated court roots; it owns no test bodies or helper definitions.'})

    def program(self):
        for move in self.moves:
            write = self.writes.get(move['destination'])
            if write and write['before_sha256'] == digest(b''):
                write['before_sha256'] = move['sha256']
        return {'moves': sorted(self.moves, key=lambda r: r['source']),
                'writes': [self.writes[k] for k in sorted(self.writes)],
                'retired_scaffolds': sorted(self.removals, key=lambda r: r['path'])}


def contained(root, name):
    path = (root / name).resolve()
    if Path(name).is_absolute() or not path.is_relative_to(root):
        raise ValueError('unsafe plan path: ' + name)
    return path


def move_groups(root, moves):
    """Use maximal unchanged subtrees, without absorbing unplanned files."""
    tracked = set(subprocess.check_output(['git', '-c', 'core.longpaths=true', 'ls-files', '-z'], cwd=command_path(root)).decode('utf-8').split('\0'))
    mapping = dict(moves)
    candidates = {}
    for source, destination in moves:
        for directory in source.parents:
            if directory == root:
                break
            candidates.setdefault(directory, set()).add(source)
            if directory.name == 'tests':
                break
    covered, result = set(), []
    for directory, sources in sorted(candidates.items(), key=lambda item: (len(item[0].parts), str(item[0]))):
        if sources & covered or len(sources) < 2:
            continue
        first = min(sources)
        destination = mapping[first]
        for _part in first.relative_to(directory).parts:
            destination = destination.parent
        if destination.exists() or any(mapping[p] != destination / p.relative_to(directory) for p in sources):
            continue
        if any(relative(p, root) not in tracked for p in sources):
            continue
        actual = {p.resolve() for p in directory.rglob('*') if p.is_file()}
        if actual != sources:
            continue
        result.append((directory, destination, True))
        covered.update(sources)
    result.extend((source, destination, relative(source, root) in tracked)
                  for source, destination in moves if source not in covered)
    return sorted(result, key=lambda row: str(row[0]))


def apply(plan, root):
    root = filesystem_path(root.resolve())
    program = plan['effects']
    # Validate every byte and destination before taking the first effect. Replays
    # accept only the original or final bytes, never a silently changed third tree.
    moves, writes = [], []
    removals = []
    for row in program.get('retired_scaffolds', []):
        path = contained(root, row['path'])
        if path.exists():
            if digest(path.read_bytes()) != row['sha256']:
                raise ValueError('scaffold precondition changed: ' + row['path'])
            removals.append(path)
    for move in program['moves']:
        source, destination = (contained(root, move[k]) for k in ('source', 'destination'))
        if source.exists():
            if digest(source.read_bytes()) != move['sha256'] or destination.exists():
                raise ValueError('move precondition changed: ' + move['source'])
            moves.append((source, destination))
        elif not destination.exists():
            raise ValueError('missing move source and destination: ' + move['source'])
        elif not any(w['path'] == move['destination'] for w in program['writes']):
            if digest(destination.read_bytes()) != move['sha256']:
                raise ValueError('applied move changed: ' + move['destination'])
    moved = {m['destination']: m['sha256'] for m in program['moves']}
    for write in program['writes']:
        path = contained(root, write['path'])
        final = write['content'].encode('utf-8')
        before = path.read_bytes() if path.exists() else b''
        if before == final:
            continue
        expected = write['before_sha256']
        if digest(before) != expected and not (not path.exists() and moved.get(write['path']) == expected):
            raise ValueError('write precondition changed: ' + write['path'])
        writes.append((path, final))
    groups = move_groups(root, moves) if moves else []
    for source, destination, tracked in groups:
        destination.parent.mkdir(parents=True, exist_ok=True)
        if tracked:
            subprocess.run(['git', '-c', 'core.longpaths=true', 'mv', '--', relative(source, root), relative(destination, root)],
                           cwd=command_path(root), check=True)
        else:
            source.rename(destination)
    for path, content in writes:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    for path in removals:
        tracked = subprocess.run(['git', 'ls-files', '--error-unmatch', '--', relative(path, root)],
                                 cwd=command_path(root), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
        if tracked:
            subprocess.run(['git', '-c', 'core.longpaths=true', 'rm', '-f', '--', relative(path, root)], cwd=command_path(root), check=True,
                           stdout=subprocess.DEVNULL)
        else:
            path.unlink()
    # Remove only empty source parents; assets and untracked files are never erased.
    for source in [source for source, _destination in moves] + removals:
        parent = source.parent
        while parent != root:
            if not parent.exists():
                parent = parent.parent
                continue
            try:
                parent.rmdir()
            except OSError:
                break
            parent = parent.parent
    return {'moved': len(moves), 'written': len(writes)}

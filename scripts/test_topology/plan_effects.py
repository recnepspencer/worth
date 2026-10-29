"""A reviewable, preflighted move/write program with idempotent replay."""
import hashlib
from pathlib import Path
import subprocess


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

    def program(self):
        return {'moves': sorted(self.moves, key=lambda r: r['source']),
                'writes': [self.writes[k] for k in sorted(self.writes)]}


def contained(root, name):
    path = (root / name).resolve()
    if Path(name).is_absolute() or not path.is_relative_to(root):
        raise ValueError('unsafe plan path: ' + name)
    return path


def apply(plan, root):
    root = root.resolve()
    program = plan['effects']
    # Validate every byte and destination before taking the first effect. Replays
    # accept only the original or final bytes, never a silently changed third tree.
    moves, writes = [], []
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
    for source, destination in moves:
        destination.parent.mkdir(parents=True, exist_ok=True)
        tracked = subprocess.run(['git', 'ls-files', '--error-unmatch', '--', relative(source, root)],
                                 cwd=root, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode == 0
        if tracked:
            subprocess.run(['git', 'mv', '--', relative(source, root), relative(destination, root)],
                           cwd=root, check=True)
        else:
            source.rename(destination)
    for path, content in writes:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    # Remove only empty source parents; assets and untracked files are never erased.
    for source, _destination in moves:
        parent = source.parent
        while parent != root:
            try:
                parent.rmdir()
            except OSError:
                break
            parent = parent.parent
    return {'moved': len(moves), 'written': len(writes)}

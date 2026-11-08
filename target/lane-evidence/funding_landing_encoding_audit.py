from pathlib import Path
import subprocess,json,hashlib
p=Path('target/lane-evidence/foundation-clone-quote-final-independent-review.md')
old=subprocess.check_output(['git','show','5a49620b1d:'+p.as_posix()])
assert p.read_bytes()==old.replace(b'\x97',bytes.fromhex('e28094'))
for name in subprocess.check_output(['git','ls-files','-z','--','target/lane-evidence'],text=True).split('\0'):
    if name and Path(name).is_file(): Path(name).read_bytes().decode('utf-8')
snapshot=json.loads(Path('target/lane-evidence/funding-landing-source-state.json').read_text())
for path,expected in snapshot['files'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==expected,path
print('All tracked evidence is strict UTF-8; original review differs by the one equivalent dash encoding only.')
print('All61 frozen source/config hashes match.')

"""Cut Store's named executable consumers over to a fallible prebuilt lookup."""
import argparse
from pathlib import Path
import re

from workspace import sources
from filesystem_path import filesystem_path


CONSTANTS = ('OFFLINE_ROOT_OBSERVER', 'OFFLINE_RECOVERY_OBSERVER', 'FRESH_PROCESS_RECOVERY')
CALL = re.compile(r'(\bworth_store_test_support::(?:' + '|'.join(CONSTANTS) + r')\s*\.path\(\))')


def rewrite(text, provided=False):
    text, count = CALL.subn(r'\1.expect("runner must prebuild the independent executable")', text)
    if provided:
        text, extra = re.subn(r'(let path = executable\s*\.path\(\))',
                             r'\1.expect("runner must prebuild the independent executable")', text)
        count += extra
    return text, count


def plans(manifest):
    for source in sources(manifest):
        path, text = source.path, source.text
        provided = path.as_posix().endswith('/support/support_binaries/declarations.rs')
        if '.expect("runner must prebuild' in text:
            continue
        original = filesystem_path(path).read_text(encoding='utf-8')
        changed, count = rewrite(original, provided)
        if count:
            yield path, count, changed


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    for path, count, changed in plans(Path('workspaces/worth-store/Cargo.toml')):
        print(f'{path}: {count}')
        if args.apply:
            filesystem_path(path).write_text(changed, encoding='utf-8')


if __name__ == '__main__':
    main()

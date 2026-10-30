"""Use Windows extended-length paths at filesystem calls without changing identities."""
import os
from pathlib import Path


def filesystem_path(path):
    path = Path(path).absolute()
    if os.name == 'nt' and not str(path).startswith('\\\\?\\'):
        return Path('\\\\?\\' + str(path))
    return path


def command_path(path):
    value = str(path)
    return Path(value[4:]) if value.startswith('\\\\?\\') else Path(value)

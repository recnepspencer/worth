"""Local Cargo dependency declarations for generated courts and testkits."""


def local_dependency(name, path):
    return {'alias': name, 'package': name, 'path': str(path), 'features': [],
            'default_features': True, 'target': None, 'requirement': None}

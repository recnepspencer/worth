"""Explain each original executable's destination and genuine build separation."""


def report(targets, courts):
    result = []
    for target in targets:
        identity = target['package'] + '::' + target['name']
        assigned = [court for court in courts if any(u['target'] == identity for u in court['units'])]
        destinations = []
        for court in assigned:
            same = [c for c in courts if c['frontier'] == court['frontier'] and c['name'] != court['name']]
            reasons = []
            if any(c['required_features'] != court['required_features'] for c in same):
                reasons.append({'kind': 'required-features', 'features': court['required_features']})
            if court['custom_harness']:
                reasons.append({'kind': 'custom-harness', 'target': court['custom_harness']})
            if any(c['required_features'] == court['required_features'] and not c['custom_harness'] for c in same):
                reasons.append({'kind': 'source-ceiling', 'limit': 15000,
                                'source_charge': court['packing_lines']})
            destinations.append({'court': court['name'], 'frontier': court['frontier'],
                                 'separation_reasons': reasons})
        result.append({'package': target['package'], 'target': target['name'],
                       'destinations': destinations})
    return result

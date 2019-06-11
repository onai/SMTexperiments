'''
Generates a set of commitments.
Writes json commitments to disk
'''

import json
import numpy as np
import string
import sys

S_CALL_LEN = 10 # n chars in service call
N_SCALLS_ALLOF = 100 # max # of s_calls in an allof C_{A0}
N_SC=10

COST_CEIL_MAX = 200 # maximum entry in the cost_ceil field for an allof

MAX_N_ALLOFS = 100 # maximum number of allofs to include in the system
MAX_N_ALLOFS_PER_COMMIT = 10 # maximum number of allofs to include in the system C_{C}
MAX_INSTANCES_PER_S_CALL = 10 # maximum instance count per service call 
MAX_N_COMMITS = 1000 #N_{C}


def build_commitments(n_commits, s_calls):
    '''
    Commitments
    '''
    return [build_allofs(s_calls, np.random.randint(1, MAX_N_ALLOFS_PER_COMMIT)) for _ in range(n_commits)]
    
def build_service_calls_list(n):
    '''
    A service call is a just a string representing the regid.
    here we generate `n` unique such strings
    '''
    s_call_regids = set()
    while len(s_call_regids) < n:
        new_str = ''.join(np.random.choice(list(string.ascii_uppercase)) for _ in range(S_CALL_LEN))
        s_call_regids.add(new_str)
    
    return s_call_regids

def build_allofs(service_calls, n):
    '''
    And allof contains a set of (service-call + instance id) pairs and
    a corresponding boolean indicating request or offer
    '''
    return [build_allof(service_calls) for _ in range(n)]
    

def build_allof(service_calls):
    service_calls = list(service_calls)
    n_calls = np.random.randint(1, N_SCALLS_ALLOF)
    included_calls_idx = np.random.choice(range(len(service_calls)), size=n_calls)
    included_calls = [service_calls[i] for i in included_calls_idx]
    is_requests_ints = np.random.binomial(1, 0.75, n_calls)
    instances = []

    is_requests = []
    for i in is_requests_ints:
        if i == 0:
            is_requests.append(True)
        else:
            is_requests.append(False)

    
    for i, call_id in enumerate(included_calls):
        if is_requests[i]:
            # is a request, so we can ask for several ids
            n_instances = np.random.randint(1, MAX_INSTANCES_PER_S_CALL)
            instances.append(range(n_instances))
        else:
            instance = np.random.randint(1, MAX_INSTANCES_PER_S_CALL)
            instances.append([instance])
    

    allof_entries = []

    for i, call_id in enumerate(included_calls):
        call_instance_ids = instances[i]
        request_or_not = is_requests[i]

        for instance_id in call_instance_ids:
            allof_entries.append(
                (
                    str(call_id) + '-' + str(instance_id),
                    request_or_not
                )
            )

    
    cost_ceil = np.random.randint(-COST_CEIL_MAX, COST_CEIL_MAX)

    return {'s_calls': allof_entries, 'cost_ceil': cost_ceil}

if __name__ == '__main__':
    n_allofs = int(sys.argv[1])

    for i in range(n_allofs):
        dest_filename = str(i) + '.json'
        with open(dest_filename, 'w') as handle:
            s_calls = build_service_calls_list(n=N_SC)
            commits = build_commitments(np.random.randint(MAX_N_COMMITS), s_calls)
            #commits = build_commitments(10, s_calls)
            json.dump(commits, handle, indent=4)
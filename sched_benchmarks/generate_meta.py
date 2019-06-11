'''
Accepts a system (collection of allofs etc) as input
produces small metadata of the output
'''

import json
import sys

def print_meta(dataset_filename):
    data = None

    with open(dataset_filename) as handle:
        data = json.load(handle)

    service_calls_counts = {}

    for allof in data:
        s_call_details = allof['s_calls']
        for s_call, is_request in s_call_details:
            service_calls.add(s_call)

    

if __name__ == '__main__':
    dataset_filename = sys.argv[1]

    print_meta(dataset_filename)
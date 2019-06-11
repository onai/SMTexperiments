# Copyright 2018- Onai (Onu Technology, Inc., San Jose, California)
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is furnished
# to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
#  copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
#  INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
# PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
# HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
# OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
# SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
"""
Accepts a system (collection of allofs etc) as input
produces small metadata of the output
"""

import json
import sys


def print_meta(dataset_filename):
    data = None

    with open(dataset_filename) as handle:
        data = json.load(handle)

    service_calls_counts = {}

    for allof in data:
        s_call_details = allof["s_calls"]
        for s_call, is_request in s_call_details:
            service_calls.add(s_call)


if __name__ == "__main__":
    dataset_filename = sys.argv[1]

    print_meta(dataset_filename)

#!/usr/bin/env python3

import sys
import json
import subprocess
from copy import deepcopy

MAX_TRIES = 100

if __name__ == "__main__":
    assert len(sys.argv) == 2
    path = sys.argv[1]
    data = open(path + "/benchmark.json", "r").read()
    data = json.loads(data)
    output = {"name": data["name"]}
    print("Benchmarking", data["name"])
    max_lenght = max(map(len, data["executors"]))
    for test in data["tests"]:
        print("Test(", end='')
        pp = ""
        for i, param in enumerate(data["parameters"]):
            print(param, "=", test[i], sep='', end='')
            pp += str(test[i])
            if(i != len(data["parameters"]) - 1):
                print(", ", end='')
                pp += "|"
        print("):")
        output[pp] = {}
        for executor, commands in data["executors"].items():
            formatted = (" " * (max_lenght - len(executor))) + executor
            cumulative_time = 0.0
            results = []
            execution_command = deepcopy(commands[0])
            execution_command[0] = path + "/" + execution_command[0]
            for i, prefix in enumerate(commands[1]):
                execution_command += prefix
                execution_command += [str(test[i])]
            output[pp][executor] = []
            tries = 0
            while cumulative_time <= data["time_limit"] and tries < MAX_TRIES:
                try:
                    result = subprocess.run(
                        execution_command,
                        timeout=data["time_limit"]*1.25,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.DEVNULL
                    )
                    result_time = float(result.stdout.decode())
                except subprocess.TimeoutExpired:
                    result_time = float('inf')
                cumulative_time += result_time
                results.append(result_time)
                output[pp][executor].append(result_time)
                tries += 1
            avg_time = sum(results) / len(results)
            print(formatted, "=>", "%.6f" % (avg_time,))
    print(json.dumps(output, indent=4), file=sys.stderr)

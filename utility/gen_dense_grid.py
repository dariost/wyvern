#!/usr/bin/env python3

from sys import argv

assert len(argv) == 3

x = int(argv[1])
y = int(argv[2])

assert x > 2
assert y > 2

n = x * y
m = n * 3

print(n, m)

for Y in range(y):
    for X in range(x):
        node = Y * x + X
        print(node, Y * x + ((X + 1) % x))
        print(node, (node + x) % n)
        print(node, ((Y + 1) * x) % n + ((X + 1) % x))

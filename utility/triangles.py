#!/usr/bin/env python3

import sys
import os
from time import time
import numpy as np
from numpy import linalg as la


if __name__ == "__main__":
    assert len(sys.argv) == 2
    fin = open(sys.argv[1], "r")
    n, m = [int(x) for x in fin.readline().strip().split()]
    g = np.zeros([n, n], dtype=np.int32)
    for _ in range(m):
        a, b = [int(x) for x in fin.readline().strip().split()]
        g[a, b] += 1
        g[b, a] += 1
    start = time()
    acc = la.matrix_power(g, 3).trace()
    print("%.9f" % (time() - start,))
    assert acc % 6 == 0
    print("Triangles:", acc // 6, file=sys.stderr)

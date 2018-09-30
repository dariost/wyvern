#include <vector>
#include <iostream>
#include <cassert>
#include <ctime>
#include <cstdio>

using namespace std;

int main(int argc, char* argv[]) {
    assert(argc == 2);
    freopen(argv[1], "r", stdin);
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC_RAW, &start);
    size_t n, m;
    auto _ = [&n](size_t i, size_t j) {
        return i * n + j;
    };
    cin >> n >> m;
    vector<size_t> adj(n * n), adj2(n * n), adj3(n * n);
    for(size_t i = 0; i < m; i++) {
        size_t a, b;
        cin >> a >> b;
        adj[_(a, b)] = 1;
        adj[_(b, a)] = 1;
    }
    auto multiply = [&n, &_](const vector<size_t>& a, const vector<size_t>& b, vector<size_t>& c) {
        #pragma omp parallel for schedule(guided) collapse(2)
        for(size_t i = 0; i < n; i++) {
            for(size_t j = 0; j < n; j++) {
                for(size_t k = 0; k < n; k++) {
                    c[_(i, j)] += a[_(i, k)] * b[_(k, j)];
                }
            }
        }
    };
    auto trace = [&n, &_](const vector<size_t>& a) {
        size_t acc = 0;
        for(size_t i = 0; i < n; i++) {
            acc += a[_(i, i)];
        }
        return acc;
    };
    multiply(adj, adj, adj2);
    multiply(adj, adj2, adj3);
    auto t = trace(adj3);
    assert(t % 6 == 0);
#ifdef DUMP_MATRIX
    for(size_t i = 0; i < n; i++) {
        for(size_t j = 0; j < n; j++) {
            cout << adj3[_(i, j)] << " ";
        }
        cout << endl;
    }
#endif
    clock_gettime(CLOCK_MONOTONIC_RAW, &end);
    uint64_t delta_us = (end.tv_sec - start.tv_sec) * 1000000 + (end.tv_nsec - start.tv_nsec) / 1000;
    double delta = double(delta_us) / 1e6;
    printf("%.9lf\n", delta);
    cerr << "Triangles: " << t / 6 << endl;
    return 0;
}

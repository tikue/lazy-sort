# Lazy Sort
A lazy-quicksort adapter for iterators.

## Performance
On my MacBook:
```
test bench::bench_eager ... bench:   3,849,660 ns/iter (+/- 277,016)
test bench::bench_lazy  ... bench:     626,719 ns/iter (+/- 565,033)
```

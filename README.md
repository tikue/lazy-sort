# Lazy Sort
A lazy-quicksort adapter for iterators.

## Performance
On my MacBook, taking the first 1,000 sorted elements from a `Vec<usize>` of `len` 50,000 runs about 6 times faster lazily. Taking all 50,000 elements runs a little over 2 times slower lazily.
```
test bench::take_1000_eager   ... bench:   3,843,635 ns/iter (+/- 257,813)
test bench::take_1000_lazy    ... bench:     684,543 ns/iter (+/- 538,358)
test bench::take_10_000_eager ... bench:   3,808,535 ns/iter (+/- 563,777)
test bench::take_10_000_lazy  ... bench:   2,368,695 ns/iter (+/- 737,863)
test bench::take_50_000_eager ... bench:   3,964,414 ns/iter (+/- 354,603)
test bench::take_50_000_lazy  ... bench:   9,187,479 ns/iter (+/- 1,173,235)
```

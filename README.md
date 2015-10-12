# Lazy Sort
A lazy-quicksort adapter for iterators.

## Performance
On my MacBook, taking the first 1,000 sorted elements from a `Vec<usize>` of `len` 50,000 runs a
little over 6 times faster lazily. Taking all 50,000 elements runs a little under 2 times slower 
lazily.
```
test bench::take_1000_eager   ... bench:   3,814,411 ns/iter (+/- 840,802)
test bench::take_1000_lazy    ... bench:     605,397 ns/iter (+/- 675,910)
test bench::take_10_000_eager ... bench:   3,824,184 ns/iter (+/- 578,607)
test bench::take_10_000_lazy  ... bench:   2,089,243 ns/iter (+/- 1,142,383)
test bench::take_50_000_eager ... bench:   3,901,673 ns/iter (+/- 896,740)
test bench::take_50_000_lazy  ... bench:   7,097,199 ns/iter (+/- 893,629)
```

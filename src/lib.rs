#![feature(slice_splits, core)]
#![cfg_attr(test, feature(test))]
extern crate core;
extern crate itertools;
extern crate rand;

use core::ptr;
use itertools::partition;
use std::cmp::Ordering::{self, Less};
use std::mem;

#[derive(Debug, Clone)]
pub struct LazySort<T> {
    inner: LazySortInternal<T>,
}

impl<T: Ord> Iterator for LazySort<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

#[derive(Debug, Clone)]
enum LazySortInternal<T> {
    Base(Vec<T>),
    Recursive(Recursive<T>),
}

impl<T: Ord> LazySortInternal<T> {
    fn new(mut v: Vec<T>) -> LazySortInternal<T> {
        if v.len() <= 32 {
            insertion_sort(&mut v, |a, b| b.cmp(a));
            LazySortInternal::Base(v)
        } else {
            LazySortInternal::Recursive(Recursive::new(v))
        }
    }
}

impl<T: Ord> Iterator for LazySortInternal<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match *self {
            LazySortInternal::Base(ref mut v) => v.pop(),
            LazySortInternal::Recursive(ref mut r) => r.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match *self {
            LazySortInternal::Base(ref v) => (v.len(), Some(v.len())),
            LazySortInternal::Recursive(ref r) => r.size_hint(),
        }
    }
}

#[derive(Clone, Debug)]
struct Recursive<T> {
    greater: Vec<T>,
    less: Option<Box<LazySortInternal<T>>>,
}

impl<T: Ord> Recursive<T> {
    fn new(v: Vec<T>) -> Recursive<T> {
        Recursive {
            greater: v,
            less: None,
        }
    }

    fn split_greater(&mut self) -> Option<T> {
        match self.greater.len() {
            0 => None,
            1 => self.greater.pop(),
            _ => {
                let pivot_idx = self.greater.len() - 1;
                let split_idx = {
                    let mid_idx = self.greater.len() / 2;
                    // I've chosen the element in the middle of the vec as the pivot.
                    // However, we first swap the pivot with the last element so that there is
                    // a contiguous space in memory to be partitioned.
                    self.greater.swap(pivot_idx, mid_idx);
                    let (pivot, rest) = self.greater.split_last_mut().unwrap();
                    // partition all but the last element, which is the pivot. This makes the vec
                    // look like [greater, greater, ..., greater, less, less, ..., less, pivot]
                    partition(rest, |el| el > pivot)
                };
                // Swapping the pivot with the first less element allows us to split off
                // vec[split_idx + 1..] to create a new vec with all the elements less than pivot.
                self.greater.swap(pivot_idx, split_idx);
                let split_off_idx = split_idx + 1;
                if split_off_idx < self.greater.len() {
                    let mut less = Box::new(LazySortInternal::new(self.greater
                                                                      .split_off(split_off_idx)));
                    // Recursively compute the next element from the LazySortInternal struct containing
                    // the elements less than the pivot.
                    let next = less.next();
                    self.less = Some(less);
                    next
                } else {
                    // If there were no elements less than the pivot, then return the pivot.
                    self.greater.pop()
                }
            }
        }
    }
}

impl<T: Ord> Iterator for Recursive<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let next = if let Some(ref mut less) = self.less {
            less.next()
        } else {
            return self.split_greater();
        };
        if next.is_some() {
            next
        } else {
            self.less = None;
            // The pivot is always the last element in the vec, and it's the first element
            // to be returned once all of the elements less than it have been returned.
            self.greater.pop()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.less {
            None => (self.greater.len(), Some(self.greater.len())),
            Some(ref less) => {
                let (lower, upper) = less.size_hint();
                (lower + self.greater.len(),
                 upper.map(|upper| upper + self.greater.len()))
            }
        }
    }
}

pub trait LazySortIterator: Iterator
    where Self: Sized,
          Self::Item: Ord
{
    fn lazy_sort(self) -> LazySort<Self::Item> {
        LazySort { inner: LazySortInternal::new(self.collect()) }
    }
}

impl<T> LazySortIterator for T
    where T: Iterator,
          T::Item: Ord { }

// This is copied from libcollections/slice.rs
fn insertion_sort<T, F>(v: &mut [T], mut compare: F)
    where F: FnMut(&T, &T) -> Ordering
{
    let len = v.len() as isize;
    let buf_v = v.as_mut_ptr();

    // 1 <= i < len;
    for i in 1..len {
        // j satisfies: 0 <= j <= i;
        let mut j = i;
        unsafe {
            // `i` is in bounds.
            let read_ptr = buf_v.offset(i) as *const T;

            // find where to insert, we need to do strict <,
            // rather than <=, to maintain stability.

            // 0 <= j - 1 < len, so .offset(j - 1) is in bounds.
            while j > 0 && compare(&*read_ptr, &*buf_v.offset(j - 1)) == Less {
                j -= 1;
            }

            // shift everything to the right, to make space to
            // insert this value.

            // j + 1 could be `len` (for the last `i`), but in
            // that case, `i == j` so we don't copy. The
            // `.offset(j)` is always in bounds.

            if i != j {
                let tmp = ptr::read(read_ptr);
                ptr::copy(&*buf_v.offset(j), buf_v.offset(j + 1), (i - j) as usize);
                ptr::copy_nonoverlapping(&tmp, buf_v.offset(j), 1);
                mem::forget(tmp);
            }
        }
    }
}

#[test]
fn test_sort() {
    let mut v = vec![2, 4, 2, 5, 8, 4, 3, 4, 6];
    let v2: Vec<_> = v.iter().cloned().lazy_sort().collect();
    v.sort();
    assert_eq!(v, v2);
}

#[test]
fn test_empty() {
    let v: Vec<u64> = vec![];
    let v2: Vec<_> = v.iter().cloned().lazy_sort().collect();
    assert_eq!(v, v2);
}

#[test]
fn test_size_hint() {
    let v = vec![2, 4, 2, 5, 8, 4, 3, 4, 6];
    let mut sort_iter = v.iter().cloned().lazy_sort();
    for i in 0..v.len() {
        let (lower, upper) = sort_iter.size_hint();
        assert_eq!(v.len() - i, lower);
        assert_eq!(Some(v.len() - i), upper);
        sort_iter.next();
    }
}

#[cfg(test)]
mod bench {
    extern crate test;

    use self::test::Bencher;
    use rand::{thread_rng, Rng};
    use super::LazySortIterator;

    #[bench]
    fn take_1000_lazy(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| v.iter().cloned().lazy_sort().take(1000).collect::<Vec<_>>());
    }

    #[bench]
    fn take_1000_eager(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| {
            let mut v = v.clone();
            v.sort();
            v.iter().cloned().take(1000).collect::<Vec<_>>();
        });
    }

    #[bench]
    fn take_10_000_lazy(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| v.iter().cloned().lazy_sort().take(10_000).collect::<Vec<_>>());
    }

    #[bench]
    fn take_10_000_eager(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| {
            let mut v = v.clone();
            v.sort();
            v.iter().cloned().take(10_000).collect::<Vec<_>>();
        });
    }

    #[bench]
    fn take_50_000_lazy(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| v.iter().cloned().lazy_sort().take(50_000).collect::<Vec<_>>());
    }

    #[bench]
    fn take_50_000_eager(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| {
            let mut v = v.clone();
            v.sort();
            v.iter().cloned().take(50_000).collect::<Vec<_>>();
        });
    }
}

#![feature(slice_splits)]
#![cfg_attr(test, feature(test))]
extern crate itertools;
extern crate rand;

use itertools::partition;

#[derive(Debug, Clone)]
pub struct LazySort<T> {
    greater: Vec<T>,
    less: Option<Box<LazySort<T>>>,
}

impl<T: Ord> LazySort<T> {
    #[inline]
    fn split_greater(&mut self) -> Option<T> {
        match self.greater.len() {
            0 => None,
            1 => self.greater.pop(),
            _ => {
                let pivot_idx = self.greater.len() - 1;
                let split_idx = {
                    let mid_idx = self.greater.len() / 2;
                    // We swap the middle value with the last value because partitioning is easier
                    // if the pivot is located in the last index, making contiguous the space to 
                    // partition
                    self.greater.swap(pivot_idx, mid_idx);
                    let (pivot, rest) = self.greater.split_last_mut().unwrap();
                    partition(rest, |el| el > pivot)
                };
                self.greater.swap(pivot_idx, split_idx);
                let split_off_idx = split_idx + 1;
                if split_off_idx < self.greater.len() {
                    let mut less = Box::new(LazySort {
                        greater: self.greater.split_off(split_off_idx),
                        less: None,
                    });
                    let next = less.next();
                    self.less = Some(less);
                    next
                } else {
                    self.greater.pop()
                }
            }
        }
    }
}

impl<T: Ord> Iterator for LazySort<T> {
    type Item = T;

    #[inline]
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
    where Self: Sized
{
    fn lazy_sort(self) -> LazySort<Self::Item> {
        LazySort {
            greater: self.collect(),
            less: None,
        }
    }
}

impl<T> LazySortIterator for T where T: Iterator { }

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

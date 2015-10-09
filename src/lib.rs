#![feature(slice_splits)]
#![cfg_attr(test, feature(test))]
extern crate itertools;
extern crate rand;
extern crate typed_arena;

use itertools::partition;
use typed_arena::Arena;

pub struct LazySort<T: 'static> {
    arena: Arena<Node<T>>,
    root: Node<T>,
}

impl<T: Ord> Iterator for LazySort<T> {
    type Item = T;
    
    #[inline]
    fn next(&mut self) -> Option<T> {
        self.root.next(&self.arena)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.root.size_hint()
    }
}

#[derive(Debug)]
struct Node<T: 'static> {
    greater: Vec<T>,
    less: Option<&'static mut Node<T>>,
}

impl<T: Ord> Node<T> {
    #[inline]
    fn next(&mut self, arena: &Arena<Self>) -> Option<T> {
        let next = if let Some(ref mut less) = self.less {
            less.next(arena)
        } else {
            return self.split_greater(arena);
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

    fn split_greater(&mut self, arena: &Arena<Self>) -> Option<T> {
        match self.greater.len() {
            0 => None,
            1 => self.greater.pop(),
            _ => {
                let split_idx = {
                    let (pivot, rest) = self.greater.split_last_mut().unwrap();
                    partition(rest, |el| el > pivot)
                };
                let pivot_idx = self.greater.len() - 1;
                self.greater.swap(pivot_idx, split_idx);
                let split_off_idx = split_idx + 1;
                if split_off_idx < self.greater.len() {
                    let mut less = arena.alloc(Node {
                        greater: self.greater.split_off(split_off_idx),
                        less: None,
                    });
                    let next = less.next(arena);
                    self.less = Some(unsafe { std::mem::transmute(less) });
                    next
                } else {
                    self.greater.pop()
                }
            }
        }
    }
}

pub trait LazySortIterator: Iterator
    where Self: Sized
{
    fn lazy_sort(self) -> LazySort<Self::Item> {
        let (lower, upper) = self.size_hint();
        LazySort {
            arena: Arena::with_capacity(upper.unwrap_or(lower)),
            root: Node {
                greater: self.collect(),
                less: None,
            },
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
    fn bench_lazy(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| v.iter().cloned().lazy_sort().take(1000).collect::<Vec<_>>());
    }

    #[bench]
    fn bench_eager(b: &mut Bencher) {
        let mut rng = thread_rng();
        let v: Vec<u32> = rng.gen_iter().take(50_000).collect();
        b.iter(|| {
            let mut v = v.clone();
            v.sort();
            v.iter().cloned().take(1000).collect::<Vec<_>>();
        });
    }
}

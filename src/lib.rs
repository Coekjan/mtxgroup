#![cfg_attr(not(feature = "std"), no_std)]
#![deny(warnings)]

#[cfg(any(
    not(any(feature = "std", feature = "spin")),
    all(feature = "std", feature = "spin")
))]
compile_error!("Exactly one of the features `std` and `spin` must be enabled");

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::sync::{Mutex, MutexGuard};

#[cfg(feature = "spin")]
use spin::{Mutex, MutexGuard};

pub struct MutexGroup<'a, T> {
    mutexes: Vec<&'a Mutex<T>>,
}

pub struct MutexGroupGuard<'a, T> {
    guards: Vec<MutexGuard<'a, T>>,
}

pub struct MutexGroupGuardIter<'a, 'b, T> {
    iter: core::slice::Iter<'a, MutexGuard<'b, T>>,
}

impl<'a, T> MutexGroup<'a, T> {
    pub fn new(mutexes: impl Iterator<Item = &'a Mutex<T>>) -> Self {
        Self {
            mutexes: mutexes.collect::<Vec<_>>(),
        }
    }

    pub fn lock(&self) -> MutexGroupGuard<'_, T> {
        loop {
            if let Some(guards) = self.try_lock_all() {
                return MutexGroupGuard { guards };
            }
            core::hint::spin_loop();
        }
    }

    fn try_lock_all(&self) -> Option<Vec<MutexGuard<'_, T>>> {
        let guards = self
            .mutexes
            .iter()
            .map(|mtx| {
                #[cfg(feature = "std")]
                {
                    mtx.try_lock().ok()
                }
                #[cfg(feature = "spin")]
                {
                    mtx.try_lock()
                }
            })
            .collect::<Vec<_>>();
        if guards.iter().any(|guard| guard.is_none()) {
            None
        } else {
            guards.into_iter().collect::<Option<Vec<_>>>()
        }
    }
}

impl<'a, 'b, T> MutexGroupGuard<'b, T> {
    pub fn iter(&'b self) -> MutexGroupGuardIter<'a, 'b, T> {
        MutexGroupGuardIter {
            iter: self.guards.iter(),
        }
    }
}

impl<'a, T> IntoIterator for MutexGroupGuard<'a, T> {
    type IntoIter = <Vec<MutexGuard<'a, T>> as IntoIterator>::IntoIter;

    type Item = MutexGuard<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.guards.into_iter()
    }
}

impl<'a, 'b, T> Iterator for MutexGroupGuardIter<'a, 'b, T> {
    type Item = &'a MutexGuard<'b, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mutex_group_0() {
        static MUTEX_0: Mutex<usize> = Mutex::new(0);
        static MUTEX_1: Mutex<usize> = Mutex::new(1);
        static MUTEX_2: Mutex<usize> = Mutex::new(2);

        #[cfg(feature = "std")]
        let mutexes = vec![&MUTEX_0, &MUTEX_1, &MUTEX_2];

        #[cfg(not(feature = "std"))]
        let mutexes = alloc::vec![&MUTEX_0, &MUTEX_1, &MUTEX_2];

        let mtxgroup = MutexGroup::new(mutexes.into_iter());
        let guard = mtxgroup.lock();

        for (i, mutex) in guard.iter().enumerate() {
            assert_eq!(**mutex, i);
        }

        for (i, mut mutex) in guard.into_iter().enumerate() {
            assert_eq!(*mutex, i);
            *mutex = 0xffff;
        }

        let guard = mtxgroup.lock();
        for mutex in guard.iter() {
            assert_eq!(**mutex, 0xffff);
        }
    }

    #[test]
    fn test_mutex_group_1() {
        let mut mtx_list = Vec::new();
        for i in 0..10 {
            mtx_list.push(Mutex::new(i));
        }
        let mtxgroup = MutexGroup::new(mtx_list.iter());
        let mtxgroup_guard = mtxgroup.lock();
        for (i, mut mtx) in mtxgroup_guard.into_iter().enumerate() {
            assert_eq!(*mtx, i);
            *mtx = i * i * i;
        }

        let mtxgroup_guard = mtxgroup.lock();
        for (i, mtx) in mtxgroup_guard.into_iter().enumerate() {
            assert_eq!(*mtx, i * i * i);
        }
    }
}

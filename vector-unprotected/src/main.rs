#![allow(dead_code)]

use std::{
    alloc::{self, Layout},
    fmt::Debug,
    mem::{self, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

#[derive(Debug)]
struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}

impl<T> RawVec<T> {
    fn new() -> Self {
        assert!(
            std::mem::size_of::<T>() != 0,
            "We're not ready to handle ZSTs"
        );

        RawVec {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    fn grow(self: &mut Self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            let new_layout = Layout::array::<T>(new_cap).unwrap();
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(old_ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }

    fn push(self: &mut Self, element: T) {
        if self.len == self.cap {
            self.grow();
        }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), element);
        }
        self.len += 1;
    }

    fn pop(self: &mut Self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }

    fn insert(self: &mut Self, index: usize, element: T) {
        //Note: `<=` because it's value to insert after everything
        //which would be equivalent to push.
        assert!(index <= self.len, "Index out of bounds");
        if self.cap == self.len {
            self.grow();
        }
        unsafe {
            ptr::copy(
                self.ptr.as_ptr().add(index),
                self.ptr.as_ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.as_ptr().add(index), element);
        }
        self.len += 1;
    }

    fn remove(self: &mut Self, index: usize) -> T {
        //Note: `<` because it's *not* valid to remove after everything
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr.as_ptr().add(index));
            ptr::copy(
                self.ptr.as_ptr().add(index + 1),
                self.ptr.as_ptr().add(index),
                self.len - index,
            );
            result
        }
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(self: &mut Self) {
        if self.cap != 0 {
            while let Some(_) = self.pop() {
                let layout = Layout::array::<T>(self.cap).unwrap();
                unsafe {
                    alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
                }
            }
        }
    }
}

impl<T> Deref for RawVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for RawVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

struct IntoIter<T> {
    buf: NonNull<T>,
    cap: usize,
    start: *const T,
    end: *const T,
}

impl<T> IntoIterator for RawVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        let vec = ManuallyDrop::new(self);

        let ptr = vec.ptr;
        let cap = vec.cap;
        let len = vec.len;

        IntoIter {
            buf: ptr,
            cap,
            start: ptr.as_ptr(),
            end: if cap == 0 {
                ptr.as_ptr()
            } else {
                unsafe { ptr.as_ptr().add(len) }
            },
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(self: &mut Self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.offset(1);
                Some(result)
            }
        }
    }

    fn size_hint(self: &Self) -> (usize, Option<usize>) {
        let len = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.offset(-1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(self: &mut Self) {
        if self.cap != 0 {
            for _ in &mut *self {}
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.buf.as_ptr() as *mut u8, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::alloc::Layout;

    #[test]
    fn it_works() {
        dbg!(std::mem::size_of::<NonNull<u128>>());
        dbg!(std::mem::size_of::<RawVec<i8>>());
        dbg!(std::mem::size_of::<RawVec<i16>>());
        dbg!(std::mem::size_of::<RawVec<i32>>());
        dbg!(Layout::array::<i8>(1).unwrap());
        dbg!(Layout::array::<i16>(5).unwrap());
        // dbg!(Layout::array::<i64>(5).unwrap());
        // dbg!(Layout::array::<i64>(1).unwrap());
        // dbg!(Layout::array::<i128>(5).unwrap());
        // assert!(false);
    }

    #[test]
    fn raw_vec() {
        let mut rv = RawVec::<i32>::new();
        rv.insert(0, 2);
        rv.insert(1, 10);
        rv.insert(2, 50);
        rv.insert(3, 99);
        rv.remove(0);
        rv.remove(1);
        rv.remove(1);
        rv.remove(0);
        dbg!(rv);
        // assert!(false);
    }
}

fn main() {
    println!("Hello, world!");
    // let a = Vec::<i8>::new();
}

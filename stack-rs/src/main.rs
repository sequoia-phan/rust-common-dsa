#![allow(dead_code, warnings)]
/*
    LIFO queue
    Stack: pop/push of O(1)
*/
use std::{
    alloc::{self, Layout},
    mem,
    ptr::{self, NonNull},
};

#[derive(Debug)]
struct Stack<T> {
    ptr: NonNull<T>,
    cap: usize,
    len: usize,
}

unsafe impl<T: Send> Send for Stack<T> {}
unsafe impl<T: Sync> Sync for Stack<T> {}

impl<T> Stack<T> {
    fn new() -> Stack<T> {
        Stack {
            ptr: NonNull::dangling(),
            cap: 0,
            len: 0,
        }
    }

    fn grow(self: &mut Self) {
        let (new_cap, layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = self.cap * 2;
            assert!(new_cap <= isize::MAX as usize, "Allocation too large!!!");
            (new_cap, Layout::array::<T>(self.cap).unwrap())
        };

        let ptr = if self.cap == 0 {
            unsafe { alloc::alloc(layout) }
        } else {
            let new_size = new_cap * mem::size_of::<T>();
            unsafe { alloc::realloc(self.ptr.as_ptr() as *mut u8, layout, new_size) }
        };

        self.cap = new_cap;
        match NonNull::new(ptr as *mut T) {
            Some(ptr) => {
                self.ptr = ptr;
            }
            None => alloc::handle_alloc_error(layout),
        }
    }

    fn pop(self: &mut Self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr.as_ptr().add(self.len))) }
        }
    }

    fn push(self: &mut Self, item: T) {
        if self.cap == self.len {
            self.grow();
        }
        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.len), item);
        }

        self.len += 1;
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        while let Some(item) = self.pop() {
            drop(item);
        }
        unsafe {
            alloc::dealloc(
                self.ptr.as_ptr() as *mut u8,
                Layout::array::<T>(self.cap).unwrap(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_works() {
        let mut s = Stack::<usize>::new();
        assert_eq!(None, s.pop());
        assert_eq!(0, s.len);
        s.push(100000);
        s.push(5);
        s.push(10);
        s.push(5);
        s.push(100000);
        s.push(5);
        s.push(10);
        s.push(5);
        for i in 1..=300 {
            s.push(i);
        }

        for _ in 1..=30 {
            s.pop();
        }
        // assert_eq!(1, s.len);
        dbg!(s);
        assert!(false);
    }

    #[test]
    fn it_works() {
        println!("ok");
    }
}

fn main() {}

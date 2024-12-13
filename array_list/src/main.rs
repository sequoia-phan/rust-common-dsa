#![allow(dead_code)]
use std::fmt::Debug;

type Array<T> = Vec<T>;

#[derive(Debug)]
struct ArrayList<T> {
    length: usize,
    inner: Option<Array<T>>,
}

impl<T> ArrayList<T>
where
    T: Default + Clone + Copy + Debug + PartialEq,
{
    fn new() -> ArrayList<T> {
        ArrayList {
            length: 0,
            inner: Some(vec![T::default(); 5]),
        }
    }

    fn grow_inner(self: &mut Self) {
        let prev = self.inner.as_ref().unwrap();
        let mut new = vec![T::default(); prev.len() * 2];
        for i in 0..prev.len() {
            new[i] = prev[i];
        }
        // dbg!(&new);
        self.inner = Some(new);
    }

    fn append(self: &mut Self, item: T) {
        println!("appending {item:?} to {self:?}");
        if self.inner.as_ref().unwrap().len() == self.length {
            self.grow_inner();
        }
        self.inner.as_mut().unwrap()[self.length] = item;
        self.length = self.length + 1;
        dbg!(&self);
    }

    fn pop(self: &mut Self) -> T {
        let tail = self.length - 1;
        let item = self.inner.as_ref().unwrap()[tail];
        println!("removing {item:?} from {self:?}");
        self.inner.as_mut().unwrap()[tail] = T::default();
        self.length = tail;
        item
    }

    fn remove_at(self: &mut Self, index: usize) -> Option<T> {
        // find the element at index
        if index < self.length {
            let item = self.inner.as_ref().unwrap()[index];
            // shift_left all remaining elements
            for i in index..self.length {
                self.inner.as_mut().unwrap()[i] = self.inner.as_ref().unwrap()[i + 1];
            }
            // decrease length
            self.length = self.length - 1;
            dbg!(&self);
            Some(item)
        } else {
            None
        }
    }

    fn remove(self: &mut Self, element: &T) -> Option<T> {
        for i in 0..self.length {
            if &self.inner.as_ref().unwrap()[i] == element {
                return self.remove_at(i);
            }
        }
        None
    }

    fn get(self: &Self, i: usize) -> Option<&T> {
        if i < self.length {
            Some(&self.inner.as_ref().unwrap()[i])
        } else {
            None
        }
    }

    fn shift(self: &mut Self, offset: usize) {
        if self.inner.as_ref().unwrap().len() == self.length {
            self.grow_inner();
        }
        for i in ((offset + 1)..=self.length).rev() {
            self.inner.as_mut().unwrap()[i] = self.inner.as_ref().unwrap()[i - 1];
        }
    }

    fn insert_at(self: &mut Self, index: usize, item: T) {
        println!("inserting {item:?} at {index:?} into {self:?}");
        if self.inner.as_ref().unwrap().len() == self.length {
            self.grow_inner();
        }
        for i in ((index + 1)..=self.length).rev() {
            self.inner.as_mut().unwrap()[i] = self.inner.as_ref().unwrap()[i - 1];
        }
        self.inner.as_mut().unwrap()[index] = item;
        self.length = self.length + 1;
        dbg!(self);
    }

    fn prepend(self: &mut Self, item: T) {
        println!("prepending {item:?} to {self:?}");
        // shift everything over one

        self.insert_at(0, item);

        // self.shift(0);
        // self.inner.as_mut().unwrap()[0] = item;
        // self.length = self.length + 1;
        // dbg!(&self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_list_work() {
        let mut l = ArrayList::<i32>::new();
        l.append(2);
        l.append(5);
        l.append(5);
        l.append(5);
        l.append(5);
        l.append(6);
        l.append(9);
        l.pop();
        assert_eq!(Some(&5), l.get(2));
        l.prepend(1000);
        l.append(9);
        assert_eq!(Some(&1000), l.get(0));
        assert_eq!(Some(&9), l.get(l.length - 1));
        assert_eq!(8, l.length);
        l.insert_at(1, 2000);
        assert_eq!(Some(&2000), l.get(1));
        assert!(false);
    }

    #[test]
    fn remove_test() {
        let mut arl = ArrayList::<i16>::new();
        arl.append(9);
        arl.append(10);
        arl.append(20);
        arl.append(30);
        // let removed = arl.remove_at(1);
        // assert!(false);
        assert_eq!(Some(10), arl.remove_at(1));
        assert_eq!(3, arl.length);
        assert_eq!(Some(&30), arl.get(arl.length - 1));
        assert_eq!(Some(20), arl.remove(&20));
        assert_eq!(None, arl.remove(&200));
        assert!(false);
    }
}

fn main() {
    let mut arrlist = ArrayList::<i32>::new();
    arrlist.append(1);
    arrlist.append(2);
    arrlist.append(3);
    arrlist.append(4);
    arrlist.append(5);
    arrlist.append(6);
    arrlist.append(7);
    arrlist.append(8);
    arrlist.append(9);
    arrlist.append(10);
    arrlist.append(11);
    arrlist.pop();
    arrlist.prepend(1000);
    arrlist.insert_at(2, 500);
    dbg!(&arrlist);
    println!("{}", arrlist.length);
    println!("get from last index {}", arrlist.get(2).unwrap());
}

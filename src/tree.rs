//! Unsafe tree

use std::{ptr, marker::PhantomData};

pub struct Tree<T> {
    /// nullable
    root: *mut Node<T>,
}

struct Node<T> {
    /// nullable
    parent: *mut Node<T>,
    value: T,

    left: *mut Node<T>,
    right: *mut Node<T>,
}

#[derive(Clone, Copy)]
pub struct Cursor<'a, T> {
    /// nullable
    ptr: *mut Node<T>,
    lifetime: PhantomData<&'a ()>,
}

impl<T> Tree<T> {
    pub fn empty() -> Self {
        Self {
            root: ptr::null_mut(),
        }
    }

    pub fn leaf(value: T) -> Self {
        let root = Box::into_raw(Box::new(Node {
            parent: ptr::null_mut(),
            value,
            left: ptr::null_mut(),
            right: ptr::null_mut(),
        }));
        Self { root }
    }

    pub fn branch(value: T, left: Self, right: Self) -> Self {
        let root = Box::into_raw(Box::new(Node {
            parent: ptr::null_mut(),
            value,
            left: left.root,
            right: right.root,
        }));
        Self { root }
    }

    pub fn root<'a>(&'a self) -> Cursor<'a, T> {
        Cursor { ptr: self.root, lifetime: PhantomData }
    }
}

impl<'a, T> Cursor<'a, T> {
    pub fn parent(mut self) -> Self {
        unsafe {
            if !self.ptr.is_null() {
                self.ptr = (*self.ptr).parent;
            }
            self
        }
    }

    pub fn left(mut self) -> Self {
        unsafe {
            if !self.ptr.is_null() {
                self.ptr = (*self.ptr).left;
            }
            self
        }
    }

    pub fn right(mut self) -> Self {
        unsafe {
            if !self.ptr.is_null() {
                self.ptr = (*self.ptr).right;
            }
            self
        }
    }

    pub fn get(self) -> Option<&'a T> {
        unsafe {
            if self.ptr.is_null() {
                None
            } else {
                Some(&(*self.ptr).value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor() {
        let tree = Tree::branch(
            5,
            Tree::leaf(3),
            Tree::branch(8, Tree::leaf(7), Tree::leaf(9)),
        );

        let three: &i32 = tree.root().left().get().unwrap();
        dbg!(three);

        drop(tree);

        // dbg!(three);
    }

    #[test]
    fn test_clone() {
        let tree = Tree::branch(
            5,
            Tree::leaf(3),
            Tree::branch(8, Tree::leaf(7), Tree::leaf(9)),
        );

        let eight = tree.root().right();
        assert_eq!(*eight.left().get().unwrap(), 7);
        assert_eq!(*eight.right().get().unwrap(), 9);
    }
}

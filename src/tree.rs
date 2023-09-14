//! Unsafe tree

use std::{marker::PhantomData, ptr};

pub struct Tree<T> {
    root: *mut Node<T>,
}

struct Node<T> {
    parent: *mut Node<T>,
    value: T,

    left: *mut Node<T>,
    right: *mut Node<T>,
}

impl<T> Drop for Tree<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.root.is_null() {
                let _ = Box::from_raw(self.root);
            }
        }
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.left.is_null() {
                let _ = Box::from_raw(self.left);
            }
            if !self.right.is_null() {
                let _ = Box::from_raw(self.right);
            }
        }
    }
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
        unsafe {
            (*left.root).parent = root;
            (*right.root).parent = root;
        }
        std::mem::forget(left);
        std::mem::forget(right);
        Self { root }
    }

    pub fn root(&self) -> Cursor<'_, T> {
        Cursor {
            ptr: self.root,
            lifetime: PhantomData,
        }
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
        unsafe { self.ptr.as_ref().map(|v| &v.value) }
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

        let eight = tree.root().right().parent().right();
        assert_eq!(*eight.left().get().unwrap(), 7);
        assert_eq!(*eight.right().get().unwrap(), 9);
    }
}

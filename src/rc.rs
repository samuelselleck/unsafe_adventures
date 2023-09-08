use std::ops::Deref;

pub struct Rc<T> {
    // todo: NonNull ?
    data: *mut HeapData<T>,
}

pub struct Weak<T> {
    data: *mut HeapData<T>,
}

impl<T> Weak<T> {
    #[cfg(test)]
    fn strong_count(&self) -> u32 {
        unsafe { (*self.data).strong_count }
    }

    #[cfg(test)]
    fn weak_count(&self) -> u32 {
        unsafe { (*self.data).weak_count }
    }
}

struct HeapData<T> {
    weak_count: u32,
    strong_count: u32,
    inner: Option<T>,
}

impl<T> Rc<T> {
    pub fn new(inner: T) -> Self {
        let data = Box::into_raw(Box::new(HeapData {
            strong_count: 1,
            weak_count: 0,
            inner: Some(inner),
        }));
        Self { data }
    }

    #[cfg(test)]
    fn strong_count(&self) -> u32 {
        unsafe { (*self.data).strong_count }
    }

    #[cfg(test)]
    fn weak_count(&self) -> u32 {
        unsafe { (*self.data).weak_count }
    }

    pub fn weak(&self) -> Weak<T> {
        unsafe {
            (*self.data).weak_count += 1;
        }
        Weak { data: self.data }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.data).strong_count += 1;
        }
        Self { data: self.data }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.data).strong_count -= 1;
            if (*self.data).strong_count == 0 {
                (*self.data).inner = None
            }
            if (*self.data).strong_count == 0 && (*self.data).weak_count == 0 {
                let _ = Box::from_raw(self.data);
            }
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        unsafe {
            (*self.data).weak_count -= 1;
            if (*self.data).strong_count == 0 && (*self.data).weak_count == 0 {
                let _ = Box::from_raw(self.data);
            }
        }
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { (*self.data).inner.as_ref().unwrap() }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[test]
    fn test() {
        let rc = Rc::new(String::from("hello world"));

        let rc2 = Rc::clone(&rc);

        {
            let _rc3 = Rc::clone(&rc2);
            assert_eq!(rc.strong_count(), 3);
        }
        assert_eq!(rc.strong_count(), 2);

        drop(rc); // (out of order)

        assert_eq!(rc2.strong_count(), 1);
    }

    #[test]
    fn weak_cycle() {
        struct A {
            friend: Option<Weak<RefCell<B>>>,
        }

        struct B {
            _friend: Option<Rc<RefCell<A>>>,
        }
        let a = Rc::new(RefCell::new(A { friend: None }));
        let b = Rc::new(RefCell::new(B {
            _friend: Some(Rc::clone(&a)),
        }));

        assert_eq!(a.strong_count(), 2);
        assert_eq!(b.strong_count(), 1);
        assert_eq!(b.weak_count(), 0);

        a.borrow_mut().friend = Some(Rc::weak(&b));
        assert_eq!(b.weak_count(), 1);
        assert_eq!(a.borrow_mut().friend.as_ref().unwrap().strong_count(), 1);
        drop(b);
        assert_eq!(a.borrow_mut().friend.as_ref().unwrap().strong_count(), 0);
    }

    // randomized tests?

    // fn assert_send<T: Send>() {}
    // // fails to compile
    // #[test]
    // fn is_send() {
    //     assert_send::<Rc<String>>();
    // }

    // fn assert_sync<T: Sync>() {}
    // // fails to compile
    // #[test]
    // fn is_sync() {
    //     assert_sync::<Rc<String>>();
    // }
}

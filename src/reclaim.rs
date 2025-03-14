use std::gc::Gc;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::{fmt, ptr};

pub(crate) struct Atomic<T>(pub(crate) AtomicPtr<T>);

impl<T> Atomic<T> {
    pub(crate) fn null() -> Self {
        Self(AtomicPtr::default())
    }

    pub(crate) fn load<'g>(&self, ordering: Ordering) -> *mut T {
        self.0.load(ordering)
    }

    pub(crate) fn store(&self, new: *mut T, ordering: Ordering) {
        self.0.store(new, ordering);
    }

    //pub(crate) unsafe fn into_boxt(self) -> Box<T> {
    //    Box::from_raw(self.0.into_inner() as *mut T)
    //}

    pub(crate) unsafe fn into_box(self) -> Box<T> {
        unsafe { Box::from_raw(self.0.into_inner()) }
    }

    pub(crate) fn swap<'g>(&self, new: Shared<'_, T>, ord: Ordering) -> Shared<'g, T> {
        self.0.swap(new.into(), ord).into()
    }

    pub(crate) fn compare_exchange(
        &self,
        current: *mut T,
        new: *mut T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<*mut T, *mut T> {
        self.0.compare_exchange(current, new, success, failure)
    }

    pub(crate) fn compare_exchange_weak(
        &self,
        current: *mut T,
        new: *mut T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<*mut T, *mut T> {
        self.0.compare_exchange_weak(current, new, success, failure)
    }
}

impl<T> From<Shared<'_, T>> for Atomic<T> {
    fn from(shared: Shared<'_, T>) -> Self {
        Atomic(AtomicPtr::new(shared.into()))
    }
}

impl<T> Clone for Atomic<T> {
    fn clone(&self) -> Self {
        Atomic(self.0.load(Ordering::Relaxed).into())
    }
}

impl<T> fmt::Debug for Shared<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", Into::<*mut T>::into(*self))
    }
}

impl<T> fmt::Debug for Atomic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.0.load(Ordering::SeqCst))
    }
}

pub(crate) struct CompareExchangeError<'g, T> {
    pub(crate) current: Shared<'g, T>,
    pub(crate) new: Shared<'g, T>,
}

pub(crate) struct Shared<'g, T> {
    pub(crate) ptr: Option<Gc<T>>,
    _g: PhantomData<&'g ()>,
}

impl<'g, T> Shared<'g, T> {
    pub(crate) fn boxed(value: T) -> Self {
        //Shared::from(Gc::into_raw(Gc::new(value)) as *mut T)
        Shared {
            ptr: Some(Gc::new(value)),
            _g: PhantomData,
        }
    }
}

impl<'g, T> Shared<'g, T> {
    pub(crate) fn null() -> Self {
        Shared::from(ptr::null_mut())
    }

    pub(crate) unsafe fn into_box(self) -> Box<T> {
        unsafe { Box::from_raw(Into::<*mut T>::into(self)) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut T {
        Into::<*mut T>::into(*self)
    }

    pub(crate) unsafe fn as_ref(&self) -> Option<&'g T> {
        unsafe { Into::<*mut T>::into(*self).as_ref() }
    }

    pub(crate) unsafe fn deref(&self) -> &'g T {
        unsafe { &*Into::<*mut T>::into(*self) }
        //*self.ptr.unwrap()
        //self.ptr.as_ref().unwrap().as_ref()
        //std::ops::Deref::deref(self.ptr.as_ref().unwrap()) //&Option<T> -> Option<&T>
        //&*(self.ptr.unwrap())
    }

    pub(crate) fn is_null(&self) -> bool {
        Into::<*mut T>::into(*self).is_null()
    }
}

impl<'g, T> PartialEq<Shared<'g, T>> for Shared<'g, T> {
    fn eq(&self, other: &Self) -> bool {
        Into::<*mut T>::into(*self) == Into::<*mut T>::into(*other)
    }
}

impl<T> Eq for Shared<'_, T> {}

impl<T> Clone for Shared<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Shared<'_, T> {}

impl<T> From<*mut T> for Shared<'_, T> {
    fn from(ptr: *mut T) -> Self {
        if ptr.is_null() {
            Shared {
                ptr: None,
                _g: PhantomData,
            }
        } else {
            Shared {
                ptr: Some(Gc::from_raw(ptr)),
                _g: PhantomData,
            }
        }
    }
}

impl<T> From<Shared<'_, T>> for *mut T {
    fn from(shared: Shared<'_, T>) -> Self {
        match shared.ptr {
            Some(gc) => Gc::into_raw(gc) as *mut T,
            None => ptr::null_mut(),
        }
    }
}

#![no_std]
#![feature(coerce_unsized)]
#![feature(unsize)]

extern crate error;
extern crate ffi;
extern crate result;

use core::marker::Unsize;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use error::prelude::*;
use result::Result;

errors!(Alloc, MisalignedPointer);

#[derive(Clone, Copy)]
pub struct Ptr<T: ?Sized> {
    ptr: *const T,
}

impl<T, U> CoerceUnsized<Ptr<U>> for Ptr<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

impl<T: ?Sized> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw() as *const u8 as usize == other.raw() as *const u8 as usize
    }
}

impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw() }
    }
}

impl<T> DerefMut for Ptr<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw() }
    }
}

impl<T> AsRef<T> for Ptr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &(*self.raw()) }
    }
}

impl<T> AsMut<T> for Ptr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.raw()) }
    }
}

impl<T: ?Sized> Ptr<T> {
    pub fn new(ptr: *const T) -> Self {
        if (ptr as *const u8 as usize) % 2 != 0 {
            /*
            exit!(
                "Invalid pointer passed in to Ptr::new. Address must be divisible by 2! ptr={}",
                ptr as *const u8 as usize
            );
            */
        }
        Self { ptr }
    }

    pub fn new_bit_set(mut ptr: *const T) -> Self {
        let tmp = (&mut ptr) as *const _ as *const *const u8;
        unsafe {
            ffi::ptr_add(tmp as *mut _, 1);
        }
        Self { ptr }
    }

    pub fn is_null(&self) -> bool {
        self.raw().is_null()
    }

    pub fn get_bit(&self) -> bool {
        self.ptr as *const u8 as usize % 2 != 0
    }

    pub fn raw(&self) -> *mut T {
        if self.get_bit() {
            let mut ret = self.ptr;
            unsafe {
                ffi::ptr_add(&mut ret as *mut _ as *mut u8, -1);
            }
            ret as *mut T
        } else {
            self.ptr as *mut T
        }
    }

    pub fn release(&self) {
        unsafe {
            ffi::release(self.raw() as *const u8);
        }
    }

    pub fn resize<R>(&mut self, n: usize) -> Result<Ptr<R>> {
        let ptr = unsafe { ffi::resize(self.raw() as *const u8, n) };
        if ptr.is_null() {
            err!(Alloc)
        } else {
            if (ptr as *const u8 as usize) % 2 != 0 {
                err!(MisalignedPointer)
            } else {
                Ok(Ptr {
                    ptr: ptr as *const R,
                })
            }
        }
    }
}

pub fn real_main(_argc: i32, _argv: *const *const i8) -> i32 {
    0
}

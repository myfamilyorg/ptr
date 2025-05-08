#![no_std]
#![feature(coerce_unsized)]
#![feature(unsize)]

extern crate error;
extern crate errors;
extern crate ffi;
extern crate macros;
extern crate raw;
extern crate result;

use core::marker::Unsize;
use core::ops::{CoerceUnsized, Deref, DerefMut};
use error::*;
use errors::*;
use macros::prelude::*;
use raw::{AsRaw, AsRawMut};
use result::Result;

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
        self.as_ptr() as *const u8 as usize == other.as_ptr() as *const u8 as usize
    }
}

impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<T> DerefMut for Ptr<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<T> AsRef<T> for Ptr<T> {
    fn as_ref(&self) -> &T {
        unsafe { &(*self.as_ptr()) }
    }
}

impl<T> AsMut<T> for Ptr<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut (*self.as_mut_ptr()) }
    }
}

impl<T: ?Sized> AsRaw<T> for Ptr<T>
where
    Self: Sized,
{
    fn as_ptr(&self) -> *const T {
        if self.get_bit() {
            let mut ret = self.ptr;
            unsafe {
                ffi::ptr_add(&mut ret as *mut _ as *mut u8, -1);
            }
            ret as *const T
        } else {
            self.ptr as *const T
        }
    }
}

impl<T: ?Sized> AsRawMut<T> for Ptr<T>
where
    Self: Sized,
{
    fn as_mut_ptr(&mut self) -> *mut T {
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
}

impl<T: ?Sized> Ptr<T> {
    pub fn new(ptr: *const T) -> Self {
        if (ptr as *const u8 as usize) % 2 != 0 {
            exit!(
                "Invalid pointer passed in to Ptr::new. Address must be divisible by 2! ptr={}",
                ptr as *const u8 as usize
            );
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
        self.as_ptr().is_null()
    }

    pub fn set_bit(&mut self, v: bool) {
        let ptr = (&mut self.ptr) as *const _ as *const *const u8;
        if v && (self.ptr as *const u8 as usize) % 2 == 0 {
            unsafe {
                ffi::ptr_add(ptr as *mut _, 1);
            } // Add 1 to set the bit
        } else if !v && (self.ptr as *const u8 as usize) % 2 != 0 {
            unsafe {
                ffi::ptr_add(ptr as *mut _, -1);
            } // Subtract 1 to clear the bit
        }
    }

    pub fn get_bit(&self) -> bool {
        self.ptr as *const u8 as usize % 2 != 0
    }

    /*
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
    */

    pub fn release(&self) {
        unsafe {
            ffi::release(self.as_ptr() as *const u8);
        }
    }

    pub fn resize<R>(&mut self, n: usize) -> Result<Ptr<R>> {
        let ptr = unsafe { ffi::resize(self.as_ptr() as *const u8, n) };
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

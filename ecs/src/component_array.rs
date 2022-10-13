use core::slice;
use std::{
    alloc::{self, Layout},
    intrinsics::size_of,
    mem::align_of,
    ptr::{self, null, NonNull},
};

//wrap a vector supporting arbitrary types
//doing lots of copying for transformations right now
//that should change
pub struct ComponentArray {
    ptr: NonNull<u8>,
    element_size: usize,
    length: usize,
    capacity: usize,
}

impl ComponentArray {
    //construct an unitialized boxed component array
    pub fn new(element_size: usize) -> Self {
        //create a new vector with a basic setup
        Self {
            ptr: NonNull::dangling(),
            element_size,
            length: 0,
            capacity: 0,
        }
    }

    //devectorize on object
    pub fn from_slice<T>(slice: &[T]) -> Self 
    where
        T: Clone
    {
        //could potentiall fuck up if component_array hasn't been allocated
        let mut component_array = ComponentArray::new(size_of::<T>());
        for e in slice.iter() {
            let owned_e = e.clone();
            unsafe {
                component_array.push(slice::from_raw_parts((&owned_e as *const T) as *const u8, size_of::<T>()));
            }
        }

        component_array
    }

    pub fn grow(&mut self) {
        let (new_cap, new_layout) = if self.capacity == 0 {
            (
                1,
                Layout::array::<u8>(self.element_size).expect("Failed to push component array"),
            )
        } else {
            let new_cap = self.capacity * 2;

            let new_layout = Layout::array::<u8>(self.capacity * self.element_size)
                .expect("Failed to push component array");
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.capacity * self.element_size).unwrap();
            let ptr = self.ptr.as_ptr() as *mut u8;
            unsafe { alloc::realloc(ptr, old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
        }
    }

    fn push(&mut self, slice: &[u8]) {
        assert!(
            slice.len() == self.element_size,
            "Pushing an object of the incorrect type"
        );

        if self.length == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::copy(
                slice.as_ptr(),
                self.ptr.as_ptr().add(self.length * self.element_size),
                slice.len(),
            );
        }
        self.length += 1;
    }

    pub fn pop<T>(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            unsafe {
                let ptr = self
                    .ptr
                    .as_ptr()
                    .offset(isize::try_from((self.length - 1) * self.element_size).unwrap())
                    .cast::<T>();
                // and some sanity checks
                debug_assert!(
                    ptr as usize % align_of::<T>() == 0,
                    "Popping to the wrong type"
                );
                self.length -= 1;

                Some(ptr.read())
            }
        }
    }

    //change this to return a result instead of asserting
    pub fn remove(&mut self, i: usize) {
        assert!(i <= self.length);
        if self.length == 1 || i == self.length {
            //a special case where no copying is necessary
            self.length -= 1;
        } else {
            //get a ptr to all the elements above the element we're removing
            let copy_length = (self.length - i) * self.element_size;
            unsafe {
                let src_ptr = self.ptr.as_ptr().offset(isize::try_from((i + 1) * self.element_size).unwrap());
                let dst_ptr = self.ptr.as_ptr().offset(isize::try_from(i * self.element_size).unwrap());
                //copy and overwrite the element at i
                ptr::copy(dst_ptr, src_ptr, copy_length)
            }
        }
    }
}

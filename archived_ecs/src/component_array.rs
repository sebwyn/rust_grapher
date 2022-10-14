use core::slice;
use std::{
    alloc::{self, Layout},
    fmt::Debug,
    intrinsics::size_of,
    mem::align_of,
    ptr::{self, null},
};

//for now we dont need to implement insert, because we wont use it
//
pub struct ComponentArray {
    ptr: *mut u8,
    element_size: usize,
    length: usize,
    capacity: usize,
}

impl ComponentArray {
    //construct an unitialized boxed component array
    pub fn new(element_size: usize) -> Self {
        //create a new vector with a basic setup
        Self {
            ptr: null::<u8>() as *mut u8,
            element_size,
            length: 0,
            capacity: 0,
        }
    }

    //copy inner data to our pointer
    pub fn from_slice<T>(slice: &[T]) -> Self
    where
        T: Clone,
    {
        //could potentiall fuck up if component_array hasn't been allocated
        let mut component_array = ComponentArray::new(size_of::<T>());
        for e in slice.iter() {
            let owned_e = e.clone();
            let slice = unsafe {
                slice::from_raw_parts(
                    (&owned_e as *const T) as *const u8,
                    size_of::<T>(),
                )
            };
            component_array.push_bytes(slice);
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

            let new_layout = Layout::array::<u8>(new_cap * self.element_size)
                .expect("Failed to push component array");
            (new_cap, new_layout)
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too larget"
        );

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.capacity * self.element_size).unwrap();
            unsafe { alloc::realloc(self.ptr, old_layout, new_layout.size()) }
        };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout)
        }

        self.ptr = new_ptr;
        self.capacity = new_cap;
    }

    pub fn push_bytes(&mut self, slice: &[u8]) {
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
                self.ptr.add(self.length * self.element_size),
                slice.len(),
            );
        }
        self.length += 1;
    }

    pub fn pop_bytes(&mut self) -> Option<Vec<u8>> {
        assert!(self.length > 0);
        self.remove_bytes(self.length - 1)
    }

    //change this to return a result instead of asserting
    pub fn remove_bytes(&mut self, i: usize) -> Option<Vec<u8>> {
        assert!(i <= self.length);
        assert!(self.length > 0);

        if self.length == 0 || i >= self.length {
            None
        } else {
            unsafe {
                //create a vector from reading the ptr
                let element_ptr = self.ptr.add(i * self.element_size);
                let mut vec = Vec::new();
                vec.extend_from_slice(slice::from_raw_parts(element_ptr, self.element_size));
                if i != self.length - 1 {
                    //only copy if we're not removing the back element
                    //get a ptr to all the elements above the element we're removing
                    let next_element_ptr = self.ptr.add((i + 1) * self.element_size);
                    let copy_length = (self.length - (i + 1)) * self.element_size;
                    //copy and overwrite the element at i
                    ptr::copy(next_element_ptr, element_ptr, copy_length);
                }
                self.length -= 1;

                Some(vec)
            }
        }
    }
}

pub trait TypedComponentArray<T> {
    fn push(&mut self, value: &T);
    fn pop(&mut self) -> Option<T>;
    fn remove(&mut self, i: usize) -> Option<T>;
}

impl<T> TypedComponentArray<T> for ComponentArray
where
    T: Copy,
{
    fn push(&mut self, value: &T) {
        let slice =
            unsafe { slice::from_raw_parts::<u8>(value as *const T as *const u8, size_of::<T>()) };
        self.push_bytes(slice);
    }

    fn pop(&mut self) -> Option<T> {
        assert!(size_of::<T>() == self.element_size);

        match self.pop_bytes() {
            None => None,
            Some(bytes) => {
                //convert the bytes to a type here
                let ptr: *const u8 = bytes.as_ptr();
                assert_eq!(ptr.align_offset(align_of::<T>()), 0);
                unsafe { Some(*ptr.cast::<T>()) }
            }
        }
    }

    fn remove(&mut self, i: usize) -> Option<T> {
        assert!(size_of::<T>() == self.element_size);

        match self.remove_bytes(i) {
            None => None,
            Some(bytes) => {
                //convert the bytes to a type here
                let ptr: *const u8 = bytes.as_ptr();
                assert_eq!(ptr.align_offset(align_of::<T>()), 0);
                unsafe {
                    let value = ptr.cast::<T>().read();
                    Some(value)
                }
            }
        }
    }
}

impl Debug for ComponentArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = unsafe {
            (0..2 * self.length)
                .map(|x| self.ptr.cast::<f32>().offset(x as isize).read())
                .collect::<Vec<f32>>()
        };
        f.debug_struct("ComponentArray")
            .field("element_size", &self.element_size)
            .field("length", &self.length)
            .field("capacity", &self.capacity)
            .field("values", &bytes)
            .finish()
    }
}

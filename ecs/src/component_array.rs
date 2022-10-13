use std::intrinsics::size_of;

pub type ComponentArray<T> = Vec<T>;

pub struct BoxedComponentArray {
    ptr: *mut u8,
    element_size: usize,
    size: usize
}

impl BoxedComponentArray {
    pub fn new<T>(component_array: &mut ComponentArray<T>) -> Self {
        let p = component_array as *mut ComponentArray<T>;
        Self {
            ptr: p as *mut u8,
            element_size: size_of::<T>(),
            size: 0
        }
    }

    pub unsafe fn cast<T>(&self) -> &mut ComponentArray<T> {
        let ptr = self.ptr as *mut ComponentArray<T>;
        ptr.as_mut().expect("Trying to cast a null BoxedComponentArray")
    }
}
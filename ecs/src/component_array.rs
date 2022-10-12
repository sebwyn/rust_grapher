pub type ComponentArray<T> = Vec<(usize, T)>;

//if i can cast a component array back to a listOfComponents then we're all good
pub trait BoxedComponentArray {
    fn get_raw_ptr(&mut self) -> *mut u8;
}

impl<T> BoxedComponentArray for Box<ComponentArray<T>> {
    fn get_raw_ptr(&mut self) -> *mut u8 {
        let ptr = &mut **self as *mut ComponentArray<T>;
        ptr as *mut u8
    }
}

pub trait CastToVec {
    unsafe fn cast<T>(&mut self) -> &mut Vec<T>;
}

pub unsafe fn cast_to_component_array<T>(component_array: &mut Box<dyn BoxedComponentArray>) -> &mut ComponentArray<T> {
    let cast_pointer = component_array.get_raw_ptr() as *mut ComponentArray<T>;
    cast_pointer.as_mut().expect("Cannot cast a null component array")
}
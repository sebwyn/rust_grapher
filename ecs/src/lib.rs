mod ecs;
mod component_array;

//pub use ecs::ECS;

#[cfg(test)]
mod test {
    use super::component_array::{cast_to_component_array, ComponentArray, BoxedComponentArray};

    #[test]
    fn test_component_array() {
        //create a list of int components
        let components = ComponentArray::<i32>::new();
        let mut component_array: Box<dyn BoxedComponentArray> = Box::new(Box::new(components.clone()));

        //down cast our component array
        let vec: &mut ComponentArray<i32> = unsafe { cast_to_component_array::<i32>(&mut component_array) };
        //ensure the elements are equal
        for (x, y) in components.iter().zip(vec.iter()) {
            assert_eq!(x, y);
        }
    }
}

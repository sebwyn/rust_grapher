#![feature(core_intrinsics)]

mod ecs;
mod component_array;
mod archetype;

pub use crate::ecs::ECS;

#[cfg(test)]
mod test {
    use super::component_array::{ComponentArray, BoxedComponentArray};
    use crate::ECS;

    #[test]
    fn test_boxed_component_array() {
        //create a list of int components
        let mut components = ComponentArray::<i32>::new();
        for i in 0..10 {
            components.push(i);
        }
        let component_array: BoxedComponentArray = BoxedComponentArray::new(&mut components);

        //down cast our component array
        let vec: &mut ComponentArray<i32> = unsafe { BoxedComponentArray::cast::<i32>(&component_array) };
        //ensure the elements are equal
        assert!(components.len() == vec.len(), "Their lengths didn't match");
    }

    #[test]
    fn test_entity_creation() {
        let mut ecs = ECS::new();
        let entity1 = ecs.create_entity();
        let entity2 = ecs.create_entity();
        assert_ne!(entity1, entity2);
    }
}

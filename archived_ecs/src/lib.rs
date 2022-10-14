#![feature(core_intrinsics)]
#![feature(slice_from_ptr_range)]

mod ecs;
mod component_array;
mod archetype;

pub use crate::ecs::ECS;

#[cfg(test)]
mod test {
    use std::intrinsics::size_of;

    use unique_type_id_derive::UniqueTypeId;

    use super::component_array::ComponentArray;
    use crate::{ECS, component_array::{TypedComponentArray}};

    #[derive(Debug, Clone, PartialEq, Copy, UniqueTypeId)]
    struct MyComponent {
        x: f32,
        y: f32,
    }

    #[derive(UniqueTypeId)]
    struct MyOtherComponent {
        x: usize,
        y: String
    }

    #[test]
    fn test_component_array() {
        //create a list of int components
        let mut components: Vec<MyComponent> = Vec::new();
        for i in 0..10 {
            components.push(MyComponent { x: i as f32, y: i as f32});
        }
        //println!("{:?}", components);
        let mut component_array: ComponentArray = ComponentArray::from_slice(&components);
        //component_array.push(&MyComponent { x: 52f32, y: 52f32 });
        //components.push(MyComponent { x: 52f32, y: 52f32 });
        //println!("{:?}", component_array);
        components.reverse();
        //let last_component: MyComponent = component_array.pop().unwrap();
        //assert_eq!(components[0], component_array.pop().unwrap());
        //let component_8: MyComponent = component_array.pop().unwrap();
        //assert_eq!(components[2], component_array.pop().unwrap());
        for c in components[..].iter() {
            assert_eq!(*c, component_array.pop().unwrap());
        }
        
    }


    #[test]
    fn test_component_array_remove() {
        let mut component_array = ComponentArray::new(size_of::<MyComponent>());
        component_array.push(&MyComponent {x: 52f32, y: 39f32 });
        component_array.push(&MyComponent {x: 29f32, y: 19f32 });
        component_array.push(&MyComponent {x: 82f32, y: 100f32 });

        let second_component: MyComponent = component_array.remove(0).unwrap();
        assert_eq!(MyComponent {x: 52f32, y: 39f32}, second_component);
    }   

    #[test]
    fn test_entity_creation() {
        let mut ecs = ECS::new();
        let entity1 = ecs.create_entity();
        let entity2 = ecs.create_entity();
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn test_component_creation() {
        let mut ecs = ECS::new();
        let entity = ecs.create_entity();
        ecs.add_component(entity, MyComponent {x: 10f32, y: 15f32});
        ecs.add_component(entity, MyOtherComponent {x: 11, y: String::from("hello there")});
    }
}

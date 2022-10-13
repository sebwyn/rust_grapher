#![feature(core_intrinsics)]

mod ecs;
mod component_array;
mod archetype;

pub use crate::ecs::ECS;

#[cfg(test)]
mod test {
    use super::component_array::ComponentArray;
    use crate::ECS;

    #[derive(Debug, Clone, PartialEq)]
    struct MyComponent {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_component_array() {
        //create a list of int components
        let mut components: Vec<MyComponent> = Vec::new();
        for i in 0..10 {
            components.push(MyComponent { x: i as f32, y: i as f32});
        }
        println!("{:?}", components);
        let mut component_array: ComponentArray = ComponentArray::from_slice(&components);

        //ensure the elements are equal
        components.reverse();
        for og_component in components.into_iter() {
            assert_eq!(og_component, component_array.pop::<MyComponent>().unwrap())
        }
    }

    #[test]
    fn test_entity_creation() {
        let mut ecs = ECS::new();
        let entity1 = ecs.create_entity();
        let entity2 = ecs.create_entity();
        assert_ne!(entity1, entity2);
    }
}

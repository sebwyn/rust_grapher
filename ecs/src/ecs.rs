use std::collections::HashMap;
use super::component_array::BoxedComponentArray;


pub struct ECS {
    component_store: HashMap<u32, Box<dyn BoxedComponentArray>>
}

impl ECS {
    
}

use std::{collections::HashMap, hash::Hash};

use super::component_array::BoxedComponentArray;

pub type ComponentId = u64;
pub type ArchetypeId = usize;
pub type Type = Vec<ComponentId>;

pub struct ArchetypeEdge {
    pub add: Option<ArchetypeId>,
    pub remove: Option<ArchetypeId>,
}
impl ArchetypeEdge {
    fn old_to_new(new_archetype: ArchetypeId) -> Self {
        Self {
            add: Some(new_archetype),
            remove: None
        }
    }

    fn new_to_old(old_archetype: ArchetypeId) -> Self {
        Self {
            add: None,
            remove: Some(old_archetype)
        }
    }
}

//the primary datastructure in an ECS
pub struct Archetype {
    pub id: ArchetypeId, //helpful for early printing
    pub types: Type,
    pub current_row: usize,
    pub components: Vec<BoxedComponentArray>,
    pub edges: HashMap<ComponentId, ArchetypeEdge>,
}

impl Archetype {
    pub fn new(id: ArchetypeId, types: Type) -> Self {
        Self {
            id,
            types,
            current_row: 0,
            components: Vec::new(),
            edges: HashMap::new()
        }
    }

    pub fn add_edge_to_old(&mut self, component_id: ComponentId, old_archetype_id: ArchetypeId){
        self.edges.insert(component_id, ArchetypeEdge::new_to_old(old_archetype_id));
    }
    pub fn add_edge_to_new(&mut self, component_id: ComponentId, new_archetype_id: ArchetypeId){
        self.edges.insert(component_id, ArchetypeEdge::old_to_new(new_archetype_id));
    }
    /*
    fn insert_components(components: Iter<BoxedComponents>) -> usize {

    }
    */
}


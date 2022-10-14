use std::collections::HashMap;

use super::component_array::ComponentArray;

pub type ComponentId = u64;
pub type ArchetypeId = usize;
pub type EntityId = usize;
pub type Type = Vec<ComponentId>;

pub struct ArchetypeEdge {
    pub add: Option<ArchetypeId>,
    pub remove: Option<ArchetypeId>,
}
impl ArchetypeEdge {
    fn old_to_new(new_archetype: ArchetypeId) -> Self {
        Self {
            add: Some(new_archetype),
            remove: None,
        }
    }

    fn new_to_old(old_archetype: ArchetypeId) -> Self {
        Self {
            add: None,
            remove: Some(old_archetype),
        }
    }
}

//the primary datastructure in an ECS
pub struct Archetype {
    pub id: ArchetypeId, //helpful for early printing
    pub types: Type,

    //these two data structures create the core of an archetype, as a 2d hashable map
    entity_map: HashMap<EntityId, usize>,
    //this can be a vec, because we don't ever really have to search this component array for by component_id
    pub component_arrays: Vec<(ComponentId, ComponentArray)>,

    pub edges: HashMap<ComponentId, ArchetypeEdge>,

    next_row: usize,
}

impl Archetype {
    pub fn new(id: ArchetypeId, types: Type, sizes: &[usize]) -> Self {
        //initialize component arrays with size from types
        assert!(types.len() == sizes.len());
        let mut component_arrays = types
            .iter()
            .zip(sizes.iter())
            .map(|(component_id, size)| (*component_id, ComponentArray::new(*size)))
            .collect();

        Self {
            id,
            types,
            entity_map: HashMap::new(),
            next_row: 0,
            component_arrays,
            edges: HashMap::new(),
        }
    }

    pub fn add_edge_to_old(&mut self, component_id: ComponentId, old_archetype_id: ArchetypeId) {
        self.edges
            .insert(component_id, ArchetypeEdge::new_to_old(old_archetype_id));
    }
    pub fn add_edge_to_new(&mut self, component_id: ComponentId, new_archetype_id: ArchetypeId) {
        self.edges
            .insert(component_id, ArchetypeEdge::old_to_new(new_archetype_id));
    }

    //this assumes that the components are in the order they appear on the archetype
    pub fn add_entity(&mut self, entity: EntityId, components: &[&[u8]]) {
        assert!(components.len() == self.types.len());
        for (i, component) in components.iter().enumerate() {
            self.component_arrays[i].1.push_bytes(*component);
        }
        //add this entity to our entity map
        self.entity_map.insert(entity, self.next_row);
        self.next_row += 1;
    }

    pub fn remove_entity(&mut self, entity: EntityId) -> HashMap<ComponentId, Vec<u8>> {
        let mut entity_series = HashMap::new();
        //get the row
        let entity_row = self
            .entity_map
            .get(&entity)
            .expect("Entity not found in archetype");

        for (component_id, component_array) in self.component_arrays.iter_mut() {
            let bytes = component_array.remove_bytes(*entity_row).expect("Invalid entity pointer in entity removal");
            entity_series.insert(*component_id, bytes);
        }

        self.entity_map.remove(&entity).unwrap();
        self.next_row -= 1;
        entity_series
    }
}

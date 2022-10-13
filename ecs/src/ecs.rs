use unique_type_id::UniqueTypeId;
use core::arch;
use std::collections::HashMap;

use crate::archetype::{ArchetypeId, ComponentId, Type, Archetype};

type EntityId = usize;

struct Record {
    archetype: ArchetypeId,
    row: usize
}

struct ComponentRecord {
    column: usize,
}

type ArchetypeMap = HashMap<ArchetypeId, ComponentRecord>;

//a fancy list of archetypes
pub struct ECS {
    archetypes: Vec<Archetype>,

    entity_index: HashMap<EntityId, Record>,
    archetype_index: HashMap<Type, ArchetypeId>,
    component_index: HashMap<ComponentId, ArchetypeMap>,

    highest_entity_id: EntityId,
    next_archetype_id: ArchetypeId,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),

            entity_index: HashMap::new(),
            archetype_index: HashMap::new(),
            component_index: HashMap::new(),

            highest_entity_id: 0,
            next_archetype_id: 0,
        }
    }

    //creates a new archetype if it doesn't exist, but doesn't touch it
    fn get_archetype(&mut self, components: Type) -> ArchetypeId {
        //either create the empty archetype or reference it
        if let Some(archetype) = self.archetype_index.get(&components) {
            *archetype
        } else {
            //construct the new archetype here
            let archetype_id = self.next_archetype_id;
            self.archetypes.push(Archetype::new(archetype_id, components.clone()));
            self.next_archetype_id = self.archetypes.len();

            self.archetype_index.insert(components, archetype_id);

            archetype_id
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        self.highest_entity_id += 1;
        let empty_archetype = self.get_archetype(Vec::new());

        //since empty archetypes don't have components we can cheat and use 0 for the row
        //when creating entities of other archetypes we need to get the row
        self.entity_index
            .insert(self.highest_entity_id, Record { archetype: empty_archetype, row: 0});
        self.highest_entity_id
    }

    pub fn add_component<T>(&mut self, entity: &EntityId, component: T) 
    where
        T: UniqueTypeId<u64>
    {
        //look up the archetype
        let record = self.entity_index.get(&entity).expect("Adding component to null entity");
        //TODO: maybe check if this type already has this component, but this is slow
        //transition along edge
        let new_component_id: ComponentId = T::id().0;

        let old_archetype_id = record.archetype;
        //get the new archetype_id by traversing edges, or creating edges/creating archetypes
        let new_archetype_id = if let Some(archetype_edge) = self.archetypes[record.archetype].edges.get(&new_component_id) {
            //transition here
            archetype_edge.add.expect("An edge is defined and this entity already has this component")
        } else {
            //get new_components from old components + new_component_id
            let new_components = {
                let old_archetype = &mut self.archetypes[old_archetype_id];
                let mut new_components = old_archetype.types.clone();
                new_components.push(new_component_id);
                new_components
            };
            let new_archetype_id = self.get_archetype(new_components);

            //borrow each archetype one after the other and update its edges
            {
                let new_archetype = &mut self.archetypes[new_archetype_id];
                new_archetype.add_edge_to_old(new_component_id, old_archetype_id)
            }
            {
                let old_archetype = &mut self.archetypes[old_archetype_id];
                old_archetype.add_edge_to_new(new_component_id, new_archetype_id);
            }

            new_archetype_id
        };

    }
}

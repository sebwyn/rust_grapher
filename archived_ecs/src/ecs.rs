use core::slice;
use std::{collections::HashMap, intrinsics::size_of, iter::empty};
use unique_type_id::UniqueTypeId;

use crate::archetype::{Archetype, ArchetypeId, ComponentId, EntityId, Type};

//for mapping components to where they lie on the relevant archetypes
type ArchetypeMap = HashMap<ArchetypeId, u64>;

//a helper function for converting structs to byte slices
fn to_bytes<'a, T>(component: &T) -> &'a [u8] {
    unsafe {
        slice::from_raw_parts(component as *const T as *const u8, size_of::<T>())
    }
}

//a fancy list of archetypes
pub struct ECS {
    archetypes: Vec<Archetype>,

    entity_index: HashMap<EntityId, ArchetypeId>, //map entities to their archetype and position within archetype
    archetype_index: HashMap<Type, ArchetypeId>, //map lists of components to the respective archetype
    component_index: HashMap<ComponentId, ArchetypeMap>, //map components to maps of archetyps and columns

    //map components to their size for archetype creation and such
    component_size_index: HashMap<ComponentId, usize>,

    next_entity_id: EntityId,
    next_archetype_id: ArchetypeId,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),

            entity_index: HashMap::new(),
            archetype_index: HashMap::new(),
            component_index: HashMap::new(),

            component_size_index: HashMap::new(),

            next_entity_id: 0,
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
            //construct a vector of ids here
            let sizes: Vec<usize> = components
                .iter()
                .map(|f| {
                    *self
                        .component_size_index
                        .get(f)
                        .expect("Component doesn't have an id")
                })
                .collect();

            let archetype_id = self.next_archetype_id;
            self.archetypes
                .push(Archetype::new(archetype_id, components.clone(), &sizes));
            self.next_archetype_id = self.archetypes.len();

            self.archetype_index.insert(components, archetype_id);

            archetype_id
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        let entity_id = self.next_entity_id;
        let empty_archetype = self.get_archetype(Vec::new());
        self.archetypes[empty_archetype].add_entity(entity_id, &[]);
        self.entity_index
            .insert(entity_id, empty_archetype);
        
        self.next_entity_id += 1;
        entity_id
    }

    pub fn add_component<T>(&mut self, entity: EntityId, component: T)
    where
        T: UniqueTypeId<u64>,
    {
        //this will do this insert every time, wasting precious cpu :(
        let new_component_id: ComponentId = T::id().0;
        self.component_size_index
            .insert(new_component_id, size_of::<T>());

        //look up the archetype
        let old_archetype_id = *self
            .entity_index
            .get(&entity)
            .expect("Adding component to null entity");

        //get the new archetype_id by traversing edges, or creating edges/creating archetypes
        let new_archetype_id = if let Some(archetype_edge) = self.archetypes[old_archetype_id]
            .edges
            .get(&new_component_id)
        {
            //transition here
            archetype_edge
                .add
                .expect("An edge is defined and this entity already has this component")
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

        //perform the copy of data from old to new
        let old_components = self.archetypes[old_archetype_id].remove_entity(entity);
        //construct an object we can pass to the new archetype so that it can instantiate
        assert!(old_components.len() + 1 == self.archetypes[new_archetype_id].types.len());
        //this assumption means that we should be able to fill the missing component with the new component
        let new_components: Vec<&[u8]> = self.archetypes[new_archetype_id].types.iter().map(|component_id| {
            match old_components.get(component_id) {
                Some(bytes) => { bytes.as_slice() },
                None => to_bytes(&component)
            }
        }).collect();

        self.archetypes[new_archetype_id].add_entity(entity, &new_components);
        self.entity_index.insert(entity, new_archetype_id);
    }

}

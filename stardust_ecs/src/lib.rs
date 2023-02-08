#[macro_use] extern crate specs;
use specs::prelude::*;
use indexmap::IndexMap;
use std::sync::Arc;

use ecs_derive::EngineComponent;

use stardust_common::math::*;

mod fields;
pub use fields::*;

mod transform;
pub use transform::*;

mod model;
pub use model::*;

pub mod prelude;

/// Use shift_remove to remove components!
pub type ComponentMap = IndexMap<String, Box<dyn EngineComponent>>;

#[derive(Debug, Component, Clone, EngineComponent)]
#[storage(VecStorage)]
pub struct CompName {
    #[editable]
    pub name: String,
}

impl CompName {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }
}

#[derive(Debug)]
pub struct SceneSettings {
    pub voxels_per_meter: f32, // For scene scale
}

impl SceneSettings {
    pub fn new() -> Self {
        Self {
            voxels_per_meter: 1.0, // 16 voxels per meter
        }
    }
}

pub enum EntityType {
    Entity(Entity),
    Camera,
    Light,
}

pub struct EntityInfo {
    pub name: String,
    pub kind: EntityType,
}

pub struct EntityComponentInfo {
    pub entity: Entity,
    pub components: ComponentMap,
}

pub struct Scene {
    world: World,

    settings: SceneSettings,
}

impl Scene {
    pub fn new() -> Self {
        let mut world = World::new();
        world.register::<CompName>();
        world.register::<CompTransform>();
        world.register::<CompModel>();
        Self {
            world: world,

            settings: SceneSettings::new(),
        }
    }

    pub fn create_entity<S: Into<String>, F: Fn(EntityBuilder) -> EntityBuilder>(&mut self, name: S, f: F) {
        f(self.world.create_entity().with(CompName::new(name.into()))).build();
    }

    /// dt = deltatime in seconds
    /// This function first runs all engine systems, then moves to user systems, and finally runs
    /// all user scripts.
    pub fn update(&mut self, dt: f32) {
        // Update all model positions
        let mut sys_transpos_mod_vpos_update = TransformPosModelVPosUpdate { voxels_per_meter: self.settings.voxels_per_meter };
        sys_transpos_mod_vpos_update.run_now(&mut self.world);

        self.world.maintain();
    }

    pub fn update_dirty_models(&mut self, voxel_world: &stardust_world::World) {
        let mut sys_update_dirty_models = DirtyModelsUpdate { voxel_world };
        sys_update_dirty_models.run_now(&mut self.world);
    }

    pub fn entity_list(&mut self) -> Vec<EntityInfo> {
        let mut info = Vec::new();

        {
            let entity_storage = self.world.entities();
            let name_storage = self.world.read_storage::<CompName>();
            for (entity, cname) in (&entity_storage, &name_storage).join() {
                info.push(
                    EntityInfo {
                        name: cname.name.clone(),
                        kind: EntityType::Entity(entity),
                    }
                );
            }
        }

        info
    }

    pub fn entity_is_alive(&self, entity: Entity) -> bool {
        self.world.is_alive(entity)
    }

    // TODO: Check if entity is still alive
    pub fn entity_component_list(&mut self, entity: Entity) -> EntityComponentInfo {
        // Storages for each component
        // let name_storage = self.world.read_storage::<CompName>();
        // let transform_storage = self.world.read_storage::<CompTransform>();
        // let model_storage = self.world.read_storage::<CompModel>();
        //
        // let mut components: ComponentMap = ComponentMap::new();
        //
        // if let Some(comp) = name_storage.get(entity) {
        //     components.insert(String::from("Name"), Box::new(comp.clone()));
        // }
        //
        // if let Some(comp) = transform_storage.get(entity) {
        //     components.insert(String::from("Transform"), Box::new(comp.clone()));
        // }
        //
        // if let Some(comp) = model_storage.get(entity) {
        //     components.insert(String::from("Model"), Box::new(comp.clone()));
        // }

        let mut components: ComponentMap = ComponentMap::new();
        read::<CompName>(&self.world, entity, &mut components);
        // read::<CompTransform>(&self.world, entity, &mut components);
        // read::<CompModel>(&self.world, entity, &mut components);

        EntityComponentInfo {
            entity,
            components,
        }
    }

    // TODO: Check if entity is still alive
    pub fn entity_upload_component_list(&mut self, entity: Entity, comp_info: &EntityComponentInfo) {
        for (_name, component) in &comp_info.components {
            component.write(&mut self.world, comp_info.entity);
        }
    }
}

struct DirtyModelsUpdate<'w> {
    voxel_world: &'w stardust_world::World,
}

impl <'a, 'w> System<'a> for DirtyModelsUpdate<'w> {
    type SystemData = WriteStorage<'a, CompModel>;

    fn run(&mut self, mut cmodel: Self::SystemData) {
        for model in (&mut cmodel).join() {
            if model.dirty {
                if let Some(next_model_ref) = &model.next_model {
                    if let Some(model_ref) = &model.model_ref {
                        self.voxel_world.update_model(Arc::clone(model_ref), model.prev_vox_pos, model.vox_pos, true);
                    }
                    self.voxel_world.update_model(Arc::clone(next_model_ref), model.prev_vox_pos, model.vox_pos, false);
                    model.update_model_ref();
                } else {
                    if let Some(model_ref) = &model.model_ref {
                        self.voxel_world.update_model(Arc::clone(model_ref), model.prev_vox_pos, model.vox_pos, false);
                    }
                }
                model.dirty = false;
            }
        }
    }
}

struct TransformPosModelVPosUpdate {
    voxels_per_meter: f32,
}

impl<'a> System<'a> for TransformPosModelVPosUpdate {
    type SystemData = (WriteStorage<'a, CompModel>, ReadStorage<'a, CompTransform>);

    fn run(&mut self, (mut cmodel, ctransform): Self::SystemData) {
        const WORLD_SIZE_HALF: UVec3 = uvec3(0,0,0);

        // join() combines the iterators, so we only iterate the objects with both components
        for (model, transform) in (&mut cmodel, &ctransform).join() {
            let scaled_pos = transform.position * self.voxels_per_meter;
            let vox_pos = scaled_pos.as_uvec3() + WORLD_SIZE_HALF;
            model.update_voxel_position(vox_pos);
        }
    }
}

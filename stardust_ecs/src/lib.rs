#[macro_use] extern crate specs;
use specs::prelude::*;

use std::collections::BTreeMap;
use std::sync::Arc;

use stardust_common::math::*;

mod fields;
pub use fields::*;

mod transform;
pub use transform::*;

mod model;
pub use model::*;

pub mod prelude;

#[derive(Debug, Component, Clone)]
#[storage(VecStorage)]
pub struct CompName(pub String);
impl EngineComponent for CompName {
    fn fields(&mut self) -> BTreeMap<String, (bool, Value)> {
        let mut map = BTreeMap::new();
        map.insert(String::from("Name"), (true, Value::String(&mut self.0)));
        map
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

#[derive(Clone)]
pub struct EntityComponentInfo {
    pub entity: Entity,
    pub name_component: CompName,
    pub transform_component: Option<CompTransform>,
    pub model_component: Option<CompModel>,
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
        f(self.world.create_entity().with(CompName(name.into()))).build();
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
            for (entity, name) in (&entity_storage, &name_storage).join() {
                info.push(
                    EntityInfo {
                        name: name.0.clone(),
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
    // TODO: Optimise this function, seems like it won't scale very well
    pub fn entity_component_list(&mut self, entity: Entity) -> EntityComponentInfo {
        // Storages for each component
        let name_storage = self.world.read_storage::<CompName>();
        let transform_storage = self.world.read_storage::<CompTransform>();
        let model_storage = self.world.read_storage::<CompModel>();

        EntityComponentInfo {
            entity: entity,
            name_component: name_storage.get(entity).unwrap().clone(),
            transform_component: transform_storage.get(entity).map(|c| c.clone()),
            model_component: model_storage.get(entity).map(|c| c.clone()),
        }
    }

    // TODO: Check if entity is still alive
    // TODO: Optimise this function, seems like it won't scale very well
    pub fn entity_upload_component_list(&mut self, entity: Entity, comp_info: EntityComponentInfo) {
        // Storages for each component
        let mut name_storage = self.world.write_storage::<CompName>();
        let mut transform_storage = self.world.write_storage::<CompTransform>();
        let mut model_storage = self.world.write_storage::<CompModel>();

        if let Some(cname) = name_storage.get_mut(entity) {
            cname.0 = comp_info.name_component.0;
        }

        if let Some(ctransform) = comp_info.transform_component {
            if let Some(cur_ctransform) = transform_storage.get_mut(entity) {
                *cur_ctransform = ctransform;
            } else {
                transform_storage.insert(entity, ctransform).expect("Failed to add component!");
            }
        }

        if let Some(cmodel) = comp_info.model_component {
            if let Some(cur_cmodel) = model_storage.get_mut(entity) {
                *cur_cmodel = cmodel;
            } else {
                model_storage.insert(entity, cmodel).expect("Failed to add component!");
            }
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
                if let Some(model_ref) = &model.model_ref {
                    self.voxel_world.update_model(Arc::clone(model_ref), model.prev_vox_pos, model.vox_pos);
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

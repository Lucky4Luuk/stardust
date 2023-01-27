use specs::prelude::*;

use stardust_common::math::*;

mod transform;
pub use transform::*;

mod model;
pub use model::*;

#[derive(Debug)]
pub struct CompName(String);
impl Component for CompName {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
pub struct SceneSettings {
    pub voxels_per_meter: f32, // For scene scale
}

impl SceneSettings {
    pub fn new() -> Self {
        Self {
            voxels_per_meter: 16.0, // 16 voxels per meter
        }
    }
}

pub enum EntityType {
    Entity,
    Camera,
    Light,
}

pub struct EntityInfo {
    pub name: String,
    pub kind: EntityType,
}

pub struct Scene {
    world: World,
    entities: Vec<Entity>,

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
            entities: Vec::new(),

            settings: SceneSettings::new(),
        }
    }

    pub fn create_entity<F: Fn(EntityBuilder) -> EntityBuilder>(&mut self, name: String, f: F) {
        self.entities.push(f(self.world.create_entity().with(CompName(name))).build());
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

    pub fn entity_list(&mut self) -> Vec<EntityInfo> {
        let mut info = Vec::new();

        {
            let mut entity_info_list_gather = EntityInfoListGather {
                info: &mut info
            };

            entity_info_list_gather.run_now(&mut self.world);
        }

        info
    }
}

struct EntityInfoListGather<'i> {
    info: &'i mut Vec<EntityInfo>,
}

impl<'a, 'i> System<'a> for EntityInfoListGather<'i> {
    type SystemData = ReadStorage<'a, CompName>;

    fn run(&mut self, cname: Self::SystemData) {
        for name in cname.join() {
            self.info.push(EntityInfo {
                name: name.0.clone(),
                kind: EntityType::Entity,
            });
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

// Demo project / test project

use stardust_common::voxel::Voxel;
use stardust_world::*;
use stardust_engine_lib::*;

pub struct App {

}

impl VoxelApp for App {
    fn new(ctx: &Context, engine: &mut EngineInternals) -> Self {
        let mv_model = stardust_magica_voxel::MagicaVoxelModel::from_bytes(include_bytes!("../../gamedata/models/monu10.vox")).unwrap();
        let model = mv_model.to_sdvx();
        let gpu_model = std::sync::Arc::new(GpuModel::from_model(ctx, String::from("Monu10"), &model));
        engine.world.register_model(gpu_model.clone());

        // engine.current_scene.create_entity("Test", |entity| {
        //     let ctrans = CompTransform::new();
        //     let mut cmodel = CompModel::new();
        //     cmodel.next_model = Some(gpu_model.clone());
        //     cmodel.dirty = true;
        //     entity
        //     .with(ctrans)
        //     .with(cmodel)
        // });

        // for x in 0..8 {
        //     for y in 0..4 {
        //         for z in 0..8 {
        //             engine.current_scene.create_entity("Test", |entity| {
        //                 let mut ctrans = CompTransform::new();
        //                 ctrans.position = vec3(x as f32 * 96.0, y as f32 * 128.0, z as f32 * 96.0);
        //                 let mut cmodel = CompModel::new();
        //                 cmodel.next_model = Some(gpu_model.clone());
        //                 cmodel.dirty = true;
        //                 entity
        //                 .with(ctrans)
        //                 .with(cmodel)
        //             });
        //         }
        //     }
        // }

        for x in 0..256 {
            for y in 0..256 {
                for z in 0..256 {
                    let cx = x as i32 - 128;
                    let cy = y as i32 - 128;
                    let cz = z as i32 - 128;
                    let rr = cx*cx + cy*cy + cz*cz;
                    if rr < 128*128 {
                        engine.world.set_voxel(Voxel::new([x as u8, y as u8, z as u8], 255, 0, false, 255), uvec3(1024 + x, 1024 + y, 1024 + z));
                    }
                }
            }
        }

        for x in 0..32 {
            for y in 0..32 {
                for z in 0..32 {
                    let cx = x as i32 - 16;
                    let cy = y as i32 - 16;
                    let cz = z as i32 - 16;
                    let rr = cx*cx + cy*cy + cz*cz;
                    if rr < 16*16 {
                        engine.world.set_voxel(Voxel::new([255; 3], 255, 255, false, 255), uvec3(1024 + x, 1024 + y, 1024 + z));
                    }
                }
            }
        }

        Self {

        }
    }

    fn update(&mut self, input: &Input, engine: &mut EngineInternals) {}
}

fn main() {
    run_app::<App>()
}

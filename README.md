# Stardust engine
A voxel engine written in Rust + OpenGL

## TODO
### In general
- stardust_runner // Engine stripped down to its basics, to run the game
- Custom voxel model format // .sdvx models, custom format, uses our voxel format but not specific for stardust
- Model parsing // Parsing .vox and .sdvx models

### Workflow related
- Voxel brush
- Model brush
- UI overhaul // The UI is pretty awful right now

### Engine
- Resource storage for things like models and sounds, to be referenced by components

### ECS
- Model Component should keep track of the model used, and voxels affected by it existing
- Scriptable Component

### Model parsing
- Define custom voxel format
- Define custom scene format
- Generic parsing frontend to turn any support format into our custom voxel format

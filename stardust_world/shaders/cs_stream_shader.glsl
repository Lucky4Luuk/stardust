#version 450
#define BRICK_MAP_SIZE 128

layout (local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

struct VoxelWithPos {
    uvec4 pos;
    uint voxel;
};

layout(std430, binding = 0) buffer stream_buffer {
    VoxelWithPos voxels[2048];
};

struct Brick {
    uint voxels[16*16*16];
};

layout(std430, binding = 1) buffer brick_pool {
    Brick bricks[1024];
};

layout(std430, binding = 2) buffer brick_map {
    // Offset by 1, so 0 means not allocated
    uint brick_pool_indices[BRICK_MAP_SIZE*BRICK_MAP_SIZE*BRICK_MAP_SIZE];
};

void main() {
    uint stream_idx = gl_GlobalInvocationID.x; //TODO: More sensible ID? Perhaps x+y+z-esque
    VoxelWithPos vwp = voxels[stream_idx];
    // Get brick pos
    uvec3 world_pos = vwp.pos.xyz;
    uvec3 brick_pos = world_pos / 16;
    uvec3 brick_local_pos = world_pos % 16;
    // Get brick index and voxel index
    uint brick_map_idx = brick_pos.x + brick_pos.y * BRICK_MAP_SIZE + brick_pos.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    uint voxel_idx = brick_local_pos.x + brick_local_pos.y * 16 + brick_local_pos.z * 16 * 16;
    // Check if brick is allocated already
    // If so, modify that brick. Otherwise, find unused brick from pool and use it
    uint brick_pool_idx = brick_pool_indices[brick_map_idx];
    if (brick_pool_idx != 0) {
        //Brick is already allocated!
        bricks[brick_pool_idx-1].voxels[voxel_idx] = vwp.voxel;
    }
}

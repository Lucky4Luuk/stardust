#version 450
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

#define BRICK_MAP_SIZE 64
#define BRICK_SIZE 16
#define LAYER0_SIZE 16

#define BRICK_POOL_SIZE 32768
#define LAYER0_POOL_SIZE 8192

struct Brick {
    uint voxels[16*16*16];
};

struct Layer0Node {
    uint brick_idx[16*16*16];
};

layout(std430, binding = 0) buffer brick_pool {
    Brick bricks[];
};

layout(std430, binding = 1) buffer layer0_pool {
    Layer0Node layer0_nodes[];
};

layout(std430, binding = 2) buffer brick_map {
    // Offset by 1, so 0 means not allocated
    uint layer0_pool_indices[];
};

layout(std430, binding = 3) buffer voxel_queue {
    uvec4 voxels[];
};

layout(std430, binding = 4) buffer dealloc_queue {
    // xyz = edit_pos
    // w = kind (0 = brick, 1 = layer0_node)
    uvec4 dealloc_targets[];
};

layout(binding = 5) uniform atomic_uint dealloc_queue_counter;

void setVoxelInternal(ivec3 pos, uint voxel, uint brick_pool_idx) {
    ivec3 local_pos = ivec3(pos);
    int voxel_idx = local_pos.x + local_pos.y * 16 + local_pos.z * 16 * 16;
    if (voxel_idx < 0) return;
    bricks[brick_pool_idx - 1].voxels[voxel_idx] = voxel;
}

bool getBrick(ivec3 pos, uint layer0_pool_idx, out uint brick_pool_idx) {
    ivec3 p = pos;
    int layer0_idx = p.x + p.y * LAYER0_SIZE + p.z * LAYER0_SIZE * LAYER0_SIZE;
    if (layer0_idx < 0) return false;
    brick_pool_idx = layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx];
    if (brick_pool_idx == 0) return false;
    return true;
}

bool getLayer0(ivec3 pos, out uint layer0_pool_idx) {
    ivec3 p = pos;
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return false;
    layer0_pool_idx = layer0_pool_indices[brick_map_idx];
    if (layer0_pool_idx == 0) return false;
    return true;
}

bool setVoxel(ivec3 wpos, uint voxel) {
    ivec3 layer0Pos = ivec3(floor(wpos / float(LAYER0_SIZE) / float(BRICK_SIZE)));
    ivec3 brickPos = ivec3(floor(wpos / float(BRICK_SIZE)));
    ivec3 voxelPos = ivec3(floor(wpos)) % BRICK_SIZE;

    uint brick_pool_idx = 0;
    uint layer0_pool_idx = 0;

    if (getLayer0(layer0Pos, layer0_pool_idx)) {
        if (getBrick(brickPos % LAYER0_SIZE, layer0_pool_idx, brick_pool_idx)) {
            setVoxelInternal(voxelPos, voxel, brick_pool_idx);
            return true;
        }
    }
    return false;
}

void main() {
    uvec4 voxel = voxels[gl_GlobalInvocationID.x];
    ivec3 pos = ivec3(voxel.xyz);
    uint raw = voxel.w;

    bool has_placed = setVoxel(pos, raw);

    if (has_placed && raw == 0) {
        // Add layer0_node and brick to potential deallocation targets
        uint i = atomicCounterIncrement(dealloc_queue_counter) + 1;
        dealloc_targets[i] = uvec4(voxel.xyz, 0); // Brick
        i = atomicCounterIncrement(dealloc_queue_counter) + 1;
        dealloc_targets[i] = uvec4(voxel.xyz, 1); // Layer0
    }
}

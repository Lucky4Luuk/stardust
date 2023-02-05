#version 460
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

#define BRICK_MAP_SIZE 64
#define BRICK_SIZE 16
#define LAYER0_SIZE 16

#define BRICK_POOL_SIZE 32768
#define LAYER0_POOL_SIZE 8192

struct Brick {
    uint voxels[16*16*16 + 4];
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

layout(binding = 3) uniform atomic_uint dealloc_counter;

layout(std430, binding = 4) buffer free_brick_pool {
    uint free_brick_indices[];
};

layout(binding = 5) uniform atomic_uint brick_pool_counter;

bool brickEmpty(uint brick_pool_idx) {
    for (int i = 0; i < 16*16*16; i++) {
        if (bricks[brick_pool_idx - 1].voxels[i] > 0) return false;
    }
    return true;
}

bool getBrick(ivec3 pos, uint layer0_pool_idx, out uint brick_pool_idx, out uint layer0_idx) {
    ivec3 p = pos;
    int signed_layer0_idx = p.x + p.y * LAYER0_SIZE + p.z * LAYER0_SIZE * LAYER0_SIZE;
    if (signed_layer0_idx < 0) return false;
    layer0_idx = uint(signed_layer0_idx);
    brick_pool_idx = layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx];
    if (brick_pool_idx == 0) return false;
    return true;
}

bool getLayer0(ivec3 pos, out uint layer0_pool_idx, out uint brick_map_idx) {
    ivec3 p = pos;
    int signed_brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (signed_brick_map_idx < 0) return false;
    brick_map_idx = uint(signed_brick_map_idx);
    layer0_pool_idx = layer0_pool_indices[brick_map_idx];
    if (layer0_pool_idx == 0) return false;
    return true;
}

void main() {
    // This shader will go through all bricks in the pool and check if they are in use and empty.
    // If they are, they get deallocated.

    uint brick_pool_idx = atomicCounterIncrement(dealloc_counter) % BRICK_POOL_SIZE;
    bricks[brick_pool_idx - 1].voxels[4099] = 0;
    if (bricks[brick_pool_idx - 1].voxels[4098] > 0) {
        if (brickEmpty(brick_pool_idx)) {
            // uint write_idx = atomicCounterIncrement(brick_pool_counter);
            // if (write_idx >= BRICK_POOL_SIZE) return;
            // free_brick_indices[write_idx] = brick_pool_idx - 1;
            bricks[brick_pool_idx - 1].voxels[4099] = 1;
        }
    }

    memoryBarrier();

    // if (bricks[brick_pool_idx - 1].voxels[4099] == 1) {
    //     uint layer0_pool_idx = bricks[brick_pool_idx - 1].voxels[4096];
    //     if (layer0_pool_idx > 0) {
    //         uint l0_idx = bricks[brick_pool_idx - 1].voxels[4097];
    //         layer0_nodes[layer0_pool_idx - 1].brick_idx[l0_idx] = 0;
    //         for (int i = 0; i < 16*16*16 + 4; i++) {
    //             bricks[brick_pool_idx - 1].voxels[i] = 0;
    //         }
    //     }
    // }
}

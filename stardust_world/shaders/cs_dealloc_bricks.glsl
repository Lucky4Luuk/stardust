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

void main() {
    // This shader will go through all bricks in the pool and check if they are in use and empty.
    // If they are, they get deallocated.

    uint brick_pool_idx = (atomicCounterIncrement(dealloc_counter) % BRICK_POOL_SIZE) + 1;

    if (bricks[brick_pool_idx - 1].voxels[4098] > 0) {
        uint layer0_pool_idx = bricks[brick_pool_idx - 1].voxels[4096];
        if (layer0_pool_idx > 0) {
            uint l0_idx = bricks[brick_pool_idx - 1].voxels[4097];
            if (layer0_nodes[layer0_pool_idx - 1].brick_idx[l0_idx] == brick_pool_idx) {
                if (brickEmpty(brick_pool_idx)) {
                    bricks[brick_pool_idx - 1].voxels[4099] += 1;
                } else {
                    bricks[brick_pool_idx - 1].voxels[4099] = 0;
                }

                if (bricks[brick_pool_idx - 1].voxels[4099] > 1) {
                    uint write_idx = atomicCounterIncrement(brick_pool_counter);
                    atomicExchange(free_brick_indices[write_idx], brick_pool_idx);

                    layer0_nodes[layer0_pool_idx - 1].brick_idx[l0_idx] = 0;
                    for (int i = 0; i < 16*16*16 + 4; i++) {
                        bricks[brick_pool_idx - 1].voxels[i] = 0;
                    }
                }
            }
        }
    }
}

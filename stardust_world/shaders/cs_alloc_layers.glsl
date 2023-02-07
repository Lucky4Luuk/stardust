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

layout(std430, binding = 0) coherent buffer brick_pool {
    Brick bricks[];
};

layout(std430, binding = 1) coherent buffer layer0_pool {
    Layer0Node layer0_nodes[];
};

layout(std430, binding = 2) coherent buffer brick_map {
    // Offset by 1, so 0 means not allocated
    uint layer0_pool_indices[];
};

layout(std430, binding = 3) coherent buffer voxel_queue {
    uvec4 voxels[];
};

layout(std430, binding = 4) coherent buffer free_brick_pool {
    uint free_brick_indices[];
};

layout(std430, binding = 5) coherent buffer free_layer0_pool {
    uint free_layer0_indices[];
};

layout(binding = 6) uniform atomic_uint brick_pool_counter;
layout(binding = 7) uniform atomic_uint layer0_pool_counter;

uint findLayer0Empty() {
    uint next_free_idx = atomicCounterDecrement(layer0_pool_counter);
    // This if-statement checks for underflowing
    if (next_free_idx >= LAYER0_POOL_SIZE) {
        atomicCounterExchange(layer0_pool_counter, LAYER0_POOL_SIZE);
        return 0;
    }
    return free_layer0_indices[next_free_idx];
}

void setVoxel(ivec3 wpos) {
    ivec3 layer0Pos = ivec3(floor(wpos / float(LAYER0_SIZE) / float(BRICK_SIZE)));

    ivec3 p = layer0Pos;
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return;

    if (atomicCompSwap(layer0_pool_indices[brick_map_idx], 0, 1) == 0) {
        uint layer0_pool_idx = findLayer0Empty();
        if (layer0_pool_idx > 0) {
            atomicExchange(layer0_pool_indices[brick_map_idx], layer0_pool_idx);
            for (uint i = 0; i < 16*16*16; i++) {
                atomicExchange(layer0_nodes[layer0_pool_idx - 1].brick_idx[i], 0);
            }
        }
    }
}

void main() {
    if (atomicCounter(layer0_pool_counter) > 0) {
        uvec4 voxel = voxels[gl_GlobalInvocationID.x];
        ivec3 wpos = ivec3(voxel.xyz);
        setVoxel(wpos);
    }
}

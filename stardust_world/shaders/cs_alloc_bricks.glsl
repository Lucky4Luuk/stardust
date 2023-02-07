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

uint findBrickEmpty() {
    uint next_free_idx = atomicCounterDecrement(brick_pool_counter); // Returns modified
    if (next_free_idx > BRICK_POOL_SIZE) {
        // Counter has underflowed
        atomicCounterExchange(brick_pool_counter, 0); // Undo the underflow
        return 0;
    }
    uint value = free_brick_indices[next_free_idx];
    atomicExchange(free_brick_indices[next_free_idx], 0);
    return value;
}

void undoBrickAlloc(uint brick_pool_idx) {
    uint write_idx = atomicCounterIncrement(brick_pool_counter); // Returns original
    atomicExchange(free_brick_indices[write_idx], brick_pool_idx);
}

bool getLayer0(ivec3 pos, out uint layer0_pool_idx) {
    ivec3 p = pos;
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return false;
    layer0_pool_idx = layer0_pool_indices[brick_map_idx];
    if (layer0_pool_idx == 0) return false;
    return true;
}

void setVoxel(ivec3 wpos) {
    ivec3 layer0Pos = ivec3(floor(wpos / float(LAYER0_SIZE) / float(BRICK_SIZE)));
    ivec3 brickPos = ivec3(floor(wpos / float(BRICK_SIZE)));
    ivec3 voxelPos = ivec3(floor(wpos)) % BRICK_SIZE;

    uint layer0_pool_idx = 0;

    if (getLayer0(layer0Pos, layer0_pool_idx)) {
        ivec3 bp = brickPos % LAYER0_SIZE;

        ivec3 p = bp;
        int layer0_idx = p.x + p.y * LAYER0_SIZE + p.z * LAYER0_SIZE * LAYER0_SIZE;
        if (layer0_idx < 0) return;

        if (atomicCompSwap(layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx], 0, 1) == 0) {
            uint brick_pool_idx = findBrickEmpty();
            if (brick_pool_idx > 0) {
                atomicExchange(layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx], brick_pool_idx);
                atomicExchange(bricks[brick_pool_idx - 1].voxels[4096], layer0_pool_idx);
                atomicExchange(bricks[brick_pool_idx - 1].voxels[4097], layer0_idx);
                atomicExchange(bricks[brick_pool_idx - 1].voxels[4098], 1);
                atomicExchange(bricks[brick_pool_idx - 1].voxels[4099], 0);
            }
        }
    }
}

void main() {
    if (atomicCounter(brick_pool_counter) > 0) {
        uvec4 voxel = voxels[gl_GlobalInvocationID.x];
        ivec3 wpos = ivec3(voxel.xyz);
        setVoxel(wpos);
    }
}

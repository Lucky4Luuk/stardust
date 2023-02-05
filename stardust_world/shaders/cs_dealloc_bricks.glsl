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

layout(std430, binding = 3) buffer potential_dealloc_queue {
    // xyz = edit_pos
    // w = kind (0 = brick, 1 = layer0_node)
    uvec4 potential_deallocs[];
};

layout(binding = 4) uniform atomic_uint potential_dealloc_queue_counter;

layout(std430, binding = 5) buffer free_brick_pool {
    uint free_brick_indices[];
};

layout(binding = 6) uniform atomic_uint brick_pool_counter;

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
    // This shader will look at all potential deallocations, and keep all
    // valid ones, while also filtering out duplicates.
    //
    // How does it work?
    // For each potential deallocation target, we first check if it's valid.
    // If the deallocation is no longer valid, we skip this deallocation.
    // Otherwise, we may continue. For each deallocation, we append it to a buffer
    // of valid deallocations. We also filter out duplicates.
    // TODO: Filter out duplicates somehow

    if (gl_GlobalInvocationID.x < atomicCounter(potential_dealloc_queue_counter)) {
        uvec4 dealloc = potential_deallocs[gl_GlobalInvocationID.x];
        bool is_brick = (dealloc.w == 0);
        ivec3 wpos = ivec3(dealloc.xyz);
        ivec3 layer0Pos = ivec3(floor(wpos / float(LAYER0_SIZE) / float(BRICK_SIZE)));
        ivec3 brickPos = ivec3(floor(wpos / float(BRICK_SIZE)));
        ivec3 voxelPos = ivec3(floor(wpos)) % BRICK_SIZE;

        uint layer0_pool_idx = 0;
        uint brick_map_idx = 0;
        if (getLayer0(layer0Pos, layer0_pool_idx, brick_map_idx)) {
            if (is_brick) {
                uint brick_pool_idx = 0;
                uint layer0_idx = 0;
                if (getBrick(brickPos % LAYER0_SIZE, layer0_pool_idx, brick_pool_idx, layer0_idx)) {
                    if (brickEmpty(brick_pool_idx)) {
                        layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx] = 0;
                        uint write_idx = atomicCounterIncrement(brick_pool_counter) + 1;
                        if (write_idx >= BRICK_POOL_SIZE) return;
                        free_brick_indices[write_idx] = brick_pool_idx - 1;
                        memoryBarrier();
                    }
                }
            }
        }
    }
}

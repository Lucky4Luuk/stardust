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

struct VoxelWithPos {
    uint voxel;
    uint x;
    uint y;
    uint z;
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
    VoxelWithPos voxels[];
};

void setVoxel(ivec3 pos, uint voxel, uint brick_pool_idx) {
    ivec3 local_pos = ivec3(pos);
    int voxel_idx = local_pos.x + local_pos.y * 16 + local_pos.z * 16 * 16;
    if (voxel_idx < 0) return;
    uint vi = uint(voxel_idx);
    bricks[brick_pool_idx - 1].voxels[vi] = voxel;
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

bool isBrickUsed(uint i) {
    // We can tell if the brick is unused if all its voxels have opacity 0
    for (uint j = 0; j < 16*16*16; j++) {
        uint voxel = bricks[i].voxels[j];
        uint opacity_metalic = (voxel & (0xFF << 24)) >> 24;
        if ((opacity_metalic << 1) != 0) return true;
    }
    return false;
}

uint findBrickEmpty() {
    for (uint i = 0; i < BRICK_POOL_SIZE; i++) {
        if (isBrickUsed(i) == false) {
            return i + 1;
        }
    }
    return 0;
}

void allocBrick(ivec3 pos, uint layer0_pool_idx, out uint brick_pool_idx) {
    ivec3 p = pos;
    int layer0_idx = p.x + p.y * LAYER0_SIZE + p.z * LAYER0_SIZE * LAYER0_SIZE;
    if (layer0_idx < 0) return;

    brick_pool_idx = findBrickEmpty();

    layer0_nodes[layer0_pool_idx - 1].brick_idx[layer0_idx] = brick_pool_idx;
}

bool isLayer0Used(uint i) {
    // We can tell if the node is unused if all its indices are 0
    for (uint j = 0; j < 16*16*16; j++) {
        if (layer0_nodes[i].brick_idx[j] > 0) return true;
    }
    return false;
}

uint findLayer0Empty() {
    // Find the first possible layer0_pool_slot
    for (uint i = 0; i < LAYER0_POOL_SIZE; i++) {
        if (isLayer0Used(i) == false) {
            return i + 1;
        }
    }
    return 0;
}

void allocLayer0(ivec3 pos, out uint layer0_pool_idx) {
    ivec3 p = pos;
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return;

    layer0_pool_idx = findLayer0Empty();

    layer0_pool_indices[brick_map_idx] = layer0_pool_idx;
}

void setVoxel(ivec3 wpos, uint voxel) {
    ivec3 layer0Pos = ivec3(floor(wpos / float(LAYER0_SIZE) / float(BRICK_SIZE)));
    ivec3 brickPos = ivec3(floor(wpos / float(BRICK_SIZE)));
    ivec3 voxelPos = ivec3(floor(wpos)) % BRICK_SIZE;

    uint brick_pool_idx = 0;
    uint layer0_pool_idx = 0;

    if (!getLayer0(layer0Pos, layer0_pool_idx)) {
        // Allocate a layer0 node if possible
        allocLayer0(layer0Pos, layer0_pool_idx);
    }

    if (layer0_pool_idx > 0) {
        ivec3 bp = brickPos % LAYER0_SIZE;
        if (!getBrick(bp, layer0_pool_idx, brick_pool_idx)) {
            // TODO: Allocate a brick
            allocBrick(bp, layer0_pool_idx, brick_pool_idx);
        }

        if (brick_pool_idx > 0) {
            setVoxel(voxelPos, voxel, brick_pool_idx);
        }
    }
}

void main() {
    VoxelWithPos voxel = voxels[gl_GlobalInvocationID.x];
    ivec3 pos = ivec3(voxel.x, voxel.y, voxel.z);
    uint raw = voxel.voxel;
    setVoxel(pos, raw);
}

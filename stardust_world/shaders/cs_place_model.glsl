#version 450
layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

#define BRICK_MAP_SIZE 64
#define BRICK_SIZE 16
#define LAYER0_SIZE 16

#define BRICK_POOL_SIZE 32768
#define LAYER0_POOL_SIZE 8192

layout(std430, binding = 0) buffer voxel_queue {
    uvec4 voxels[];
};

layout(std430, binding = 1) buffer model_voxels {
    uvec4 mvoxels[];
};
uniform uint offset;
uniform uvec4 pos; // w = 0 means to remove voxels instead of place them

void main() {
    uint index = gl_GlobalInvocationID.x;
    uvec4 voxel = mvoxels[index + offset];
    voxel.xyz += pos.xyz;
    voxel.w *= pos.w;
    voxels[index] = voxel;
}

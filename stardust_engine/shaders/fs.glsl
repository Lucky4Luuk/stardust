#version 450

in vec2 uv;

out vec4 FragColor;

struct Brick {
    uint voxels[16*16*16];
};

layout(std430, binding = 0) buffer brick_pool {
    Brick bricks[1024];
};

layout(std430, binding = 1) buffer brick_map {
    // Offset by 1, so 0 means not allocated
    uint brick_pool_indices[64*64*64];
};

bool getVoxel(ivec3 p, out vec3 color) {
    ivec3 check_pos = p + 32*16;
    if (check_pos.x < 0.0 || check_pos.y < 0.0 || check_pos.z < 0.0) return false;
    if (check_pos.x >= 64.0*16.0 || check_pos.y >= 64.0*16.0 || check_pos.z >= 64.0*16.0) return false;
    uvec3 world_pos = uvec3(p + 32*16);
    uvec3 brick_pos = world_pos / 16;
    uvec3 local_pos = world_pos % 16;
    uint brick_map_idx = brick_pos.x + brick_pos.y * 64 + brick_pos.z * 64 * 64;
    uint brick_pool_idx = brick_pool_indices[brick_map_idx];
    if (brick_pool_idx == 0) return false;
    uint voxel_idx = local_pos.x + local_pos.y * 16 + local_pos.z * 16 * 16;
    uint opacity_metalic = (bricks[brick_pool_idx - 1].voxels[voxel_idx] & (0xFF << 24)) >> 24;
    if ((opacity_metalic << 1) == 0) return false;
    uint color_rgb565 = bricks[brick_pool_idx - 1].voxels[voxel_idx] & 0xFFFF;
    uint r5 = color_rgb565 & 31;
    uint g6 = (color_rgb565 & (63 << 5)) >> 5;
    uint b5 = (color_rgb565 & (31 << 11)) >> 11;
    uint r = r5 << 3;
    uint g = g6 << 2;
    uint b = b5 << 3;
    color = vec3(float(r) / 255.0, float(g) / 255.0, float(b) / 255.0);
    return true;
}

bool getBrick(ivec3 p, out uint brick_pool_idx) {
    uvec3 world_pos = uvec3(p + 32*16);
    uvec3 brick_pos = world_pos / 16;
    uint brick_map_idx = brick_pos.x + brick_pos.y * 64 + brick_pos.z * 64 * 64;
    brick_pool_idx = brick_pool_indices[brick_map_idx];
    if (brick_pool_idx == 0) return false;
    brick_pool_idx -= 1;
    return true;
}

bool traceBricks(inout vec3 ro, vec3 rd, inout bvec3 mask, out vec3 color) {
    ivec3 mapPos = ivec3(floor(ro + 0.));
    vec3 deltaDist = abs(vec3(length(rd)) / rd);
	ivec3 rayStep = ivec3(sign(rd));
	vec3 sideDist = (sign(rd) * (vec3(mapPos) - ro) + (sign(rd) * 0.5) + 0.5) * deltaDist;

    for (int i = 0; i < 256; i++) {
        if (getVoxel(mapPos, color)) {
            float d = length(vec3(mask) * (sideDist - deltaDist)) / length(rd);
            ro = vec3(mapPos) + rd * d;
            return true;
        }
        mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
        sideDist += vec3(mask) * deltaDist;
		mapPos += ivec3(vec3(mask)) * rayStep;
    }
    return false;
}

void main() {
    FragColor = vec4(1.0, 0.1, 0.2, 1.0);

    vec2 screenPos = uv * 2.0 - 1.0;
    vec3 cameraDir = vec3(0.0, 0.0, 0.8);
    vec3 cameraPlaneU = vec3(1.0, 0.0, 0.0);
	vec3 cameraPlaneV = vec3(0.0, 1.0, 0.0);
    vec3 rayDir = cameraDir + screenPos.x * cameraPlaneU + screenPos.y * cameraPlaneV;
	vec3 rayPos = vec3(0.0, 0.0, -76.0);

    bvec3 mask;
    vec3 color;
    bool hit = traceBricks(rayPos, rayDir, mask, color);
    if (hit) {
        // vec3 color;
        // if (mask.x) {
        //     color = vec3(0.5);
        // }
        // if (mask.y) {
        //     color = vec3(1.0);
        // }
        // if (mask.z) {
        //     color = vec3(0.75);
        // }
        FragColor = vec4(color, 1.0);
    }
}

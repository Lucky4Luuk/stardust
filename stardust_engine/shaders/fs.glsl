#version 450

#define BRICK_MAP_SIZE 128

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
    uint brick_pool_indices[];
};

uniform mat4 invprojview;
uniform vec3 rayPos;

bool getVoxel(ivec3 p, out vec3 color, uint brick_pool_idx) {
    uvec3 world_pos = uvec3(p + (BRICK_MAP_SIZE/2)*16);
    uvec3 local_pos = world_pos % 16;
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
    // ivec3 checked_pos = (p + (BRICK_MAP_SIZE/2)*16) / 16;
    // if (min(checked_pos.x, min(checked_pos.y, checked_pos.z)) < 0 || max(checked_pos.x, max(checked_pos.y, checked_pos.z)) > 63) return false;
    uvec3 world_pos = uvec3(p + (BRICK_MAP_SIZE/2)*16);
    uvec3 brick_pos = world_pos / 16;
    uint brick_map_idx = brick_pos.x + brick_pos.y * BRICK_MAP_SIZE + brick_pos.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    brick_pool_idx = brick_pool_indices[brick_map_idx];
    if (brick_pool_idx == 0) return false;
    return true;
}

bool traceBricks(inout vec3 ro, vec3 rd, inout bvec3 mask, out vec3 color) {
    ivec3 worldPos = ivec3(floor(ro + 0.));
    vec3 deltaDist = abs(vec3(length(rd)) / rd);
	ivec3 rayStep = ivec3(sign(rd));
	vec3 sideDist = (sign(rd) * (vec3(worldPos) - ro) + (sign(rd) * 0.5) + 0.5) * deltaDist;
    uint brick_idx;

    for (int i = 0; i < 1024; i++) {
        if (!getBrick(worldPos, brick_idx)) {
            for (int j = 0; j < 16; j++) {
                mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
                sideDist += vec3(mask) * deltaDist;
                worldPos += ivec3(vec3(mask)) * rayStep;
            }
        } else {
            // Brick found, step through voxel grid
            if (getVoxel(worldPos, color, brick_idx)) return true;
            mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
            sideDist += vec3(mask) * deltaDist;
            worldPos += ivec3(vec3(mask)) * rayStep;
        }
    }
    return false;
}

void main() {
    FragColor = vec4(1.0, 0.1, 0.2, 1.0);

    vec2 pos = uv * 2.0 - 1.0;
	float near = 0.02;
	float far = 512.0;
    vec3 rayPos = rayPos;
    vec3 rayDir = (invprojview * vec4(pos * (far - near), far + near, far - near)).xyz;

    rayDir = normalize(rayDir);

    bvec3 mask;
    vec3 color;
    bool hit = traceBricks(rayPos, rayDir, mask, color);
    if (hit) {
        FragColor = vec4(color, 1.0);
    }
}

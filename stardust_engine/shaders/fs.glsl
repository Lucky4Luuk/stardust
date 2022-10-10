#version 450

#define BRICK_MAP_SIZE 128

in vec2 uv;

out vec4 FragColor;

struct Brick {
    uint voxels[16*16*16];
};

layout(std430, binding = 0) buffer brick_pool {
    Brick bricks[];
};

layout(std430, binding = 1) buffer brick_map {
    // Offset by 1, so 0 means not allocated
    uint brick_pool_indices[];
};

uniform mat4 invprojview;
uniform vec3 rayPos;

bool getVoxel(ivec3 p, out vec3 color, uint brick_pool_idx) {
    ivec3 local_pos = ivec3(p % 16);
    int voxel_idx = local_pos.x + local_pos.y * 16 + local_pos.z * 16 * 16;
    if (voxel_idx < 0) return false;
    uint vi = uint(voxel_idx);
    uint opacity_metalic = (bricks[brick_pool_idx - 1].voxels[vi] & (0xFF << 24)) >> 24;
    if ((opacity_metalic << 1) == 0) return false;
    uint color_rgb565 = bricks[brick_pool_idx - 1].voxels[vi] & 0xFFFF;
    uint r5 = color_rgb565 & 31;
    uint g6 = (color_rgb565 & (63 << 5)) >> 5;
    uint b5 = (color_rgb565 & (31 << 11)) >> 11;
    uint r = r5 << 3;
    uint g = g6 << 2;
    uint b = b5 << 3;
    color = vec3(float(r) / 255.0, float(g) / 255.0, float(b) / 255.0);
    return true;
}

bool getBrick(ivec3 pos, out uint brick_pool_idx) {
    ivec3 p = pos + BRICK_MAP_SIZE / 2;
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return false;
    brick_pool_idx = brick_pool_indices[uint(brick_map_idx)];
    if (brick_pool_idx == 0) return false;
    return true;
}

bool traceVoxels(vec3 ro, vec3 rd, inout bvec3 mask, out vec3 color, uint brick_idx) {
    ivec3 voxelPos = ivec3(floor(ro + 0.));
    vec3 deltaDist = abs(vec3(length(rd)) / rd);
	ivec3 rayStep = ivec3(sign(rd));
	vec3 sideDist = (sign(rd) * (vec3(voxelPos) - ro) + (sign(rd) * 0.5) + 0.5) * deltaDist;

    for (int i = 0; i < 32; i++) {
        if (getVoxel(voxelPos, color, brick_idx)) {
            return true;
        }

        mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
        sideDist += vec3(mask) * deltaDist;
        voxelPos += ivec3(vec3(mask)) * rayStep;
    }

    return false;
}

bool traceBricks(vec3 ro, vec3 rd, inout bvec3 mask, out vec3 color) {
    ivec3 brickPos = ivec3(floor(ro));
    vec3 deltaDist = abs(vec3(length(rd)) / rd);
	ivec3 rayStep = ivec3(sign(rd));
	vec3 sideDist = (sign(rd) * (vec3(brickPos) - ro) + (sign(rd) * 0.5) + 0.5) * deltaDist;
    uint brick_idx;

    for (int i = 0; i < 512; i++) {
        ivec3 brickPosOffset = brickPos + BRICK_MAP_SIZE / 2;
        if (min(brickPosOffset.x, min(brickPosOffset.y, brickPosOffset.z)) < 0 || max(brickPosOffset.x, max(brickPosOffset.y, brickPosOffset.z)) > BRICK_MAP_SIZE) {
            color = vec3(1.0);
            return true;
        }

        if (getBrick(brickPos, brick_idx)) {
            if (traceVoxels(vec3(brickPos) * 16.0 + rd * 0.1, rd, mask, color, brick_idx)) {
                return true;
            }
            // return true;
        }

        mask = lessThanEqual(sideDist.xyz, min(sideDist.yzx, sideDist.zxy));
        sideDist += vec3(mask) * deltaDist;
        brickPos += ivec3(vec3(mask)) * rayStep;
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

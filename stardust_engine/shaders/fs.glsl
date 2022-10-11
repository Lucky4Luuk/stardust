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

bool getVoxel(ivec3 pos, out vec3 color, uint brick_pool_idx) {
    ivec3 local_pos = ivec3(pos % 16);
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
    ivec3 p = pos + ivec3(BRICK_MAP_SIZE / 2);
    int brick_map_idx = p.x + p.y * BRICK_MAP_SIZE + p.z * BRICK_MAP_SIZE * BRICK_MAP_SIZE;
    if (brick_map_idx < 0) return false;
    brick_pool_idx = brick_pool_indices[uint(brick_map_idx)];
    if (brick_pool_idx == 0) return false;
    return true;
}

// Calcs intersection and exit distances, and normal at intersection.
// The ray must be in box/object space.
vec2 boxIntersection(in vec3 ro, in vec3 rd, in vec3 rad)
{
    vec3 m = 1.0/rd;
    vec3 n = m*ro;
    vec3 k = abs(m)*rad;
    vec3 t1 = -n - k;
    vec3 t2 = -n + k;

    float tN = max( max( t1.x, t1.y ), t1.z );
    float tF = min( min( t2.x, t2.y ), t2.z );

    if( tN>tF || tF<0.0) return vec2(-1.0); // no intersection

    return vec2( tN, tF );
}

// Trace at voxel layer, inside a brick
bool traceVoxels(vec3 ro, vec3 rd, uint brick_idx, out vec3 color) {
    // Steps:
    // 1. Get current voxel location
    // 2. Check if voxel exists, yes -> return true
    // 3. Get voxel box intersections, no hits -> return false?
    // 4. Move raypos to exit intersection

    float t = 0.0;

    for (int i = 0; i < 32; i++) {
        vec3 ray_pos = ro + rd * t;
        vec3 voxel_pos = floor(ray_pos);

        if (getVoxel(ivec3(voxel_pos), color, brick_idx)) return true;

        vec2 hit = boxIntersection(ray_pos - voxel_pos, rd, vec3(0.5));
        // No intersection
        if (hit.y < 0.0) return false;

        t += hit.y;
    }

    return false;
}

// Trace at brick layer
bool traceBricks(vec3 ro, vec3 rd, out vec3 color) {
    // Steps:
    // 1. Get current brick location
    // 2. Check if brick is allocated, yes -> traceVoxels()
    // 3. Get brick box intersections, no hits -> return false?
    // 4. Move raypos to exit intersection

    uint brick_idx;
    float t = 0.0;

    for (int i = 0; i < 256; i++) {
        vec3 ray_pos = ro + rd * t;
        vec3 brick_pos = floor(ray_pos / 16.0);

        if (getBrick(ivec3(brick_pos), brick_idx)) {
            if (!traceVoxels(ray_pos, rd, brick_idx, color)) color = vec3(0.0, 0.0, 1.0);
            return true;
        }

        vec2 hit = boxIntersection(ray_pos - brick_pos * 16.0, rd, vec3(8.0));
        // No intersection
        if (hit.y < 0.0) return false;

        t += hit.y;
    }

    return false;
}

bool trace(vec3 ro, vec3 rd, out vec3 color) {
    vec2 hit = boxIntersection(ro, rd, vec3(BRICK_MAP_SIZE / 2));
    if (hit.y < 0.0) return false; // No intersection
    vec3 hit_pos = ro + rd * hit.x;
    if (hit.x < 0.0) hit_pos = ro; // Inside the box already
    if (!traceBricks(hit_pos, rd, color)) color = vec3(0.0, 1.0, 0.0);
    return true;
    // return traceBricks(hit_pos, rd, color);
}

void main() {
    FragColor = vec4(1.0, 0.1, 0.2, 1.0);

    vec2 pos = uv * 2.0 - 1.0;
	float near = 0.02;
	float far = 512.0;
    vec3 rayDir = (invprojview * vec4(pos * (far - near), far + near, far - near)).xyz;
    rayDir = normalize(rayDir);

    bvec3 mask;
    vec3 color = vec3(0.0);
    bool hit = trace(rayPos, rayDir, color);
    if (hit) {
        FragColor = vec4(color, 1.0);
    }
}

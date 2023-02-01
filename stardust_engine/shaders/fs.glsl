#version 450

#define BRICK_MAP_SIZE 64
#define BRICK_SIZE 16
#define LAYER0_SIZE 16

#define pow2(x) (x*x)

in vec2 uv;

out vec4 FragColor;

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

uniform mat4 invprojview;
uniform vec3 rayPos;

bool getVoxel(ivec3 pos, out vec3 color, uint brick_pool_idx) {
    ivec3 local_pos = ivec3(pos);
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

float traceVoxels(vec3 ro, vec3 rd, float tmax, out vec3 normal, out vec3 color, out bool hitsBrick, out bool hitsLayer) {
    float tmax2 = tmax*tmax;

    normal = vec3(0.0, 0.0, 0.0);
	hitsBrick = false;
    hitsLayer = false;

    vec3 gridPos = floor(ro);
    vec3 sideDist = abs(length(rd)/rd);
    vec3 toSide = ((sign(rd) * 0.5 + 0.5) - fract(ro)) / rd;

    float dist;
    vec3 mask;

	uint brick_pool_idx = 0;
    uint layer0_pool_idx = 0;

    for(int i = 0; i < 1024;) {
        ivec3 layer0Pos = ivec3(floor(gridPos / float(LAYER0_SIZE) / float(BRICK_SIZE)));
        ivec3 brickPos = ivec3(floor(gridPos / float(BRICK_SIZE)));
        ivec3 voxelPos = ivec3(floor(gridPos)) % BRICK_SIZE;
        if (getLayer0(layer0Pos, layer0_pool_idx)) {
            hitsLayer = true;
            if (getBrick(brickPos % LAYER0_SIZE, layer0_pool_idx, brick_pool_idx)) {
    			hitsBrick = true;

                if (getVoxel(voxelPos, color, brick_pool_idx)) return dist;

                mask = vec3(lessThanEqual(toSide.xyz, min(toSide.yzx, toSide.zxy)));
                dist = dot(toSide * mask, vec3(1.0));
                normal = mask * -sign(rd);
    			++i;
            } else {
                vec3 toExit = ((sign(rd) * 0.5 + 0.5 + vec3(brickPos)) * float(BRICK_SIZE) - ro) / rd;
                normal = -sign(rd) * vec3(lessThanEqual(toExit.xyz, min(toExit.yzx, toExit.zxy)));
                dist = dot(abs(normal), toExit);
                mask = abs(floor(ro + rd * dist - normal * 0.1) - gridPos);
    			i += max(int(mask.x + mask.y + mask.z), 1);
            }
        } else {
            vec3 toExit = ((sign(rd) * 0.5 + 0.5 + vec3(layer0Pos)) * float(LAYER0_SIZE) * float(BRICK_SIZE) - ro) / rd;
            normal = -sign(rd) * vec3(lessThanEqual(toExit.xyz, min(toExit.yzx, toExit.zxy)));
            dist = dot(abs(normal), toExit);
            mask = abs(floor(ro + rd * dist - normal * 0.1) - gridPos);
            i += max(int(mask.x + mask.y + mask.z), 1);
        }

        toSide += sideDist * mask;
        gridPos += mask * sign(rd);

        float d2 = pow2(gridPos.x - ro.x) + pow2(gridPos.y - ro.y);
        if (d2 > tmax2) return -1.0;
    }

    return -1.0;
}

float trace(vec3 ro, vec3 rd, out vec3 normal, out vec3 color, out bool hitsBrick, out bool hitsLayer, out bool hitsMap) {
    hitsMap = false;
    vec2 hit = boxIntersection(ro - vec3(BRICK_MAP_SIZE / 2) * float(BRICK_SIZE) * float(LAYER0_SIZE), rd, vec3(BRICK_MAP_SIZE / 2) * float(BRICK_SIZE) * float(LAYER0_SIZE));
    if (hit.y < 0.0) return -1.0; // No intersection
    hitsMap = true;
    vec3 hit_pos = ro + rd * hit.x;
    if (hit.x < 0.0) hit_pos = ro; // Inside the box already
	return traceVoxels(hit_pos, rd, hit.y, normal, color, hitsBrick, hitsLayer);
}

void main() {
    FragColor = vec4(0.0, 0.0, 0.0, 1.0);

    vec2 pos = uv * 2.0 - 1.0;
	float near = 0.02;
	float far = 512.0;
    vec3 rayDir = (invprojview * vec4(pos * (far - near), far + near, far - near)).xyz;
    rayDir = normalize(rayDir);

    vec3 color;
	vec3 normal;
	bool hitsBrick;
    bool hitsLayer;
    bool hitsMap;
    float hitDist = trace(rayPos, rayDir, normal, color, hitsBrick, hitsLayer, hitsMap);
    if (hitDist > 0.0) {
        FragColor = vec4(color, 1.0);
    } else if (hitsBrick) {
		FragColor = vec4(0.2, 0.0, 0.0, 1.0);
	} else if (hitsLayer) {
        FragColor = vec4(0.0, 0.2, 0.0, 1.0);
    } else if (hitsMap) {
        FragColor = vec4(0.0, 0.0, 0.2, 1.0);
    }
}

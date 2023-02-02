# Specification file for .sdvx models
Version 0.1

## Contents
1. [Overview](#overview)
2. [Part specification](#part-specification)
3. [Recommendations](#recommendations)

## Overview
Each file contains 1 [model](#model).
Internally, files are stored as [CBOR](https://cbor.io/) data.

## Part specification
### Voxel data
Voxels are stored as unsigned 32 bit integers.
```
Format (bits):
[0-15]  - rgb565
[16-19] - roughness
[20-23] - emissive
[24]    - metallic
[25-31] - opacity
```

### Model
Models are stored as a list of voxels and a list of positions with voxel indices.
This means there is a voxel pool, containing all unique voxels, and a list of [positions with indices](#positionindex) into this pool.

### PositionIndex
Positions with indices are stored as `[u32; 4] = [x, y, z, index]`.
This means that positions cannot be negative!
Duplicate positions are technically allowed, because in practice, they tend to map to the same location, so it won't do anything.

## Recommendations
Here's some advice for those implementing support for SDVX files. Keep in mind none of this is required!
- Sort your voxel list by their raw value before serializing. Simply sort them low to high, based on the internal unsigned 32 bit number representing the voxel. This way you can consistently get the same file, and if everyone does this, get the same file as others.
- Sort your position list by their location, X first, then Y, then Z, 0 to high.

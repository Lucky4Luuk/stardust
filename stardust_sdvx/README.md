# Specification file for .sdvx models
Version 0.1

## Contents
1. [Overview](#overview)
2. [Part specification](#part-specification)

## Overview
Each file contains 1 model.
1. First comes the header, which gets padded to 16 bytes in length.
2. After the header first comes the list of voxels, sized `( voxel_count * voxel_size )`.
3. After the list of voxels comes the list of bricks, which are sized `( (brick_size^3) * index_size )`.

## Part specification
### Header
| Bytes | Type  | Description   |
|-------|-------|---------------|
| 2     | u16   | Major version |
| 2     | u16   | Minor version |
| 2     | u16   | Brick size    |
| 8     | u64   | Voxel count   |

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
Models are stored as a list of bricks and a list of voxels. They are kept separated to support duplicate voxels, so voxels can simply be referenced multiple times.
There is always 1 voxel already defined in the model, which is the empty voxel. In the future, this empty voxel might be used to define certain settings for the model, but this is not yet defined. The space is still taken up however!
The bricks size is variable to allow some headroom for compression vs performance.
More information on the bricks can be found [here](#bricks).

### Bricks
Each brick is defined as a `[brick_size x brick_size x brick_size]` array of voxel indices.
These indices are meant to index into the list of voxels. Index 0 points at the first voxel, index 1 points at the 2nd voxel, etc.
Each index is an unsigned 32 bit integer.

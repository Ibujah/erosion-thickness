# Erosion thickness computation

This is a personal implementation, in rust, of erosion thickness, based on [Erosion Thickness on Medial Axes of 3D Shapes](https://www.cs.wustl.edu/~taoju/research/et.pdf) from Yan et al. I am not an author of this article.

This algorithm requires an input skeleton, which can be computed with this [3D skeletonization method](https://github.com/Ibujah/compact-skel-3d). 

## Instructions

### Installing Rust

This program is writen in Rust. You can install the main tool (cargo) [here](https://www.rust-lang.org/tools/install).

### Build and run

Build instructions:
```
cargo build --release
```

Erosion thickness computation:
```
cargo run --release -- --input_skel ./ressources/skeleton.ply
```

Arguments description:
```
cargo run --release -- --help
```

### Input/Output ply

The input skeleton should be a .ply file, with at least this minimal header:
```
ply
format ascii 1.0
element vertex XX
property float x
property float y
property float z
property float radius
element face XX
property list uchar int vertex_index
end_header
```
Other properties will just be saved in the output ply file.

Two outputs are generated, one is the skeleton with erosion thickness values, with the following minimal header:
```
ply
format ascii 1.0
comment Erosion thickness generated with https://github.com/Ibujah/erosion-thickness
element vertex XX
property uchar green
property float radius
property uchar red
property float erosion_thickness
property float z
property float y
property float x
property uchar blue
element face XX
property list uchar uint vertex_indices
end_header
```
A color for each vertex is added, as a funcion of erosion thickness value.

The second output is the set of erosion paths, using the subdivided skeleton.



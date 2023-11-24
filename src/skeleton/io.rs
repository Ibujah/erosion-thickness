use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use nalgebra::base::*;

use crate::skeleton::skeleton;

pub fn export_to_ply(skel: &skeleton::Skeleton, file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;

    writeln!(file, "ply")?;
    writeln!(file, "format ascii 1.0")?;
    writeln!(file, "element vertex {}", skel.vertices.len())?;
    writeln!(file, "property float x")?;
    writeln!(file, "property float y")?;
    writeln!(file, "property float z")?;
    writeln!(file, "element edge {}", skel.edges.len())?;
    writeln!(file, "property int vertex1")?;
    writeln!(file, "property int vertex2")?;
    writeln!(file, "element face {}", skel.faces.len())?;
    writeln!(file, "property list uchar int vertex_indices")?;
    writeln!(file, "end_header")?;

    for vertex in &skel.vertices {
        writeln!(file, "{} {} {}", vertex[0], vertex[1], vertex[2])?;
    }

    for edge in &skel.edges {
        writeln!(file, "{} {}", edge[0], edge[1])?;
    }

    for face in &skel.faces {
        let mut vertex_indices = Vec::new();
        for &edge_index in face {
            let [v1, v2] = skel.edges[edge_index];
            vertex_indices.push(v1);
            vertex_indices.push(v2);
        }
        vertex_indices.sort();
        vertex_indices.dedup();
        writeln!(
            file,
            "3 {} {} {}",
            vertex_indices[0], vertex_indices[1], vertex_indices[2]
        )?;
    }

    Ok(())
}

pub fn export_to_obj(skel: &skeleton::Skeleton, file_path: &str) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;

    for vertex in &skel.vertices {
        writeln!(file, "v {} {} {}", vertex[0], vertex[1], vertex[2])?;
    }

    for face in &skel.faces {
        let mut vertex_indices = Vec::new();
        for &edge_index in face {
            let [v1, v2] = skel.edges[edge_index];
            vertex_indices.push(v1);
            vertex_indices.push(v2);
        }
        vertex_indices.sort();
        vertex_indices.dedup();
        writeln!(
            file,
            "f {} {} {}",
            vertex_indices[0] + 1,
            vertex_indices[1] + 1,
            vertex_indices[2] + 1
        )?;
    }

    Ok(())
}

pub fn import_from_obj(file_path: &str) -> std::io::Result<skeleton::Skeleton> {
    let mut skel = skeleton::Skeleton::new();
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        match parts.next() {
            Some("v") => {
                let x: f32 = parts.next().unwrap().parse().unwrap();
                let y: f32 = parts.next().unwrap().parse().unwrap();
                let z: f32 = parts.next().unwrap().parse().unwrap();
                skel.add_vertex(Vector3::new(x, y, z));
                skel.radii.push(1.0);
            }
            Some("f") => {
                let mut vertex_indices: Vec<usize> = Vec::new();
                for part in parts {
                    let indices: Vec<usize> = part
                        .split('/')
                        .map(|index| index.parse().unwrap_or(0))
                        .collect();
                    let vertex_index = indices[0] - 1;
                    vertex_indices.push(vertex_index);
                }
                let v1: usize = vertex_indices[0];
                let v2: usize = vertex_indices[1];
                let v3: usize = vertex_indices[2];

                let e1 = skel.add_edge([v1, v2]);
                let e2 = skel.add_edge([v2, v3]);
                let e3 = skel.add_edge([v3, v1]);

                skel.add_face([e1, e2, e3]);
            }
            _ => continue,
        }
    }

    Ok(skel)
}

pub fn import_radii(skel: &mut skeleton::Skeleton, file_path: &str) -> std::io::Result<()> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    skel.radii.clear();
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();

        let r: f32 = parts.next().unwrap().parse().unwrap();
        skel.radii.push(r);
    }

    Ok(())
}

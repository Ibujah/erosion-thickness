use nalgebra::base::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

pub struct Skeleton {
    vertices: Vec<Vector3<f32>>,
    radii: Vec<f32>,
    edges: Vec<[usize; 2]>,
    faces: Vec<[usize; 3]>,
    vertex_to_edges: HashMap<usize, Vec<usize>>,
    edge_to_faces: HashMap<usize, Vec<usize>>,
}

impl Skeleton {
    pub fn new() -> Self {
        Skeleton {
            vertices: Vec::new(),
            radii: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            vertex_to_edges: HashMap::new(),
            edge_to_faces: HashMap::new(),
        }
    }

    fn add_vertex(&mut self, position: Vector3<f32>) -> usize {
        self.vertices.push(position);
        self.vertices.len() - 1
    }

    fn add_edge(&mut self, vertex_indices: [usize; 2]) -> usize {
        if let Some(edge_indices) = self.get_edges_from_vertex(vertex_indices[0]) {
            for &edge_index in edge_indices.iter() {
                let [v1, v2] = self.edges[edge_index];
                if v1 == vertex_indices[0] && v2 == vertex_indices[1] {
                    return edge_index;
                }
                if v2 == vertex_indices[0] && v1 == vertex_indices[1] {
                    return edge_index;
                }
            }
        }

        let edge_index = self.edges.len();
        self.edges.push(vertex_indices);

        for vertex_index in &vertex_indices {
            self.vertex_to_edges
                .entry(*vertex_index)
                .or_insert(Vec::new())
                .push(edge_index);
        }

        edge_index
    }

    fn add_face(&mut self, edge_indices: [usize; 3]) {
        self.faces.push(edge_indices);

        for edge_index in &edge_indices {
            self.edge_to_faces
                .entry(*edge_index)
                .or_insert(Vec::new())
                .push(self.faces.len() - 1);
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vector3<f32>> {
        &self.vertices
    }

    pub fn get_radii(&self) -> &Vec<f32> {
        &self.radii
    }

    pub fn get_edges(&self) -> &Vec<[usize; 2]> {
        &self.edges
    }

    pub fn get_faces(&self) -> &Vec<[usize; 3]> {
        &self.faces
    }

    pub fn get_edges_from_vertex(&self, vertex_index: usize) -> Option<&Vec<usize>> {
        self.vertex_to_edges.get(&vertex_index)
    }

    pub fn get_faces_from_edge(&self, edge_index: usize) -> Option<&Vec<usize>> {
        self.edge_to_faces.get(&edge_index)
    }

    pub fn export_to_ply(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;

        writeln!(file, "ply")?;
        writeln!(file, "format ascii 1.0")?;
        writeln!(file, "element vertex {}", self.vertices.len())?;
        writeln!(file, "property float x")?;
        writeln!(file, "property float y")?;
        writeln!(file, "property float z")?;
        writeln!(file, "element edge {}", self.edges.len())?;
        writeln!(file, "property int vertex1")?;
        writeln!(file, "property int vertex2")?;
        writeln!(file, "element face {}", self.faces.len())?;
        writeln!(file, "property list uchar int vertex_indices")?;
        writeln!(file, "end_header")?;

        for vertex in &self.vertices {
            writeln!(file, "{} {} {}", vertex[0], vertex[1], vertex[2])?;
        }

        for edge in &self.edges {
            writeln!(file, "{} {}", edge[0], edge[1])?;
        }

        for face in &self.faces {
            let mut vertex_indices = Vec::new();
            for &edge_index in face {
                let [v1, v2] = self.edges[edge_index];
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

    pub fn export_to_obj(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;

        for vertex in &self.vertices {
            writeln!(file, "v {} {} {}", vertex[0], vertex[1], vertex[2])?;
        }

        for face in &self.faces {
            let mut vertex_indices = Vec::new();
            for &edge_index in face {
                let [v1, v2] = self.edges[edge_index];
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

    pub fn import_from_obj(&mut self, file_path: &str) -> std::io::Result<()> {
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
                    self.add_vertex(Vector3::new(x, y, z));
                    self.radii.push(1.0);
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

                    let e1 = self.add_edge([v1, v2]);
                    let e2 = self.add_edge([v2, v3]);
                    let e3 = self.add_edge([v3, v1]);

                    self.add_face([e1, e2, e3]);
                }
                _ => continue,
            }
        }

        Ok(())
    }

    pub fn import_radii(&mut self, file_path: &str) -> std::io::Result<()> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        self.radii.clear();
        for line in reader.lines() {
            let line = line?;
            let mut parts = line.split_whitespace();

            let r: f32 = parts.next().unwrap().parse().unwrap();
            self.radii.push(r);
        }

        Ok(())
    }
}

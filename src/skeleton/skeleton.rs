use nalgebra::base::*;
use std::collections::HashMap;

pub struct Skeleton {
    pub(super) vertices: Vec<Vector3<f32>>,
    pub(super) radii: Vec<f32>,
    pub(super) edges: Vec<[usize; 2]>,
    pub(super) faces: Vec<[usize; 3]>,
    pub(super) vertex_to_edges: HashMap<usize, Vec<usize>>,
    pub(super) edge_to_faces: HashMap<usize, Vec<usize>>,
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

    pub(super) fn add_vertex(&mut self, position: Vector3<f32>) -> usize {
        self.vertices.push(position);
        self.vertices.len() - 1
    }

    pub(super) fn add_edge(&mut self, vertex_indices: [usize; 2]) -> usize {
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

    pub(super) fn add_face(&mut self, edge_indices: [usize; 3]) {
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
}

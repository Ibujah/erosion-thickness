use nalgebra::base::*;
use std::collections::HashMap;

use ply_rs::ply::{
    Addable, DefaultElement, ElementDef, Property, PropertyDef, PropertyType, ScalarType,
};

pub struct Skeleton {
    vertex_property_types: HashMap<String, PropertyType>,
    face_property_types: HashMap<String, PropertyType>,
    vertex_coords: Vec<Vector3<f32>>,
    vertex_radius: Vec<f32>,
    vertex_properties: Vec<HashMap<String, Property>>,
    edges: Vec<[usize; 2]>,
    face_vertices: Vec<Vec<usize>>,
    face_edges: Vec<Vec<usize>>,
    faces_properties: Vec<HashMap<String, Property>>,
    vertex_to_edges: HashMap<usize, Vec<usize>>,
    edge_to_faces: HashMap<usize, Vec<usize>>,
}

fn property_to_propertytype(prop: Property) -> PropertyType {
    match prop {
        Property::Char(_) => PropertyType::Scalar(ScalarType::Char),
        Property::UChar(_) => PropertyType::Scalar(ScalarType::UChar),
        Property::Short(_) => PropertyType::Scalar(ScalarType::Short),
        Property::UShort(_) => PropertyType::Scalar(ScalarType::UShort),
        Property::Int(_) => PropertyType::Scalar(ScalarType::Int),
        Property::UInt(_) => PropertyType::Scalar(ScalarType::UInt),
        Property::Float(_) => PropertyType::Scalar(ScalarType::Float),
        Property::Double(_) => PropertyType::Scalar(ScalarType::Double),
        Property::ListChar(_) => PropertyType::List(ScalarType::UChar, ScalarType::Char),
        Property::ListUChar(_) => PropertyType::List(ScalarType::UChar, ScalarType::UChar),
        Property::ListShort(_) => PropertyType::List(ScalarType::UChar, ScalarType::Short),
        Property::ListUShort(_) => PropertyType::List(ScalarType::UChar, ScalarType::UShort),
        Property::ListInt(_) => PropertyType::List(ScalarType::UChar, ScalarType::Int),
        Property::ListUInt(_) => PropertyType::List(ScalarType::UChar, ScalarType::UInt),
        Property::ListFloat(_) => PropertyType::List(ScalarType::UChar, ScalarType::Float),
        Property::ListDouble(_) => PropertyType::List(ScalarType::UChar, ScalarType::Double),
    }
}

impl Skeleton {
    pub fn new() -> Self {
        Skeleton {
            vertex_property_types: HashMap::new(),
            face_property_types: HashMap::new(),
            vertex_coords: Vec::new(),
            vertex_radius: Vec::new(),
            vertex_properties: Vec::new(),
            edges: Vec::new(),
            face_vertices: Vec::new(),
            face_edges: Vec::new(),
            faces_properties: Vec::new(),
            vertex_to_edges: HashMap::new(),
            edge_to_faces: HashMap::new(),
        }
    }

    pub fn vertex_header_element(&self) -> ElementDef {
        let mut vertex_element = ElementDef::new("vertex".to_string());
        for (&key, &prop) in self.vertex_property_types.iter() {
            vertex_element.properties.add(PropertyDef::new(key, prop));
        }
        vertex_element
    }

    pub fn vertex_payload_element(&self) -> Vec<DefaultElement> {
        let mut vertices = Vec::new();

        for i in 0..self.vertex_coords.len() {
            let mut vertex = DefaultElement::new();
            vertex.insert("x".to_string(), Property::Float(self.vertex_coords[i][0]));
            vertex.insert("y".to_string(), Property::Float(self.vertex_coords[i][1]));
            vertex.insert("z".to_string(), Property::Float(self.vertex_coords[i][2]));
            vertex.insert("radius".to_string(), Property::Float(self.vertex_radius[i]));
            for (&key, &val) in self.vertex_properties[i].iter() {
                vertex.insert(key, val);
            }
            vertices.push(vertex);
        }

        vertices
    }

    pub fn face_header_element(&self) -> ElementDef {
        let mut face_element = ElementDef::new("face".to_string());
        for (&key, &prop) in self.face_property_types.iter() {
            face_element.properties.add(PropertyDef::new(key, prop));
        }
        face_element
    }

    pub fn face_payload_element(&self) -> Vec<DefaultElement> {
        let mut faces = Vec::new();

        for i in 0..self.face_edges.len() {
            let mut face = DefaultElement::new();
            face.insert(
                "vertex_indices".to_string(),
                Property::ListUInt(
                    self.face_vertices[i]
                        .iter()
                        .map(|&i| u32::try_from(i).unwrap())
                        .collect(),
                ),
            );
            for (&key, &val) in self.faces_properties[i].iter() {
                face.insert(key, val);
            }
            faces.push(face);
        }

        faces
    }

    pub(super) fn add_vertex(
        &mut self,
        position: Vector3<f32>,
        radius: f32,
        properties: HashMap<String, Property>,
    ) -> usize {
        self.vertex_coords.push(position);
        self.vertex_radius.push(radius);
        self.vertex_properties.push(properties);
        self.vertex_property_types
            .insert("x".to_string(), PropertyType::Scalar(ScalarType::Float));
        self.vertex_property_types
            .insert("y".to_string(), PropertyType::Scalar(ScalarType::Float));
        self.vertex_property_types
            .insert("z".to_string(), PropertyType::Scalar(ScalarType::Float));
        self.vertex_property_types.insert(
            "radius".to_string(),
            PropertyType::Scalar(ScalarType::Float),
        );
        for (key, prop) in properties {
            let ptype = property_to_propertytype(prop);
            self.vertex_property_types.insert(key, ptype);
        }

        self.vertex_coords.len() - 1
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

    pub(super) fn add_face(
        &mut self,
        vertex_indices: Vec<usize>,
        properties: HashMap<String, Property>,
    ) {
        let mut edge_indices = Vec::new();
        for i in 0..vertex_indices.len() {
            let j = (i + 1) % vertex_indices.len();
            let vi = vertex_indices[i];
            let vj = vertex_indices[j];
            let ei = self.add_edge([vi, vj]);
            edge_indices.push(ei);
        }
        self.face_vertices.push(vertex_indices);
        self.face_edges.push(edge_indices);
        self.faces_properties.push(properties);
        self.face_property_types.insert(
            "vertex_indices".to_string(),
            PropertyType::List(ScalarType::UChar, ScalarType::UInt),
        );
        for (key, prop) in properties {
            let ptype = property_to_propertytype(prop);
            self.face_property_types.insert(key, ptype);
        }

        for edge_index in &edge_indices {
            self.edge_to_faces
                .entry(*edge_index)
                .or_insert(Vec::new())
                .push(self.face_edges.len() - 1);
        }
    }

    pub fn get_vertices(&self) -> &Vec<Vector3<f32>> {
        &self.vertex_coords
    }

    pub fn get_radii(&self) -> &Vec<f32> {
        &self.vertex_radius
    }

    pub fn get_edges(&self) -> &Vec<[usize; 2]> {
        &self.edges
    }

    pub fn get_faces(&self) -> &Vec<Vec<usize>> {
        &self.face_edges
    }

    pub fn get_edges_from_vertex(&self, vertex_index: usize) -> Option<&Vec<usize>> {
        self.vertex_to_edges.get(&vertex_index)
    }

    pub fn get_faces_from_edge(&self, edge_index: usize) -> Option<&Vec<usize>> {
        self.edge_to_faces.get(&edge_index)
    }
}

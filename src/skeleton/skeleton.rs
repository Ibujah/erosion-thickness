use anyhow::Result;
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

pub(super) fn property_to_propertytype(prop: &Property) -> PropertyType {
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

    pub(super) fn vertex_header_element(&self) -> ElementDef {
        let mut vertex_element = ElementDef::new("vertex".to_string());
        for (key, prop) in self.vertex_property_types.iter() {
            vertex_element
                .properties
                .add(PropertyDef::new(key.clone(), prop.clone()));
        }
        vertex_element
    }

    pub(super) fn vertex_payload_element(&self) -> Vec<DefaultElement> {
        let mut vertices = Vec::new();

        for i in 0..self.vertex_coords.len() {
            let mut vertex = DefaultElement::new();
            for (key, val) in self.vertex_properties[i].iter() {
                vertex.insert(key.clone(), val.clone());
            }
            vertex.insert("x".to_string(), Property::Float(self.vertex_coords[i][0]));
            vertex.insert("y".to_string(), Property::Float(self.vertex_coords[i][1]));
            vertex.insert("z".to_string(), Property::Float(self.vertex_coords[i][2]));
            vertex.insert("radius".to_string(), Property::Float(self.vertex_radius[i]));
            vertices.push(vertex);
        }

        vertices
    }

    pub(super) fn face_header_element(&self) -> ElementDef {
        let mut face_element = ElementDef::new("face".to_string());
        for (key, prop) in self.face_property_types.iter() {
            face_element
                .properties
                .add(PropertyDef::new(key.clone(), prop.clone()));
        }
        face_element
    }

    pub(super) fn face_payload_element(&self) -> Vec<DefaultElement> {
        let mut faces = Vec::new();

        for i in 0..self.face_edges.len() {
            let mut face = DefaultElement::new();
            for (key, val) in self.faces_properties[i].iter() {
                face.insert(key.clone(), val.clone());
            }
            face.insert(
                "vertex_indices".to_string(),
                Property::ListUInt(
                    self.face_vertices[i]
                        .iter()
                        .map(|&i| u32::try_from(i).unwrap())
                        .collect(),
                ),
            );
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
        self.vertex_properties.push(properties.clone());
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
            let ptype = property_to_propertytype(&prop);
            self.vertex_property_types.insert(key.clone(), ptype);
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
        self.face_property_types.insert(
            "vertex_indices".to_string(),
            PropertyType::List(ScalarType::UChar, ScalarType::UInt),
        );
        for (key, prop) in properties.iter() {
            let ptype = property_to_propertytype(&prop);
            self.face_property_types.insert(key.clone(), ptype);
        }

        for edge_index in &edge_indices {
            self.edge_to_faces
                .entry(*edge_index)
                .or_insert(Vec::new())
                .push(self.face_edges.len() - 1);
        }
        self.face_edges.push(edge_indices);
        self.faces_properties.push(properties);
    }

    pub fn set_property_f32(&mut self, prop_name: &str, prop_value: &Vec<f32>) -> Result<()> {
        if prop_value.len() != self.vertex_properties.len() {
            Err(anyhow::Error::msg(
                "Number of vertices and properties does not match",
            ))
        } else {
            self.vertex_property_types.insert(
                prop_name.to_string(),
                PropertyType::Scalar(ScalarType::Float),
            );
            for i in 0..self.vertex_properties.len() {
                self.vertex_properties[i]
                    .insert(prop_name.to_string(), Property::Float(prop_value[i]));
            }
            Ok(())
        }
    }

    pub fn set_vertex_color_from_property_f32(&mut self, prop_name: &str) -> Result<()> {
        if !self.vertex_property_types.contains_key(prop_name) {
            return Err(anyhow::Error::msg("Property does not exist"));
        }
        if *self.vertex_property_types.get(prop_name).unwrap()
            != PropertyType::Scalar(ScalarType::Float)
        {
            return Err(anyhow::Error::msg("Property is not a float"));
        }
        let (min_p, max_p) = self
            .vertex_properties
            .iter()
            .fold(None, |val, vert_prop| {
                let v_cur = vert_prop.get(prop_name).unwrap();
                let v_cur = if let Property::Float(v) = v_cur {
                    *v
                } else {
                    panic!()
                };
                if let Some((v_min, v_max)) = val {
                    let v_min = if v_min < v_cur { v_min } else { v_cur };
                    let v_max = if v_max > v_cur { v_max } else { v_cur };
                    Some((v_min, v_max))
                } else {
                    Some((v_cur, v_cur))
                }
            })
            .unwrap();

        self.vertex_property_types
            .insert("red".to_string(), PropertyType::Scalar(ScalarType::UChar));
        self.vertex_property_types
            .insert("green".to_string(), PropertyType::Scalar(ScalarType::UChar));
        self.vertex_property_types
            .insert("blue".to_string(), PropertyType::Scalar(ScalarType::UChar));
        for prop in self.vertex_properties.iter_mut() {
            let p_cur = prop.get(prop_name).unwrap();
            let p_cur = if let Property::Float(p) = p_cur {
                p
            } else {
                panic!()
            };
            let factor = (p_cur - min_p) / (max_p - min_p);
            let r = (255.0 * factor) as u8;
            let g = 0 as u8;
            let b = (255.0 * (1.0 - factor)) as u8;
            prop.insert("red".to_string(), Property::UChar(r));
            prop.insert("green".to_string(), Property::UChar(g));
            prop.insert("blue".to_string(), Property::UChar(b));
        }

        Ok(())
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

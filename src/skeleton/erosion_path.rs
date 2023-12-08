use anyhow::Result;
use nalgebra::base::*;
use std::collections::HashMap;

use ply_rs::ply::{
    Addable, DefaultElement, ElementDef, Property, PropertyDef, PropertyType, ScalarType,
};

pub struct ErosionPath {
    vertex_property_types: HashMap<String, PropertyType>,
    edge_property_types: HashMap<String, PropertyType>,
    vertex_properties: Vec<HashMap<String, Property>>,
    edge_properties: Vec<HashMap<String, Property>>,
}

impl ErosionPath {
    pub fn new() -> Self {
        let mut vertex_property_types = HashMap::new();
        vertex_property_types.insert("x".to_string(), PropertyType::Scalar(ScalarType::Float));
        vertex_property_types.insert("y".to_string(), PropertyType::Scalar(ScalarType::Float));
        vertex_property_types.insert("z".to_string(), PropertyType::Scalar(ScalarType::Float));
        vertex_property_types.insert(
            "burntime".to_string(),
            PropertyType::Scalar(ScalarType::Float),
        );
        let mut edge_property_types = HashMap::new();
        edge_property_types.insert("vertex1".to_string(), PropertyType::Scalar(ScalarType::Int));
        edge_property_types.insert("vertex2".to_string(), PropertyType::Scalar(ScalarType::Int));
        ErosionPath {
            vertex_property_types,
            edge_property_types,
            vertex_properties: Vec::new(),
            edge_properties: Vec::new(),
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

        for i in 0..self.vertex_properties.len() {
            let mut vertex = DefaultElement::new();
            for (key, val) in self.vertex_properties[i].iter() {
                vertex.insert(key.clone(), val.clone());
            }
            vertices.push(vertex);
        }

        vertices
    }

    pub(super) fn edge_header_element(&self) -> ElementDef {
        let mut edge_element = ElementDef::new("edge".to_string());
        for (key, prop) in self.edge_property_types.iter() {
            edge_element
                .properties
                .add(PropertyDef::new(key.clone(), prop.clone()));
        }
        edge_element
    }

    pub(super) fn edge_payload_element(&self) -> Vec<DefaultElement> {
        let mut edges = Vec::new();

        for i in 0..self.edge_properties.len() {
            let mut edge = DefaultElement::new();
            for (key, val) in self.edge_properties[i].iter() {
                edge.insert(key.clone(), val.clone());
            }
            edges.push(edge);
        }

        edges
    }

    pub fn add_vertex(&mut self, position: Vector3<f32>, burntime: f32) -> usize {
        let mut vertex_property = HashMap::new();
        vertex_property.insert("x".to_string(), Property::Float(position.x));
        vertex_property.insert("y".to_string(), Property::Float(position.y));
        vertex_property.insert("z".to_string(), Property::Float(position.z));
        vertex_property.insert("burntime".to_string(), Property::Float(burntime));
        self.vertex_properties.push(vertex_property);

        self.vertex_properties.len() - 1
    }

    pub fn add_edge(&mut self, vertex_indices: [usize; 2]) -> usize {
        let mut edge_property = HashMap::new();
        edge_property.insert(
            "vertex1".to_string(),
            Property::Int(vertex_indices[0] as i32),
        );
        edge_property.insert(
            "vertex2".to_string(),
            Property::Int(vertex_indices[1] as i32),
        );
        self.edge_properties.push(edge_property);

        self.edge_properties.len() - 1
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
}

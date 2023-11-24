use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;

use nalgebra::base::*;

use ply_rs::parser::Parser;
use ply_rs::ply::{Addable, DefaultElement, Encoding, Ply, Property};
use ply_rs::writer::Writer;

use crate::skeleton::skeleton;

pub fn import_from_ply(file_path: &str) -> Result<()> {
    let mut f = std::fs::File::open(file_path).unwrap();

    let p = Parser::<DefaultElement>::new();
    let ply = p.read_ply(&mut f)?;

    let mut skel = skeleton::Skeleton::new();

    // load vertices
    if !ply.payload.contains_key("vertex") {
        return Err(anyhow::Error::msg("No vertex element in file"));
    }
    for v in ply.payload["vertex"].iter() {
        let mut x = None;
        let mut y = None;
        let mut z = None;
        let mut radius = None;
        let mut properties = HashMap::new();

        for (key, &prop) in v.iter() {
            match (key.as_ref(), prop) {
                ("x", Property::Float(val)) => x = Some(val),
                ("y", Property::Float(val)) => y = Some(val),
                ("z", Property::Float(val)) => z = Some(val),
                ("radius", Property::Float(val)) => radius = Some(val),
                (k, p) => {
                    properties.insert(k.to_string(), p);
                    ()
                }
            }
        }
        let x = x.ok_or(anyhow::Error::msg("No x property in vertex"))?;
        let y = y.ok_or(anyhow::Error::msg("No y property in vertex"))?;
        let z = z.ok_or(anyhow::Error::msg("No z property in vertex"))?;
        let radius = radius.ok_or(anyhow::Error::msg("No radius property in vertex"))?;
        skel.add_vertex(Vector3::new(x, y, z), radius, properties);
    }

    // load faces
    if !ply.payload.contains_key("face") {
        return Err(anyhow::Error::msg("No face element in file"));
    }
    for f in ply.payload["face"].iter() {
        let mut list_vertices = None;
        let mut properties = HashMap::new();

        for (key, &prop) in f.iter() {
            match (key.as_ref(), prop) {
                ("vertex_indices", Property::ListUInt(val)) => {
                    list_vertices = Some(val.iter().map(|&v| usize::try_from(v).unwrap()).collect())
                }
                (k, p) => {
                    properties.insert(k.to_string(), p);
                    ()
                }
            }
        }

        let list_vertices: Vec<usize> =
            list_vertices.ok_or(anyhow::Error::msg("No vertex_indices property in face"))?;

        skel.add_face(list_vertices, properties);
    }

    Ok(())
}

pub fn export_to_ply(skel: &skeleton::Skeleton, file_path: &str) -> Result<()> {
    let mut ply = {
        let mut ply = Ply::<DefaultElement>::new();
        ply.header.encoding = Encoding::Ascii;
        ply.header
            .comments
            .push("Generated with https://github.com/Ibujah/erosion-thickness".to_string());

        ply.header.elements.add(skel.vertex_header_element());
        ply.header.elements.add(skel.face_header_element());

        let vertices = skel.vertex_payload_element();
        let faces = skel.face_payload_element();

        ply.payload.insert("vertex".to_string(), vertices);
        ply.payload.insert("face".to_string(), faces);

        ply.make_consistent().unwrap();
        ply
    };

    let mut file = File::create(file_path)?;
    let w = Writer::new();
    let written = w.write_ply(&mut file, &mut ply).unwrap();
    Ok(())
}

use log;
use ndarray::ShapeBuilder;
use std::fs::File;
use std::io::Write;

use super::{burntime::BurnTime, vertex::Vertex};
use crate::skeleton::skeleton::Skeleton;

pub struct ETGraph<'a> {
    skel: &'a Skeleton,

    vert: Vec<Vertex>,
}

impl<'a> ETGraph<'a> {
    pub fn new(skel: &'a Skeleton, dist_max: f32, subdiv_max: usize) -> ETGraph<'a> {
        let mut etgraph = ETGraph {
            skel,
            vert: Vec::new(),
        };

        log::info!("build_subdiv_vertices");
        let subdiv_inds = etgraph.build_subdiv_vertices(dist_max, subdiv_max);

        log::info!("build_subdiv_faces");
        etgraph.build_subdiv_faces(subdiv_inds);

        log::info!("build_sectors");
        etgraph.build_sectors();

        etgraph
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vert
    }

    fn build_subdiv_vertices(&mut self, dist_max: f32, subdiv_max: usize) -> Vec<Vec<usize>> {
        // include original vertices and subdivision vertices in graph
        // return subdivided edges

        for i in 0..self.skel.get_vertices().len() {
            self.vert.push(Vertex::new(
                self.skel.get_vertices()[i],
                self.skel.get_radii()[i],
            ));
        }

        // subdivisions
        // subdiv_inds -> to each edge, associates a set of points which subdivises it
        // (including extremities)
        let mut subdivs_ind = Vec::new();
        for i in 0..self.skel.get_edges().len() {
            let [ind_v1, ind_v2] = self.skel.get_edges()[i];

            let v1 = self.vert[ind_v1].pos();
            let v2 = self.vert[ind_v2].pos();
            let len_edg = (v1 - v2).norm();

            let r1 = self.vert[ind_v1].rad();
            let r2 = self.vert[ind_v2].rad();

            let mut vec_sub = Vec::new();

            let nb_subdiv = (len_edg / dist_max).floor() as usize;
            let nb_subdiv = if nb_subdiv > subdiv_max {
                subdiv_max
            } else {
                nb_subdiv
            };
            vec_sub.push(ind_v1);
            for j in 1..nb_subdiv {
                let prop = (j as f32) / (nb_subdiv as f32);
                let cur_pos = (1.0 - prop) * v1 + prop * v2;
                let cur_rad = (1.0 - prop) * r1 + prop * r2;
                let ind = self.vert.len();
                self.vert.push(Vertex::new(cur_pos, cur_rad));
                vec_sub.push(ind);
            }
            vec_sub.push(ind_v2);
            subdivs_ind.push(vec_sub);
        }

        subdivs_ind
    }

    fn build_subdiv_faces(&mut self, subdivs_ind: Vec<Vec<usize>>) -> () {
        let nb_faces = self.skel.get_faces().len();

        for i in 0..nb_faces {
            let ind_edges = self.skel.get_faces()[i];
            let subdivs = vec![subdivs_ind[ind_edges[0]]];
            let mut all_length_2 = true;
            for i in 1..ind_edges.len() {
                let ind_edge = ind_edges[i];
                let subdiv_cur = subdivs_ind[ind_edge];
                if subdiv_cur.len() != 2 {
                    all_length_2 = false;
                }
                if *subdiv_cur.first().unwrap() == *subdivs[i - 1].last().unwrap() {
                    subdivs.push(subdiv_cur.clone());
                } else if *subdiv_cur.last().unwrap() == *subdivs[i - 1].last().unwrap() {
                    subdivs.push(subdiv_cur.iter().map(|&x| x).rev().collect());
                } else if *subdiv_cur.last().unwrap() == *subdivs[0].first().unwrap() {
                    subdivs.insert(0, subdiv_cur.clone());
                } else {
                    subdivs.insert(0, subdiv_cur.iter().map(|&x| x).rev().collect());
                }
            }

            if all_length_2 {
                for i in 0..subdivs.len() {
                    let ind_vert_i = subdivs[i][0];
                    for j in 0..subdivs.len() - 1 {
                        let j_prev = (i + j + subdivs.len()) % subdivs.len();
                        let j_next = (i + j + 1) % subdivs.len();
                        let ind_vert_prev = subdivs[j_prev][0];
                        let ind_vert_next = subdivs[j_next][0];
                        self.vert[ind_vert_i].add_couple_neigh(ind_vert_prev, ind_vert_next);
                    }
                }
            } else {
                let mut build_subdivided = |s1: &Vec<usize>, s2: &Vec<usize>, s3: &Vec<usize>| {
                    log::debug!("build_subdivided");
                    log::debug!("{:?}", s1);
                    log::debug!("{:?}", s2);
                    log::debug!("{:?}", s3);
                    // internal nodes
                    for i in 1..(s1.len() - 1) {
                        let ind_vert1 = s1[i];
                        let mut it = s1[(i + 1)..(i + 2)]
                            .iter()
                            .chain(s2[1..].iter())
                            .chain(s3[1..(s3.len() - 1)].iter())
                            .chain(s1[(i - 1)..i].iter());

                        let mut ind_vert2 = *it.next().unwrap();
                        for &ind_vert3 in it {
                            log::debug!("{} -> ({}, {})", ind_vert1, ind_vert2, ind_vert3);
                            self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                            ind_vert2 = ind_vert3;
                        }
                    }

                    // extremity node
                    let ind_vert1 = s1[0];
                    if s2.len() == 2 {
                        let ind_vert2 = s1[1];
                        let ind_vert3 = s3[s3.len() - 2];
                        log::debug!("{} -> ({}, {})", ind_vert1, ind_vert2, ind_vert3);
                        self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                    } else {
                        let ind_vert2 = s1[1];
                        let ind_vert3 = s2[1];
                        log::debug!("{} -> ({}, {})", ind_vert1, ind_vert2, ind_vert3);
                        self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                        for i in 1..(s2.len() - 2) {
                            let ind_vert2 = s2[i];
                            let ind_vert3 = s2[i + 1];
                            log::debug!("{} -> ({}, {})", ind_vert1, ind_vert2, ind_vert3);
                            self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                        }
                        let ind_vert2 = s2[s2.len() - 2];
                        let ind_vert3 = s3[s3.len() - 2];
                        log::debug!("{} -> ({}, {})", ind_vert1, ind_vert2, ind_vert3);
                        self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                    }
                };

                build_subdivided(&subdiv1, &subdiv2, &subdiv3);
                build_subdivided(&subdiv2, &subdiv3, &subdiv1);
                build_subdivided(&subdiv3, &subdiv1, &subdiv2);
            }
        }
    }

    fn build_sectors(&mut self) -> () {
        for i in 0..self.vert.len() {
            self.vert[i].compute_sectors();
        }
    }

    pub fn export_to_ply(&self, file_path: &str) -> std::io::Result<()> {
        let mut file = File::create(file_path)?;

        writeln!(file, "ply")?;
        writeln!(file, "format ascii 1.0")?;

        writeln!(file, "element vertex {}", self.skel.get_vertices().len())?;
        writeln!(file, "property float x")?;
        writeln!(file, "property float y")?;
        writeln!(file, "property float z")?;
        writeln!(file, "property uchar red")?;
        writeln!(file, "property uchar green")?;
        writeln!(file, "property uchar blue")?;

        writeln!(file, "element face {}", self.skel.get_faces().len())?;
        writeln!(file, "property list uchar int vertex_indices")?;

        writeln!(file, "end_header")?;

        let mut t_min = -1.0;
        let mut t_max = -1.0;
        for v in self.vert.iter() {
            if let &BurnTime::Time(t) = v.time() {
                if t_min < 0.0 || t_min > t {
                    t_min = t;
                }
                if t_max < 0.0 || t_max < t {
                    t_max = t;
                }
            }
        }

        log::info!("dists: {} {}", t_min, t_max);

        for i in 0..self.skel.get_vertices().len() {
            let v = &self.vert[i];
            write!(file, "{} {} {} ", v.pos()[0], v.pos()[1], v.pos()[2])?;
            if let &BurnTime::Time(vt) = v.time() {
                let t = (vt - t_min) / (t_max - t_min);
                write!(
                    file,
                    "{} {} {} ",
                    (t * 255.0) as u8,
                    0,
                    ((1.0 - t) * 255.0) as u8
                )?;
            } else {
                write!(file, "{} {} {} ", 0, 0, 0)?;
            };
            writeln!(file, "")?;
        }

        for face in self.skel.get_faces().iter() {
            let mut vertex_indices = Vec::new();
            for &edge_index in face {
                let [v1, v2] = self.skel.get_edges()[edge_index];
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

    pub fn export_geodesics_to_ply(&self, file_path: &str) -> std::io::Result<()> {
        let mut prime_arcs = Vec::new();
        for i in 0..self.vert.len() {
            let v = &self.vert[i];
            if let Some(ind_prime) = v.prime_neighbor() {
                prime_arcs.push([i, ind_prime]);
            }
        }

        let mut file = File::create(file_path)?;

        writeln!(file, "ply")?;
        writeln!(file, "format ascii 1.0")?;

        writeln!(file, "element vertex {}", self.vert.len())?;
        writeln!(file, "property float x")?;
        writeln!(file, "property float y")?;
        writeln!(file, "property float z")?;
        writeln!(file, "property uchar red")?;
        writeln!(file, "property uchar green")?;
        writeln!(file, "property uchar blue")?;

        writeln!(file, "element edge {}", prime_arcs.len())?;
        writeln!(file, "property int vertex1")?;
        writeln!(file, "property int vertex2")?;

        writeln!(file, "end_header")?;

        let mut t_min = -1.0;
        let mut t_max = -1.0;
        for v in self.vert.iter() {
            if let &BurnTime::Time(t) = v.time() {
                if t_min < 0.0 || t_min > t {
                    t_min = t;
                }
                if t_max < 0.0 || t_max < t {
                    t_max = t;
                }
            }
        }

        log::info!("dists: {} {}", t_min, t_max);

        for v in self.vert.iter() {
            write!(file, "{} {} {} ", v.pos()[0], v.pos()[1], v.pos()[2])?;
            if let &BurnTime::Time(vt) = v.time() {
                let t = (vt - t_min) / (t_max - t_min);
                write!(
                    file,
                    "{} {} {} ",
                    (t * 255.0) as u8,
                    0,
                    ((1.0 - t) * 255.0) as u8
                )?;
            } else {
                write!(file, "{} {} {} ", 0, 0, 0)?;
            };
            writeln!(file, "")?;
        }

        for i in 0..prime_arcs.len() {
            write!(file, "{} {} ", prime_arcs[i][0], prime_arcs[i][1])?;
            writeln!(file, "")?;
        }

        Ok(())
    }

    pub fn check_neighbors(&self) -> () {
        for i in 0..self.vert.len() {
            for &n in self.vert[i].neigh() {
                let nn = self.vert[n].get_num_neigh(i);
                if nn.is_none() {
                    log::debug!("{} -> {}, {} -x> {}", i, n, n, i);
                }
            }
        }
    }
}

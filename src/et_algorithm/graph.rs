use log;
use std::fs::File;
use std::io::Write;
use std::{cmp::Ordering, collections::HashSet};

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
                let cur_rad = if r1 < 0.0 || r2 < 0.0 {
                    -1.0
                } else {
                    (1.0 - prop) * r1 + prop * r2
                };
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
            // for &[ind_edg1, ind_edg2, ind_edg3] in faces.iter() {
            let [ind_edg1, ind_edg2, ind_edg3] = self.skel.get_faces()[i];
            let subdiv1: Vec<usize> = subdivs_ind[ind_edg1].clone();
            let subdiv2: Vec<usize> =
                if *subdivs_ind[ind_edg2].first().unwrap() == *subdiv1.last().unwrap() {
                    subdivs_ind[ind_edg2].clone()
                } else if *subdivs_ind[ind_edg2].last().unwrap() == *subdiv1.last().unwrap() {
                    subdivs_ind[ind_edg2].iter().map(|&x| x).rev().collect()
                } else if *subdivs_ind[ind_edg3].first().unwrap() == *subdiv1.last().unwrap() {
                    subdivs_ind[ind_edg3].clone()
                } else {
                    subdivs_ind[ind_edg3].iter().map(|&x| x).rev().collect()
                };
            let subdiv3: Vec<usize> =
                if *subdivs_ind[ind_edg2].last().unwrap() == *subdiv1.first().unwrap() {
                    subdivs_ind[ind_edg2].clone()
                } else if *subdivs_ind[ind_edg2].first().unwrap() == *subdiv1.first().unwrap() {
                    subdivs_ind[ind_edg2].iter().map(|&x| x).rev().collect()
                } else if *subdivs_ind[ind_edg3].last().unwrap() == *subdiv1.first().unwrap() {
                    subdivs_ind[ind_edg3].clone()
                } else {
                    subdivs_ind[ind_edg3].iter().map(|&x| x).rev().collect()
                };
            if subdiv1.len() == 2 && subdiv2.len() == 2 && subdiv3.len() == 2 {
                let ind_vert1 = subdiv1[0];
                let ind_vert2 = subdiv2[0];
                let ind_vert3 = subdiv3[0];
                self.vert[ind_vert1].add_couple_neigh(ind_vert2, ind_vert3);
                self.vert[ind_vert2].add_couple_neigh(ind_vert3, ind_vert1);
                self.vert[ind_vert3].add_couple_neigh(ind_vert1, ind_vert2);
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

    pub fn erosion_thickness(&mut self) -> () {
        let mut q = HashSet::new();
        // for i in 0..self.vert.len() {
        //     if self.vert[i].is_boundary() {
        //         let rad = self.vert[i].rad();
        //         self.vert[i].set_time(rad);
        //         q.insert(i);
        //     }
        // }
        for i in 0..self.vert.len() {
            if self.vert[i].rad() >= 0.0 {
                let rad = self.vert[i].rad();
                self.vert[i].set_time(rad);
                q.insert(i);
            }
        }

        let mut cpt = 0;
        while !q.is_empty() {
            cpt = cpt + 1;
            let mut v = None;
            let mut t_min = BurnTime::Infinity;
            for &ind_v in q.iter() {
                if self.vert[ind_v].time().inf_eq(&t_min) {
                    v = Some(ind_v);
                    t_min = *self.vert[ind_v].time();
                }
            }
            let v = v.unwrap();
            q.remove(&v);
            let v_time = if self.vert[v].rad() >= 0.0 {
                self.vert[v].rad()
            } else if let &BurnTime::Time(t) = self.vert[v].time() {
                t
            } else {
                continue;
            };

            if self.vert[v].is_burned() {
                continue;
            }
            log::info!("step {}: q: {}, v: {}, v_time: {}", cpt, q.len(), v, v_time);

            if let &Some(prime_sector) = self.vert[v].prime_sector() {
                self.vert[v].burn_sector(prime_sector);
            }

            let exposed_sectors = self.vert[v].get_exposed_sectors();
            for ind_sec in exposed_sectors {
                self.vert[v].burn_sector(ind_sec);
                self.vert[v].sectors()[ind_sec].set_time(v_time);
            }

            let unburned_sectors = self.vert[v].get_unburned_sectors();
            if unburned_sectors.is_empty() {
                self.vert[v].burn();
                // update all Neighbors
                for i in 0..self.vert[v].neigh().len() {
                    let u = self.vert[v].neigh()[i];
                    if !self.vert[u].is_burned() && self.vert[u].rad() < 0.0 {
                        // detection of sector and arc of u that contains v
                        let num_neigh_v = self.vert[u].get_num_neigh(v).unwrap();
                        let vec_t = self.vert[u].attached_sectors(num_neigh_v);

                        let arc_norm = (self.vert[v].pos() - self.vert[u].pos()).norm();
                        for t in vec_t {
                            if !self.vert[u].sectors()[t].is_burned() {
                                // computation of new burn time for u
                                let h = arc_norm + v_time;
                                if BurnTime::Time(h).inf_eq(self.vert[u].sectors()[t].time()) {
                                    // update on sector t if burntime is lower
                                    self.vert[u].sectors()[t].set_time(h);
                                    let ind_prime_arc =
                                        self.vert[u].sectors()[t].get_arc(num_neigh_v).unwrap();
                                    self.vert[u].sectors()[t].set_prime_arc(ind_prime_arc);
                                    if BurnTime::Time(h).inf_eq(self.vert[u].time()) {
                                        self.vert[u].set_time(h);
                                        self.vert[u].set_prime_sector(t);
                                        q.insert(u);
                                    }
                                }
                            }
                        }
                    } else if self.vert[v].is_singular() && self.vert[u].is_singular() && false {
                        // get v neighbors on prime sector
                        let prime_neigh_v =
                            self.vert[v].get_sector_neighs(self.vert[v].prime_sector().unwrap());
                        // get sector(s) of u intersecting v prime sector
                        let intersectors_u = self.vert[u].get_intersector(&prime_neigh_v);
                        let prime_sector_u = self.vert[u].prime_sector().unwrap();
                        if !intersectors_u.contains(&prime_sector_u) {
                            // if prime sectors does not intersect, then there is a prime sector
                            // switch, i.e. a kink point
                            log::info!("kink_point detected");

                            // compute wake direction
                            // after v
                            let prev_v = self.vert[v].prime_neighbor().unwrap();
                            let dir_v = self.vert[v].pos() - self.vert[prev_v].pos();
                            let prime_neigh_u = self.vert[u]
                                .get_sector_neighs(self.vert[u].prime_sector().unwrap());
                            let candidates_v = self.vert[v].set_kink_point(dir_v, &prime_neigh_u);

                            for cv in candidates_v.iter() {
                                let vecs = cv
                                    .iter()
                                    .map(|&i| self.vert[i].pos() - self.vert[v].pos())
                                    .collect();
                                let (ind_next, dir_next) = self.vert[v].choose_next(&vecs);
                                self.vert[cv[ind_next]].set_wake_point(dir_next);
                            }

                            // after u
                            let prev_u = self.vert[u].prime_neighbor().unwrap();
                            let dir_u = self.vert[u].pos() - self.vert[prev_u].pos();
                            let candidates_u = self.vert[u].set_kink_point(dir_u, &prime_neigh_v);

                            for cv in candidates_u.iter() {
                                let vecs = cv
                                    .iter()
                                    .map(|&i| self.vert[i].pos() - self.vert[u].pos())
                                    .collect();
                                let (ind_next, dir_next) = self.vert[u].choose_next(&vecs);
                                self.vert[cv[ind_next]].set_wake_point(dir_next);
                            }

                            // update next points if needed

                            // return;
                        }
                    }
                }
            } else {
                let sec_min = unburned_sectors
                    .into_iter()
                    .min_by(|&s1, &s2| {
                        let &t1 = self.vert[v].sectors()[s1].time();
                        let &t2 = self.vert[v].sectors()[s2].time();
                        if t1.inf_eq(&t2) {
                            Ordering::Less
                        } else {
                            Ordering::Greater
                        }
                    })
                    .unwrap();

                if let &BurnTime::Time(t) = self.vert[v].sectors()[sec_min].time() {
                    self.vert[v].set_prime_sector(sec_min);
                    self.vert[v].set_time(t);
                    q.insert(v);
                } else {
                    self.vert[v].reset_prime_sector();
                    self.vert[v].reset_time();
                    q.remove(&v);
                }
            }
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

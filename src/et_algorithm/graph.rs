use log;
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

    pub(super) fn get_vertices(&mut self) -> &mut Vec<Vertex> {
        &mut self.vert
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
            let ind_edges = &self.skel.get_faces()[i];
            let mut subdivs = vec![subdivs_ind[ind_edges[0]].clone()];
            for i in 1..ind_edges.len() {
                let ind_edge = ind_edges[i];
                let subdiv_cur = subdivs_ind[ind_edge].clone();
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

            for i in 0..subdivs.len() {
                // extremity of i-th subdivision
                let ind_vert = subdivs[i][0];
                let mut prev_ind = subdivs[(i + 1) % subdivs.len()][0];
                // all chains in between
                for j in 1..(subdivs.len() - 1) {
                    let j_cur = (i + j) % subdivs.len();
                    for k in 0..subdivs[j_cur].len() {
                        let cur_ind = subdivs[j_cur][k];
                        if prev_ind != cur_ind {
                            self.vert[ind_vert].add_couple_neigh(prev_ind, cur_ind);
                        }
                        prev_ind = cur_ind;
                    }
                }

                // inner node of i-th subdivision
                for ii in 1..(subdivs[i].len() - 1) {
                    let ind_vert = subdivs[i][ii];
                    let mut prev_ind = subdivs[i][ii + 1];
                    // all chains in between
                    for j in 1..subdivs.len() {
                        let j_cur = (i + j) % subdivs.len();
                        let range = if j == 1 {
                            1..subdivs[j_cur].len()
                        } else if j == subdivs.len() - 1 {
                            0..(subdivs[j_cur].len() - 1)
                        } else {
                            0..subdivs[j_cur].len()
                        };
                        for k in range {
                            let cur_ind = subdivs[j_cur][k];
                            if prev_ind != cur_ind {
                                self.vert[ind_vert].add_couple_neigh(prev_ind, cur_ind);
                            }
                            prev_ind = cur_ind;
                        }
                    }
                }
            }
        }
    }

    fn build_sectors(&mut self) -> () {
        for i in 0..self.vert.len() {
            self.vert[i].compute_sectors();
        }
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
}

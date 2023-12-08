use std::collections::HashSet;

use nalgebra::base::*;

use super::burntime::BurnTime;
use super::sector::Sector;

#[derive(Clone, Copy)]
pub enum ErosionThickness {
    Infinity,
    ET(f32),
}

#[derive(Clone)]
pub struct Vertex {
    pos: Vector3<f32>,
    rad: f32,
    neigh: Vec<usize>,                    // neighbor vertex indices
    neigh_adj: Vec<Vec<usize>>,           // adjacency graph
    edge_sector: Vec<Vec<Option<usize>>>, // sector associated to each edge

    time: BurnTime,
    burned: bool,
    boundary: bool,
    prime_sector: Option<usize>,

    sector: Vec<Sector>,
}

impl Vertex {
    pub fn new(pos: Vector3<f32>, rad: f32) -> Vertex {
        Vertex {
            pos,
            rad,
            neigh: Vec::new(),
            neigh_adj: Vec::new(),
            edge_sector: Vec::new(),
            time: BurnTime::Infinity,
            burned: false,
            boundary: false,
            prime_sector: None,
            sector: Vec::new(),
        }
    }

    pub fn pos(&self) -> Vector3<f32> {
        self.pos
    }

    pub fn rad(&self) -> f32 {
        self.rad
    }

    pub fn time(&self) -> &BurnTime {
        &self.time
    }

    pub fn erosion_thickness(&self) -> ErosionThickness {
        match self.time {
            BurnTime::Infinity => ErosionThickness::Infinity,
            BurnTime::Time(t) => ErosionThickness::ET(t - self.rad),
        }
    }

    pub fn burn(&mut self) -> () {
        self.burned = true
    }

    pub fn prime_sector(&self) -> &Option<usize> {
        &self.prime_sector
    }

    pub fn prime_neighbor(&self) -> Option<usize> {
        if let Some(sec) = self.prime_sector {
            if let &Some(arc) = self.sector[sec].prime_arc() {
                let num_neigh = self.sector[sec].arc()[arc];
                Some(self.neigh[num_neigh])
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn set_prime_sector(&mut self, sec: usize) -> () {
        self.prime_sector = Some(sec);
    }

    pub fn reset_prime_sector(&mut self) -> () {
        self.prime_sector = None;
    }

    pub fn set_time(&mut self, time: f32) -> () {
        self.time = BurnTime::Time(time);
    }

    pub fn reset_time(&mut self) -> () {
        self.time = BurnTime::Infinity;
    }

    pub fn is_burned(&self) -> bool {
        self.burned
    }

    pub fn is_boundary(&self) -> bool {
        self.boundary
    }

    pub fn sectors(&mut self) -> &mut Vec<Sector> {
        &mut self.sector
    }

    pub fn neigh(&self) -> &Vec<usize> {
        &self.neigh
    }

    pub fn get_num_neigh(&self, ind_vert: usize) -> Option<usize> {
        let mut pos = None;
        for i in 0..self.neigh.len() {
            if self.neigh[i] == ind_vert {
                pos = Some(i);
                break;
            }
        }
        pos
    }

    pub fn attached_sectors(&self, num_neigh: usize) -> Vec<usize> {
        let mut sec: Vec<usize> = self.edge_sector[num_neigh]
            .iter()
            .map(|&val| val.unwrap())
            .collect();
        sec.sort();
        sec.dedup();
        sec
    }

    pub fn add_couple_neigh(&mut self, ind_vert1: usize, ind_vert2: usize) -> () {
        let ind1 = if let Some(ind) = self.neigh.iter().position(|&iv| iv == ind_vert1) {
            ind
        } else {
            self.neigh.push(ind_vert1);
            self.neigh_adj.push(Vec::new());
            self.neigh.len() - 1
        };
        let ind2 = if let Some(ind) = self.neigh.iter().position(|&iv| iv == ind_vert2) {
            ind
        } else {
            self.neigh.push(ind_vert2);
            self.neigh_adj.push(Vec::new());
            self.neigh.len() - 1
        };

        self.neigh_adj[ind1].push(ind2);
        self.neigh_adj[ind2].push(ind1);
    }

    fn follow_sector(&self, first_vert: usize, curr_sec: usize) -> (Vec<usize>, bool) {
        let mut arc = Vec::new();
        arc.push(first_vert);
        let mut current_position = first_vert;
        let mut last_position = first_vert;
        let mut reached_extremity = false;
        let mut closed = false;
        while !reached_extremity {
            reached_extremity = true;
            for j in 0..self.edge_sector[current_position].len() {
                if self.edge_sector[current_position][j] == Some(curr_sec)
                    && self.neigh_adj[current_position][j] != last_position
                {
                    last_position = current_position;
                    current_position = self.neigh_adj[current_position][j];
                    if current_position != first_vert {
                        reached_extremity = false;
                    } else {
                        closed = true;
                    }
                    break;
                }
            }
            if !reached_extremity {
                arc.push(current_position);
            }
        }
        (arc, closed)
    }

    fn detect_sectors(&mut self) -> (usize, bool) {
        let mut num_sector = 0;
        // first, compute sectors with begin and end
        for i in 0..self.edge_sector.len() {
            if self.edge_sector[i].len() != 2 {
                for j in 0..self.edge_sector[i].len() {
                    if self.edge_sector[i][j].is_none() {
                        self.edge_sector[i][j] = Some(num_sector);
                        let mut prev_ind = i;
                        let mut curr_ind = self.neigh_adj[i][j];
                        loop {
                            if self.edge_sector[curr_ind].len() != 2 {
                                for k in 0..self.neigh_adj[curr_ind].len() {
                                    if self.neigh_adj[curr_ind][k] == prev_ind {
                                        self.edge_sector[curr_ind][k] = Some(num_sector);
                                    }
                                }
                                break;
                            } else {
                                for k in 0..self.edge_sector[curr_ind].len() {
                                    self.edge_sector[curr_ind][k] = Some(num_sector);
                                }
                                for k in 0..self.neigh_adj[curr_ind].len() {
                                    if self.neigh_adj[curr_ind][k] != prev_ind {
                                        prev_ind = curr_ind;
                                        curr_ind = self.neigh_adj[curr_ind][k];
                                        break;
                                    }
                                }
                            }
                        }
                        num_sector = num_sector + 1;
                    }
                }
            }
        }
        if num_sector == 0 {
            // no sector found, i.e. only degree 2 edges, i.e. only one sector
            for i in 0..self.edge_sector.len() {
                for j in 0..self.edge_sector[i].len() {
                    self.edge_sector[i][j] = Some(0);
                }
            }
            return (1, true);
        }

        (num_sector, false)
    }

    pub fn compute_sectors(&mut self) -> () {
        log::debug!("Neighbors");
        let mut bound = false;
        let mut sing = false;
        for i in 0..self.neigh_adj.len() {
            self.neigh_adj[i].sort();
            self.neigh_adj[i].dedup();
            self.edge_sector.push(vec![None; self.neigh_adj[i].len()]);
            if self.neigh_adj[i].len() == 1 {
                bound = true;
            }
            if self.neigh_adj[i].len() >= 3 {
                sing = true;
            }
            log::debug!("{}({}): {:?}", i, self.neigh[i], self.neigh_adj[i]);
        }
        self.boundary = bound && !sing;
        log::debug!("");

        // sectors detection
        let (num_sector, one_closed) = self.detect_sectors();

        log::debug!("Sector labels");
        for i in 0..self.edge_sector.len() {
            log::debug!("{}: {:?}", i, self.edge_sector[i]);
        }
        log::debug!("");

        // sectors creation
        if !one_closed {
            for curr_sec in 0..num_sector {
                // find extremity, first element with only one occurence of sector number
                let extremity = if let Some(ext) = self
                    .edge_sector
                    .iter()
                    .position(|v| v.iter().filter(|&&i| i == Some(curr_sec)).count() == 1)
                {
                    ext
                } else {
                    self.edge_sector
                        .iter()
                        .position(|v| {
                            v.len() != 2 && v.iter().filter(|&&i| i == Some(curr_sec)).count() != 0
                        })
                        .unwrap()
                };
                let (arc, closed) = self.follow_sector(extremity, curr_sec);
                let beg_sec: HashSet<usize> = self.edge_sector[*arc.first().unwrap()]
                    .iter()
                    .filter_map(|&s| {
                        if s == Some(curr_sec) && !closed {
                            None
                        } else {
                            s
                        }
                    })
                    .collect();
                let end_sec: HashSet<usize> = self.edge_sector[*arc.last().unwrap()]
                    .iter()
                    .filter_map(|&s| {
                        if s == Some(curr_sec) && !closed {
                            None
                        } else {
                            s
                        }
                    })
                    .collect();
                self.sector.push(Sector::new(arc, beg_sec, end_sec));
            }
        } else {
            let (arc, _) = self.follow_sector(0, 0);
            self.sector.push(Sector::new(
                arc,
                vec![0].into_iter().collect(),
                vec![0].into_iter().collect(),
            ));
        }

        // sectors neighboring
        log::debug!("Sectors neighboring");
        for i in 0..self.sector.len() {
            log::debug!(
                "{}: {:?} {:?}",
                i,
                self.sector[i].arc(),
                self.sector[i].get_sec_unexposed()
            );
        }
        log::debug!("");
    }

    pub fn burn_sector(&mut self, ind_sec: usize) -> () {
        self.sector[ind_sec].burn();
        let mut exposed = vec![ind_sec];
        while let Some(ind_exp) = exposed.pop() {
            for j in 0..self.sector.len() {
                if !self.sector[j].is_exposed() {
                    self.sector[j].expose_neigh(ind_exp);
                    if self.sector[j].is_exposed() {
                        exposed.push(j);
                    }
                }
            }
        }
    }

    pub fn get_exposed_sectors(&self) -> Vec<usize> {
        self.sector
            .iter()
            .enumerate()
            .filter_map(|(ind_sector, sector)| {
                if sector.is_exposed() && !sector.is_burned() {
                    Some(ind_sector)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_unburned_sectors(&self) -> Vec<usize> {
        self.sector
            .iter()
            .enumerate()
            .filter_map(|(ind_sector, sector)| {
                if !sector.is_burned() {
                    Some(ind_sector)
                } else {
                    None
                }
            })
            .collect()
    }
}

use std::collections::HashSet;

use super::burntime::BurnTime;

#[derive(Clone)]
pub struct Sector {
    arc: Vec<usize>, // contains references to Vertex.neigh
    burned: bool,
    prime_arc: Option<usize>,
    time: BurnTime,
    sec_neigh_unexposed: [HashSet<usize>; 2],
}

impl Sector {
    pub fn new(arc: Vec<usize>, beg_sec: HashSet<usize>, end_sec: HashSet<usize>) -> Sector {
        Sector {
            arc,
            burned: false,
            prime_arc: None,
            time: BurnTime::Infinity,
            sec_neigh_unexposed: [beg_sec, end_sec],
        }
    }

    pub fn is_exposed(&self) -> bool {
        self.sec_neigh_unexposed[0].len() == 0 || self.sec_neigh_unexposed[1].len() == 0
    }

    pub fn is_burned(&self) -> bool {
        self.burned
    }

    pub fn time(&self) -> &BurnTime {
        &self.time
    }

    pub fn arc(&self) -> &Vec<usize> {
        &self.arc
    }

    pub fn set_prime_arc(&mut self, prime_arc: usize) -> () {
        self.prime_arc = Some(prime_arc)
    }

    pub fn prime_arc(&self) -> &Option<usize> {
        &self.prime_arc
    }

    pub fn get_arc(&self, num_neigh: usize) -> Option<usize> {
        let mut pos = None;
        for i in 0..self.arc.len() {
            if self.arc[i] == num_neigh {
                pos = Some(i);
                break;
            }
        }
        pos
    }

    pub fn set_time(&mut self, time: f32) -> () {
        self.time = BurnTime::Time(time)
    }

    pub fn get_sec_unexposed(&self) -> &[HashSet<usize>; 2] {
        &self.sec_neigh_unexposed
    }

    pub fn burn(&mut self) -> () {
        self.burned = true;
        self.sec_neigh_unexposed[0].clear();
        self.sec_neigh_unexposed[1].clear();
    }

    pub fn expose_neigh(&mut self, ind_exp: usize) -> () {
        self.sec_neigh_unexposed[0].remove(&ind_exp);
        self.sec_neigh_unexposed[1].remove(&ind_exp);
    }
}

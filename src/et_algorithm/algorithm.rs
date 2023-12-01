use anyhow::Result;
use log;
use std::{cmp::Ordering, collections::HashSet};

use super::{burntime::BurnTime, graph::ETGraph, vertex::ErosionThickness};
use crate::skeleton::erosion_path::ErosionPath;
use crate::skeleton::skeleton::Skeleton;

pub fn erosion_thickness_computation(
    skeleton: &mut Skeleton,
    dist_max: f32,
    subdiv_max: usize,
) -> Result<ErosionPath> {
    let mut et_graph = ETGraph::new(skeleton, dist_max, subdiv_max);

    let mut q = HashSet::new();
    for i in 0..et_graph.get_vertices().len() {
        if et_graph.get_vertices()[i].is_boundary() {
            let rad = et_graph.get_vertices()[i].rad();
            et_graph.get_vertices()[i].set_time(rad);
            q.insert(i);
        }
    }

    let mut cpt = 0;
    while !q.is_empty() {
        cpt = cpt + 1;
        let mut v = None;
        let mut t_min = BurnTime::Infinity;
        for &ind_v in q.iter() {
            if et_graph.get_vertices()[ind_v].time().inf_eq(&t_min) {
                v = Some(ind_v);
                t_min = *et_graph.get_vertices()[ind_v].time();
            }
        }
        let v = v.unwrap();
        q.remove(&v);
        let v_time = if let &BurnTime::Time(t) = et_graph.get_vertices()[v].time() {
            t
        } else {
            continue;
        };

        if et_graph.get_vertices()[v].is_burned() {
            continue;
        }
        log::info!("step {}: q: {}, v: {}, v_time: {}", cpt, q.len(), v, v_time);

        if let &Some(prime_sector) = et_graph.get_vertices()[v].prime_sector() {
            et_graph.get_vertices()[v].burn_sector(prime_sector);
        }

        let exposed_sectors = et_graph.get_vertices()[v].get_exposed_sectors();
        for ind_sec in exposed_sectors {
            et_graph.get_vertices()[v].burn_sector(ind_sec);
            et_graph.get_vertices()[v].sectors()[ind_sec].set_time(v_time);
        }

        let unburned_sectors = et_graph.get_vertices()[v].get_unburned_sectors();
        if unburned_sectors.is_empty() {
            et_graph.get_vertices()[v].burn();
            // update all Neighbors
            for i in 0..et_graph.get_vertices()[v].neigh().len() {
                let u = et_graph.get_vertices()[v].neigh()[i];
                if !et_graph.get_vertices()[u].is_burned() {
                    // detection of sector and arc of u that contains v
                    let num_neigh_v = et_graph.get_vertices()[u].get_num_neigh(v).unwrap();
                    let vec_t = et_graph.get_vertices()[u].attached_sectors(num_neigh_v);

                    let arc_norm = (et_graph.get_vertices()[v].pos()
                        - et_graph.get_vertices()[u].pos())
                    .norm();
                    for t in vec_t {
                        if !et_graph.get_vertices()[u].sectors()[t].is_burned() {
                            // computation of new burn time for u
                            let h = arc_norm + v_time;
                            if BurnTime::Time(h)
                                .inf_eq(et_graph.get_vertices()[u].sectors()[t].time())
                            {
                                // update on sector t if burntime is lower
                                et_graph.get_vertices()[u].sectors()[t].set_time(h);
                                let ind_prime_arc = et_graph.get_vertices()[u].sectors()[t]
                                    .get_arc(num_neigh_v)
                                    .unwrap();
                                et_graph.get_vertices()[u].sectors()[t]
                                    .set_prime_arc(ind_prime_arc);
                                if BurnTime::Time(h).inf_eq(et_graph.get_vertices()[u].time()) {
                                    et_graph.get_vertices()[u].set_time(h);
                                    et_graph.get_vertices()[u].set_prime_sector(t);
                                    q.insert(u);
                                }
                            }
                        }
                    }
                }
            }
        } else {
            let sec_min = unburned_sectors
                .into_iter()
                .min_by(|&s1, &s2| {
                    let &t1 = et_graph.get_vertices()[v].sectors()[s1].time();
                    let &t2 = et_graph.get_vertices()[v].sectors()[s2].time();
                    if t1.inf_eq(&t2) {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .unwrap();

            if let &BurnTime::Time(t) = et_graph.get_vertices()[v].sectors()[sec_min].time() {
                et_graph.get_vertices()[v].set_prime_sector(sec_min);
                et_graph.get_vertices()[v].set_time(t);
                q.insert(v);
            } else {
                et_graph.get_vertices()[v].reset_prime_sector();
                et_graph.get_vertices()[v].reset_time();
                q.remove(&v);
            }
        }
    }

    let mut erosion_path = ErosionPath::new();

    let mut bt_max = 0.0;
    for i in 0..skeleton.get_vertices().len() {
        let bt = et_graph.get_vertices()[i].time();
        if let &BurnTime::Time(bt) = bt {
            if bt > bt_max {
                bt_max = bt;
            }
        }
    }

    let mut prime_arcs = Vec::new();
    for i in 0..et_graph.get_vertices().len() {
        let v = &et_graph.get_vertices()[i];
        if let Some(ind_prime) = v.prime_neighbor() {
            prime_arcs.push([i, ind_prime]);
        }
        let bt = if let &BurnTime::Time(bt) = v.time() {
            bt
        } else {
            bt_max
        };
        erosion_path.add_vertex(v.pos(), bt);
    }
    for i in 0..prime_arcs.len() {
        erosion_path.add_edge(prime_arcs[i]);
    }
    erosion_path.set_vertex_color_from_property_f32("burntime")?;

    let mut et_max = 0.0;
    for i in 0..skeleton.get_vertices().len() {
        let et = et_graph.get_vertices()[i].erosion_thickness();
        if let ErosionThickness::ET(et) = et {
            if et > et_max {
                et_max = et;
            }
        }
    }

    let mut et_values: Vec<f32> = Vec::new();
    for i in 0..skeleton.get_vertices().len() {
        let et = et_graph.get_vertices()[i].erosion_thickness();
        if let ErosionThickness::ET(et) = et {
            et_values.push(et);
        } else {
            et_values.push(et_max);
        }
    }

    skeleton.set_property_f32("erosion_thickness", &et_values)?;
    skeleton.set_vertex_color_from_property_f32("erosion_thickness")?;

    Ok(erosion_path)
}

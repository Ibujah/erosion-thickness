use et_rust_gpt::et_algorithm::graph::ETGraph;
use et_rust_gpt::skeleton::skeleton::Skeleton;

use env_logger;

fn main() -> std::io::Result<()> {
    let dist_max = 0.005;
    let subdiv_max = 1;
    let mut skeleton = Skeleton::new();

    env_logger::init();
    // Add vertices, edges, and faces as needed
    // skeleton.import_from_obj("resources/skeleton.obj")?;
    // skeleton.import_radii("resources/radius.rad")?;
    // skeleton.import_from_obj("resources/unit_skeleton4.obj")?;
    // skeleton.import_radii("resources/unit_radius4.rad")?;
    // skeleton.import_from_obj("resources/unit_skeleton5.obj")?;
    // skeleton.import_radii("resources/unit_radius5.rad")?;
    skeleton.import_from_obj("resources/test_kink_point.obj")?;

    let mut et_graph = ETGraph::new(&skeleton, dist_max, subdiv_max);

    et_graph.erosion_thickness();
    et_graph.export_to_ply("resources/res_skel.ply")?;
    et_graph.export_geodesics_to_ply("resources/geo_skel.ply")?;
    // et_graph.print_state();

    Ok(())
}

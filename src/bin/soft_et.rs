use anyhow::Result;
use erosion_thickness::et_algorithm::algorithm::erosion_thickness_computation;
use erosion_thickness::skeleton::io;

use env_logger;

fn main() -> Result<()> {
    let dist_max = 0.005;
    let subdiv_max = 1;

    env_logger::init();
    let mut skeleton = io::import_from_ply("resources/skeleton.ply")?;
    let erosion_path = erosion_thickness_computation(&mut skeleton, dist_max, subdiv_max)?;

    io::export_to_ply(&skeleton, "resources/skeleton_out.ply")?;
    io::export_erosion_path_to_ply(&erosion_path, "resources/erosion_path_out.ply")?;

    Ok(())
}

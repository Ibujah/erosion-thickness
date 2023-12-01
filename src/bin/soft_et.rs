use anyhow::Result;
use clap::Parser;
use std::fs;

use erosion_thickness::et_algorithm::algorithm::erosion_thickness_computation;
use erosion_thickness::skeleton::io;

use env_logger;

#[derive(Parser)]
struct Cli {
    #[arg(long = "input_skel")]
    ply_in_path: std::path::PathBuf,
    #[arg(default_value = "0.005", long = "dist_max")]
    dist_max: f32,
    #[arg(default_value = "1", long = "subdiv_max")]
    subdiv_max: usize,
    #[arg(default_value = "./output/", long = "pathout")]
    out_path: std::path::PathBuf,
    #[arg(default_value = "skeleton_erosion_thickness.ply", long = "output_skel")]
    ply_out_path: std::path::PathBuf,
    #[arg(default_value = "erosion_path.ply", long = "output_erosion_path")]
    ply_erosion_out_path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let ply_in_path_str = args.ply_in_path.to_str().unwrap();
    let out_path_str = args.out_path.to_str().unwrap();
    let ply_out_path_str = args.ply_out_path.to_str().unwrap();
    let ply_erosion_out_path_str = args.ply_erosion_out_path.to_str().unwrap();
    let dist_max = 0.005;
    let subdiv_max = 1;

    env_logger::init();
    let mut skeleton = io::import_from_ply(ply_in_path_str)?;
    let erosion_path = erosion_thickness_computation(&mut skeleton, dist_max, subdiv_max)?;

    fs::create_dir_all(out_path_str)?;
    io::export_to_ply(&skeleton, &format!("{}{}", out_path_str, ply_out_path_str))?;
    io::export_erosion_path_to_ply(
        &erosion_path,
        &format!("{}{}", out_path_str, ply_erosion_out_path_str),
    )?;

    Ok(())
}

#[cfg(test)]
mod et_tests {
    use env_logger;
    use erosion_thickness::et_algorithm::graph::ETGraph;
    use erosion_thickness::skeleton::skeleton::Skeleton;
    use std::sync::Once;

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    #[test]
    fn test_1_a() {
        initialize();
        let dist_max = 0.1;
        let nb_subdiv_max = 1;
        let mut skeleton = Skeleton::new();

        // Add vertices, edges, and faces as needed
        skeleton
            .import_from_obj("resources/unit_skeleton.obj")
            .unwrap();
        skeleton.import_radii("resources/unit_radius.rad").unwrap();

        let et_graph = ETGraph::new(&skeleton, dist_max, nb_subdiv_max);

        for v in 0..et_graph.get_vertices().len() {
            for s in 0..et_graph.get_vertices()[v].get_sectors().len() {
                let arc = et_graph.get_vertices()[v].get_sectors()[s].arc();
                log::debug!("{:?}", arc);
            }
        }

        // et_graph.erosion_thickness();
        assert!(true);
    }

    #[test]
    fn test_1_b() {
        initialize();
        let dist_max = 0.1;
        let nb_subdiv_max = 2;
        let mut skeleton = Skeleton::new();

        // Add vertices, edges, and faces as needed
        skeleton
            .import_from_obj("resources/unit_skeleton.obj")
            .unwrap();
        skeleton.import_radii("resources/unit_radius.rad").unwrap();

        let et_graph = ETGraph::new(&skeleton, dist_max, nb_subdiv_max);

        for v in 0..et_graph.get_vertices().len() {
            for s in 0..et_graph.get_vertices()[v].get_sectors().len() {
                let arc = et_graph.get_vertices()[v].get_sectors()[s].arc();
                log::debug!("{:?}", arc);
            }
        }

        // et_graph.erosion_thickness();
        assert!(true);
    }

    #[test]
    fn test_2_a() {
        initialize();
        let dist_max = 0.5;
        let nb_subdiv_max = 1;
        let mut skeleton = Skeleton::new();

        // Add vertices, edges, and faces as needed
        skeleton
            .import_from_obj("resources/unit_skeleton2.obj")
            .unwrap();
        skeleton.import_radii("resources/unit_radius2.rad").unwrap();

        let et_graph = ETGraph::new(&skeleton, dist_max, nb_subdiv_max);

        // et_graph.erosion_thickness();
        assert!(true);
    }

    #[test]
    fn test_3_a() {
        initialize();
        let dist_max = 0.5;
        let nb_subdiv_max = 1;
        let mut skeleton = Skeleton::new();

        // Add vertices, edges, and faces as needed
        skeleton
            .import_from_obj("resources/unit_skeleton3.obj")
            .unwrap();
        skeleton.import_radii("resources/unit_radius3.rad").unwrap();

        let et_graph = ETGraph::new(&skeleton, dist_max, nb_subdiv_max);

        // et_graph.erosion_thickness();
        assert!(true);
    }

    #[test]
    fn test_3_b() {
        initialize();
        let dist_max = 0.5;
        let nb_subdiv_max = 3;
        let mut skeleton = Skeleton::new();

        // Add vertices, edges, and faces as needed
        skeleton
            .import_from_obj("resources/unit_skeleton3.obj")
            .unwrap();
        skeleton.import_radii("resources/unit_radius3.rad").unwrap();

        let mut et_graph = ETGraph::new(&skeleton, dist_max, nb_subdiv_max);
        et_graph.check_neighbors();
        // et_graph.erosion_thickness();
        assert!(true);
    }
}

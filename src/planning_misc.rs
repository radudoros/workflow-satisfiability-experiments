pub mod planning_misc {
    use crate::predicates::{binary_predicates, weight_predicates};
    use crate::workflow::graph;

    pub struct Generator<'a> {
        g: &'a mut graph,
        node_option_size: i32,
        preds: &'a binary_predicates,
        selected_nodes: &'a Vec<usize>,
        current_index: Option<usize>,
    }

    impl<'a> Generator<'a> {
        pub fn new(
            g: &'a mut graph,
            node_option_size: i32,
            preds: &'a binary_predicates,
            selected_nodes: &'a Vec<usize>,
        ) -> Self {
            Self {
                g,
                node_option_size,
                preds,
                selected_nodes,
                current_index: Some(0),
            }
        }

        pub fn next(&mut self) -> Option<&[i32]> {
            while self.current_index.is_some() {
                if self.current_index.unwrap() == self.selected_nodes.len() {
                    if let Some(index) = self.current_index {
                        self.current_index = if index == 0 { None } else { Some(index - 1) };
                    }
                    return Some(&self.g.nodes_id);
                }

                let current_node_index = self.selected_nodes[self.current_index.unwrap()];
                self.g.nodes_id[current_node_index] += 1;
                if self.g.nodes_id[current_node_index] >= self.node_option_size {
                    self.g.nodes_id[current_node_index] = -1;
                    if let Some(index) = self.current_index {
                        self.current_index = if index == 0 { None } else { Some(index - 1) };
                    }
                    continue;
                }

                if self.preds.eval(&self.g) {
                    if let Some(index) = self.current_index {
                        self.current_index = Some(index + 1);
                    }
                }
            }

            None
        }
    }

    // Simple backtracking algorithm:
    pub fn plan(
        g: &mut graph,
        i: usize,
        node_options: &Vec<i32>,
        preds: &binary_predicates,
        sol: &mut Vec<i32>,
    ) {
        if !sol.is_empty() {
            // We already found a solution... prune out everything:
            return;
        }

        if i >= g.nodes_id.len() {
            sol.clone_from(&g.nodes_id);
            return;
        }

        for opt in node_options.iter() {
            g.nodes_id[i] = *opt;
            let res = preds.eval(g);
            if !res {
                continue;
            }

            plan(g, i + 1, node_options, preds, sol);

            if !sol.is_empty() {
                // avoid recalling predicates
                break;
            }
        }

        g.nodes_id[i] = -1;
    }

    // Backtracking while respecting the topological order:
    pub fn plan_ordered(
        g: &mut graph,
        i: usize,
        node_order: &Vec<usize>,
        node_options: &Vec<i32>,
        preds: &binary_predicates,
        sol: &mut Vec<i32>,
    ) {
        if !sol.is_empty() {
            // We already found a solution... prune out everything:
            return;
        }

        if i >= node_order.len() {
            sol.clone_from(&g.nodes_id);
            return;
        }

        let node = node_order[i];

        for opt in node_options.iter() {
            g.nodes_id[node] = *opt;
            let res = preds.filtered_eval(g, *opt);
            if !res {
                continue;
            }

            plan_ordered(g, i + 1, node_order, node_options, preds, sol);

            if !sol.is_empty() {
                // avoid recalling predicates
                break;
            }
        }

        // reset for backtrack recursion:
        g.nodes_id[node] = -1;
    }

    pub fn plan_weigthed(
        g: &mut graph,
        i: usize,
        node_order: &Vec<usize>,
        node_options: &Vec<i32>,
        preds: &weight_predicates,
        crt_weight: f64,
        allowed_weight: f64,
        sol: &mut Vec<i32>,
    ) {
        if !sol.is_empty() || crt_weight - 0.0001 > allowed_weight {
            // We already found a solution... prune out everything:
            return;
        }

        if i >= node_order.len() {
            sol.clone_from(&g.nodes_id);
            return;
        }

        let node = node_order[i];

        for opt in node_options.iter() {
            g.nodes_id[node] = *opt;
            let new_weight: f64 = preds.eval(g);

            plan_weigthed(
                g,
                i + 1,
                node_order,
                node_options,
                preds,
                new_weight,
                allowed_weight,
                sol,
            );

            if !sol.is_empty() {
                // avoid recalling predicates
                break;
            }
        }

        // reset for backtrack recursion:
        g.nodes_id[node] = -1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planning_misc::planning_misc::Generator;
    use crate::predicates::{
        generate_mock_ud_predicates, generate_mock_ui_predicates, generate_mock_weight_predicates,
    };
    use crate::workflow::{self, graph, topo_sort};

    fn create_graph() -> workflow::graph {
        let mut g = graph::new(4);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 3);

        g
    }

    #[test]
    fn test_plan_simple() {
        let mut g = create_graph();

        let node_options = vec![0, 1, 2, 3, 4, 5, 6];
        let preds = generate_mock_ud_predicates();
        let selected_nodes = vec![0, 1, 2, 3];

        let mut generator =
            Generator::new(&mut g, node_options.len() as i32, &preds, &selected_nodes);
        let mut cnt: usize = 0;
        while let Some(_solution) = generator.next() {
            cnt += 1;
        }

        assert_eq!(cnt, node_options.len() * node_options.len());
    }

    #[test]
    fn test_plan_topo() {
        let mut g = create_graph();

        let topo_sorted = topo_sort(&g);

        let node_options = vec![0, 1, 2, 3, 4, 5, 6];
        let preds = generate_mock_ui_predicates();

        let mut sol: Vec<i32> = Vec::new();
        planning_misc::plan_ordered(&mut g, 0, &topo_sorted, &node_options, &preds, &mut sol);

        // Replace the println statements with assertions
        assert!(!sol.is_empty(), "No plans worked...");
    }

    #[test]
    fn test_plan_weigthed() {
        let mut g = create_graph();

        let topo_sorted = topo_sort(&g);

        let node_options = vec![0, 1, 2, 3, 4, 5, 6];
        let preds = generate_mock_weight_predicates();

        let mut sol: Vec<i32> = Vec::new();
        planning_misc::plan_weigthed(
            &mut g,
            0,
            &topo_sorted,
            &node_options,
            &preds,
            0.0,
            1.0,
            &mut sol,
        );

        // Replace the println statements with assertions
        assert!(!sol.is_empty(), "No plans worked...");
    }
}

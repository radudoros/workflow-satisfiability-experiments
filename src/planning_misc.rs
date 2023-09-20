pub mod planning_misc {
    use crate::predicates::BinaryPredicateSet;
    use crate::workflow::Graph;

    pub struct Generator<'a> {
        g: &'a mut Graph,
        preds: &'a BinaryPredicateSet,
        selected_nodes: &'a Vec<usize>,
        auth_set_indices: Vec<i32>,
        auth_sets: Vec<Vec<i32>>,
        current_index: Option<usize>,
    }

    impl<'a> Generator<'a> {
        pub fn new(
            g: &'a mut Graph,
            preds: &'a BinaryPredicateSet,
            selected_nodes: &'a Vec<usize>,
            auth_sets: Vec<Vec<i32>>,
        ) -> Self {
            let len = g.nodes_id.len();
            Self {
                g,
                preds,
                selected_nodes,
                auth_set_indices: vec![-1; len],
                auth_sets,
                current_index: Some(0),
            }
        }

        pub fn next(&mut self) -> Option<&[i32]> {
            while let Some(index) = self.current_index {
                if index == self.selected_nodes.len() {
                    self.current_index = Some(index - 1);
                    return Some(&self.g.nodes_id);
                }

                let current_node_index = self.selected_nodes[index];
                self.auth_set_indices[current_node_index] += 1;
                let auth_len = self.auth_sets[current_node_index].len() as i32;
                if self.auth_set_indices[current_node_index] >= auth_len {
                    self.g.nodes_id[current_node_index] = -1;
                    self.auth_set_indices[current_node_index] = -1;
                    self.current_index = if index == 0 { None } else { Some(index - 1) };
                    continue;
                }

                let idx = self.auth_set_indices[current_node_index] as usize;
                self.g.nodes_id[current_node_index] = self.auth_sets[current_node_index][idx];
                // self.g.nodes_id[current_node_index] =
                //     self.auth_set_indices[current_node_index] as i32;

                if self.preds.eval(&self.g) {
                    self.current_index = Some(index + 1);
                }
            }

            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::planning_misc::planning_misc::Generator;
    use crate::predicates::generate_mock_ud_predicates;
    use crate::workflow::{self, Graph};

    fn create_graph() -> workflow::Graph {
        let mut g = Graph::new(4);
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

        let single_auth_set: Vec<i32> = (0..node_options.len()).map(|x| x as i32).collect();
        let auth_sets: Vec<Vec<i32>> = vec![single_auth_set.clone(); selected_nodes.len()];

        let mut generator = Generator::new(&mut g, &preds, &selected_nodes, auth_sets);
        let mut cnt: usize = 0;
        while let Some(_solution) = generator.next() {
            cnt += 1;
        }

        assert_eq!(cnt, node_options.len() * node_options.len());
    }
}

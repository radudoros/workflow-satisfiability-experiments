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
        conflict_sets: Vec<Vec<usize>>,
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
                conflict_sets: vec![Vec::new(); selected_nodes.len()],
            }
        }

        fn print_progress(&self) {
            if let Some(index) = self.current_index {
                // Only take into account the first (index + 1) selected_nodes
                let relevant_nodes = &self.selected_nodes[0..=index];
                // if self.auth_sets[relevant_nodes[0]][self.auth_set_indices[relevant_nodes[0]] as usize] != 2 {return;}

                // Zip the indices with their corresponding values
                let zipped: Vec<(usize, i32)> = relevant_nodes
                    .iter() // provides the index
                    .map(|&node| (node, self.auth_set_indices[node]))
                    .collect();

                println!("Zipped Indices and Values: {:?}", zipped);
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

                self.print_progress();
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

        pub fn smart_next(&mut self, ui_pred: &BinaryPredicateSet) -> Option<&[i32]> {
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
                    if index == 0 {
                        self.current_index = None;
                        break;
                    }

                    if !self.conflict_sets[index].is_empty() {
                        let best_jump_point = self.conflict_sets[index]
                            .iter()
                            .max_by_key(|&&node_idx| {
                                self.selected_nodes
                                    .iter()
                                    .position(|&selected_node_idx| selected_node_idx == node_idx)
                            })
                            .copied();

                        if let Some(best_point) = best_jump_point {
                            let best_point_index = self
                                .selected_nodes
                                .iter()
                                .position(|&selected_node_idx| selected_node_idx == best_point)
                                .unwrap_or(index);

                            // Clear conflict sets between best_jump_point and current index
                            for conflict_index in best_point_index..index {
                                self.conflict_sets[conflict_index].clear();
                            }

                            self.current_index = Some(best_point_index);

                            // self.current_index = self.selected_nodes
                            //     .iter()
                            //     .position(|&selected_node_idx| selected_node_idx == best_point);
                        } else {
                            self.current_index = Some(index - 1);
                        }

                        self.conflict_sets[index].clear(); // Clear the conflict set for the next iteration
                        continue;
                    }

                    self.current_index = Some(index - 1);
                    continue;
                }

                let idx = self.auth_set_indices[current_node_index] as usize;
                self.g.nodes_id[current_node_index] = self.auth_sets[current_node_index][idx];

                // let actual_state: Vec<(usize, i32)> = self.selected_nodes.iter()
                // .filter_map(|&node_idx| {
                //     let idx = self.auth_set_indices[node_idx];
                //     if idx != -1 {
                //         Some((node_idx, self.auth_sets[node_idx][idx as usize]))
                //     } else {
                //         None
                //     }
                // })
                // .collect();

                // println!("Zipped Indices and Actual Values: {:?}", actual_state);

                // if actual_state == [(6, 6), (8, 7)] || actual_state == [(12, 9)] {
                //     // Debugger breakpoint should be set on the next line
                //     println!("Hit the target state!"); // Set your breakpoint here
                // }

                let preds_result = self.preds.eval_idx(&self.g, current_node_index);
                let ui_pred_result = ui_pred.eval_idx(&self.g, current_node_index);

                match (preds_result, ui_pred_result) {
                    (Ok(()), Ok(())) => {
                        self.current_index = Some(index + 1);
                    }
                    (Err(Some(pred_prev)), _) | (_, Err(Some(pred_prev))) => {
                        self.conflict_sets[index].push(pred_prev);
                    }
                    _ => (),
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

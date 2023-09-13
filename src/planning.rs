use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod planning {
    use crate::bipartite_matching::BipartiteGraph;
    use crate::partition_generator::{IncrementalPartitionGenerator, PartitionsGenerator};
    use crate::planning_misc::planning_misc::Generator;
    use crate::predicates::binary_predicates;
    use crate::workflow::graph;

    // Find combination
    fn combine(
        auth: &Vec<Vec<usize>>,
        pattern_map: &Vec<Vec<usize>>,
        ulen: usize,
        g: &mut graph,
    ) -> Option<Vec<(usize, usize)>> {
        // 1. Initialize bipartite graph
        let pattern_size = pattern_map.len();
        let n = pattern_size + ulen;

        // 2. Populate the bipartite graph based on frequencies
        for (bi, block) in pattern_map.iter().enumerate() {
            // Initialize frequency vector for each user
            let mut frequency = vec![0; ulen];

            // Count the frequency of each authorized user for each step in the block
            for &s in block {
                for &authorized in &auth[s] {
                    frequency[authorized] += 1;
                }
            }

            // Add edges to the graph based on frequencies
            let block_size = block.len();
            g.adjacency_list[bi].clear(); // Clear existing adjacency list for this block

            for (user_id, &freq) in frequency.iter().enumerate() {
                if freq == block_size {
                    g.add_edge(bi, user_id + pattern_size);
                }
            }

            // Early exit: No valid edges for this block
            if g.adjacency_list[bi].is_empty() {
                return None;
            }
        }

        // 3. Find maximum matching
        let mut bipartite_graph = BipartiteGraph::new(&g.adjacency_list, pattern_size, n);
        let mut found_matching = bipartite_graph.max_matching_set();

        // Return the matching if it covers the entire pattern
        if found_matching.len() == pattern_size {
            found_matching.sort_by(|a, b| a.0.cmp(&b.0));
            return Some(found_matching);
        }

        None
    }

    pub fn plan_pattern_incremental(
        graph: &mut graph,
        node_priorities: &Vec<usize>,
        authorizations: &Vec<Vec<usize>>,
        predicates: &binary_predicates,
        user_length: usize,
    ) -> Option<Vec<usize>> {
        // Initialize incremental pattern generator and other variables
        let pattern_length = graph.nodes_id.len();
        let mut pattern_generator = IncrementalPartitionGenerator::new(pattern_length);
        let mut pattern_nodes = vec![vec![]; pattern_length];
        let assignment_graph_size = pattern_length + user_length;
        let mut assignment_graph = graph::new(assignment_graph_size);

        // Loop over generated partitions
        let mut next_partition = pattern_generator.next();
        while let Some(partition) = next_partition {
            // Step 1: Update the graph based on the new partition and evaluate predicates
            update_graph_labels(graph, partition, &node_priorities);

            if !predicates.eval(graph) {
                next_partition = pattern_generator.inc_next();
                continue;
            }

            // Step 2: Bipartite Matching
            let pattern_size =
                build_assignment_graph(&mut pattern_nodes, partition, node_priorities);
            if let Some(matching) = combine(
                authorizations,
                &pattern_nodes,
                user_length,
                &mut assignment_graph,
            ) {
                if partition.len() == pattern_length {
                    return Some(
                        partition
                            .iter()
                            .map(|&n| matching[n].1 - pattern_size)
                            .collect(),
                    );
                }

                next_partition = pattern_generator.next();
            } else {
                next_partition = pattern_generator.inc_next();
            }
        }

        None
    }

    /// Update node labels in the graph based on the given partition.
    fn update_graph_labels(graph: &mut graph, partition: &[usize], node_priorities: &[usize]) {
        for id in &mut graph.nodes_id {
            *id = -1;
        }
        for (index, &value) in partition.iter().enumerate() {
            graph.nodes_id[node_priorities[index]] = value as i32;
        }
    }

    /// Update node labels in the graph based on the given partition.
    fn update_graph_labels_no_prio(graph: &mut graph, partition: &[usize]) {
        for id in &mut graph.nodes_id {
            *id = -1;
        }
        for (index, &value) in partition.iter().enumerate() {
            graph.nodes_id[index] = value as i32;
        }
    }

    pub fn build_assignment_graph(
        pattern_nodes: &mut Vec<Vec<usize>>,
        partition: &[usize],
        node_priorities: &[usize],
    ) -> usize {
        let pattern_size = *partition.iter().max().unwrap_or(&0) + 1;

        pattern_nodes.resize_with(pattern_size, Vec::new);
        for pattern in 0..pattern_size {
            pattern_nodes[pattern].clear();
        }
        // Map nodes to patterns
        for (node, &pattern) in partition.iter().enumerate() {
            pattern_nodes[pattern].push(node_priorities[node]);
        }

        return pattern_size;
    }

    pub fn build_assignment_graph_no_prio(
        pattern_nodes: &mut Vec<Vec<usize>>,
        partition: &[usize],
    ) -> usize {
        let pattern_size = *partition.iter().max().unwrap_or(&0) + 1;

        pattern_nodes.resize_with(pattern_size, Vec::new);
        for pattern in 0..pattern_size {
            pattern_nodes[pattern].clear();
        }
        // Map nodes to patterns
        for (node, &pattern) in partition.iter().enumerate() {
            pattern_nodes[pattern].push(node);
        }

        return pattern_size;
    }

    pub fn plan_pattern(
        g: &mut graph,
        auth: &Vec<Vec<usize>>,
        preds: &binary_predicates,
        ulen: usize,
    ) -> Option<Vec<usize>> {
        let pattern_length = g.nodes_id.len();
        let node_priorities: Vec<usize> = (0..pattern_length).collect(); // [0, 1, 2, ..., k-1]

        let mut pattern_nodes: Vec<Vec<usize>> = vec![vec![]; g.nodes_id.len()];

        let assignment_graph_sz = g.nodes_id.len() + ulen;
        let mut assignment_graph = graph::new(assignment_graph_sz);

        let mut generator = PartitionsGenerator::new(g.nodes_id.len());
        while let Some(partition) = generator.next() {
            for (index, &value) in partition.iter().enumerate() {
                g.nodes_id[index] = value as i32;
            }

            if !preds.eval(&g) {
                continue;
            }

            let pattern_size =
                build_assignment_graph(&mut pattern_nodes, partition, &node_priorities);

            // combine now:
            let matching = combine(auth, &pattern_nodes, ulen, &mut assignment_graph);
            if let Some(matching) = matching {
                return Some(
                    partition
                        .iter()
                        .map(|&n| matching[n].1 - pattern_size)
                        .collect(),
                );
            }
        }

        return None;
    }

    pub fn plan_all(
        g: &mut graph,
        node_priorities: &Vec<usize>,
        auth: &Vec<Vec<usize>>,
        genneral_preds: &binary_predicates,
        general_nodes: &Vec<usize>,
        ui_preds: &binary_predicates,
        _ui_nodes: &[usize],
        ulen: usize,
    ) -> Option<Vec<usize>> {
        if general_nodes.is_empty() {
            return plan_pattern_incremental(g, node_priorities, &auth, ui_preds, ulen);
        }

        let mut g_clone = g.clone();
        let mut auth_cpy = auth.clone();

        let auth_i32: Vec<Vec<i32>> = auth
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|&num| num as i32)
                    .collect::<Vec<i32>>()
            })
            .collect();

        let mut generator = Generator::new(g, ulen, genneral_preds, &general_nodes, auth_i32);
        while let Some(solution) = generator.next() {
            // 1. check first if the solution and the authentication sets intersect:
            let mut ok = true;
            for (index, label) in solution.iter().enumerate() {
                if *label == -1 {
                    continue;
                }
                let label_usize = *label as usize;

                if !auth[index].contains(&label_usize) {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }

            if general_nodes.len() == g_clone.nodes_id.len() && ui_preds.len() == 0 {
                return Some(solution.iter().map(|&x| x as usize).collect());
            }

            // 2. Use the generated solutions in the authorization set:
            for (index, label) in solution.iter().enumerate() {
                if *label == -1 {
                    continue;
                }

                auth_cpy[index] = vec![*label as usize];
            }

            // 3. Pattern plan:
            let res =
                plan_pattern_incremental(&mut g_clone, node_priorities, &auth_cpy, ui_preds, ulen);
            if res.is_some() {
                return res;
            }
        }

        return None;
    }
}

pub fn read_auth_sets(filename: &str) -> Result<Vec<Vec<usize>>, Box<dyn std::error::Error>> {
    // Open the file for reading
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    // Create an iterator for the lines in the file
    let mut lines = reader.lines();

    // Read the number of users from the first line
    let _u: usize = lines.next().unwrap()?.trim().parse()?;

    // Read the number of nodes from the second line
    let n: usize = lines.next().unwrap()?.trim().parse()?;

    // Initialize an empty vector to store the authentication sets
    let mut auth_sets: Vec<Vec<usize>> = Vec::with_capacity(n);

    // Read the authorized users for each node and add to auth_sets
    for line in lines.take(n) {
        let users: Vec<usize> = line?
            .trim()
            .split_whitespace()
            .map(|num| num.parse().unwrap())
            .collect();

        auth_sets.push(users);
    }

    Ok(auth_sets)
}

pub fn read_auth_sets1(filename: &str) -> Result<Vec<Vec<usize>>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Skip the header lines about steps, users, and constraints
    for _ in 0..3 {
        lines.next();
    }

    // Convert user's authorization into a vector of authorized steps
    let mut auth_sets = Vec::new();
    for line in lines {
        let auths = line?
            .split(':')
            .nth(1)
            .ok_or("Malformed line")?
            .trim()
            .split_whitespace()
            .enumerate()
            .filter_map(
                |(index, value)| {
                    if value == "1" {
                        Some(index)
                    } else {
                        None
                    }
                },
            )
            .collect();
        auth_sets.push(auths);
    }

    Ok(auth_sets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::predicates::{generate_mock_ud_predicates, generate_mock_ui_predicates};
    use crate::workflow::graph;

    fn create_graph() -> graph {
        let mut g = graph::new(4);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 3);

        g
    }

    #[test]
    fn test_plan_pattern() {
        // Initialize the graph, authorized users, predicates and user length
        let mut g = graph::new(5);
        g.add_edge(0, 1);
        g.add_edge(0, 2);
        g.add_edge(1, 3);
        g.add_edge(2, 3);
        g.add_edge(3, 4);

        let auth = vec![vec![0, 1], vec![0], vec![1, 2], vec![0, 1, 2], vec![1, 2]];
        let preds = generate_mock_ui_predicates();
        let ulen = 3;

        // Call the function
        let result = planning::plan_pattern(&mut g, &auth, &preds, ulen);

        let expected_result = Some(vec![1, 0, 2, 1, 2]);

        // Check the result
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_plan_all() {
        let mut g = create_graph();
        let auth = vec![vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2], vec![0, 1, 2]];

        assert_eq!(g.nodes_id.len(), auth.len());

        let general_preds = generate_mock_ud_predicates();
        let ui_preds = generate_mock_ui_predicates();
        let general_nodes = vec![1, 2, 3];

        // TOOD: currently unused
        let ui_nodes = &[0, 1, 2, 3];

        let pattern_length = g.nodes_id.len();
        let node_priorities: Vec<usize> = (0..pattern_length).collect(); // [0, 1, 2, ..., k-1]

        let result = planning::plan_all(
            &mut g,
            &node_priorities,
            &auth,
            &general_preds,
            &general_nodes,
            &ui_preds,
            ui_nodes,
            3,
        );

        assert!(result.is_some(), "Expected all combinations to be tried.");
    }
}

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::io::Cursor;

pub mod planning {
    use crate::bipartite_matching::BipartiteGraph;
    use crate::partition_generator::PartitionsGenerator;
    use crate::planning_misc::planning_misc::Generator;
    use crate::predicates::binary_predicates;
    use crate::workflow::graph;

    // Find combination
    fn combine(
        auth: &Vec<Vec<usize>>,
        pattern_map: &Vec<Vec<usize>>,
        ulen: usize,
    ) -> Option<Vec<(usize, usize)>> {
        // 1. make bipartite graph
        let pattern_size = pattern_map.len();
        let n = pattern_size + ulen;
        let mut g = graph::new(n);

        for (bi, block) in pattern_map.iter().enumerate() {
            let mut frequency: std::collections::HashMap<usize, usize> =
                std::collections::HashMap::new();
            // for each step in the block, count the users allowed
            for s in block {
                for authorized in &auth[*s as usize] {
                    match frequency.get(&authorized) {
                        Some(count) => {
                            frequency.insert(*authorized, count + 1);
                        }
                        None => {
                            frequency.insert(*authorized, 1);
                        }
                    }
                }
            }

            let bsize = block.len();

            // frequencies of block size means that the node covers the full block:
            frequency.retain(|_, cnt| *cnt == bsize);
            for k in frequency.keys() {
                g.add_edge(bi, k + pattern_size);
            }
        }

        let mut bipartite_graph = BipartiteGraph::new(&g.adjacency_list, pattern_size, n);
        let mut found_matching = bipartite_graph.max_matching_set();

        if found_matching.len() != pattern_size {
            return None;
        }
        found_matching.sort_by(|a, b| a.0.cmp(&b.0));
        return Some(found_matching);
    }

    pub fn plan_pattern(
        g: &mut graph,
        auth: &Vec<Vec<usize>>,
        preds: &binary_predicates,
        ulen: usize,
    ) -> Option<Vec<usize>> {
        let mut generator = PartitionsGenerator::new(g.nodes_id.len());
        let mut pattern_nodes: Vec<Vec<usize>> = vec![vec![]; g.nodes_id.len()];

        while let Some(partition) = generator.next() {
            for (index, &value) in partition.iter().enumerate() {
                g.nodes_id[index] = value as i32;
            }
            // g.nodes_id = partition.iter().map(|&n| n as i32).collect();
            if !preds.eval(&g) {
                continue;
            }

            let pattern_size = *partition.iter().max().unwrap_or(&0) + 1;

            pattern_nodes.resize_with(pattern_size, Vec::new);
            for pattern in 0..pattern_size {
                pattern_nodes[pattern].clear();
            }
            // Map nodes to patterns
            for (node, &pattern) in partition.iter().enumerate() {
                pattern_nodes[pattern].push(node);
            }

            // combine now:
            let matching = combine(auth, &pattern_nodes, ulen);
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
        auth: &Vec<Vec<usize>>,
        genneral_preds: &binary_predicates,
        general_nodes: &Vec<usize>,
        ui_preds: &binary_predicates,
        _ui_nodes: &[usize],
        ulen: usize,
    ) -> Option<Vec<usize>> {
        // TODO: we can actually restrict the items that the generator goes through by using auth set

        let mut g_clone = g.clone();
        let mut auth_cpy = auth.clone();
        let mut generator = Generator::new(g, ulen as i32, genneral_preds, &general_nodes);
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
            let res = plan_pattern(&mut g_clone, &auth_cpy, ui_preds, ulen);
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
            .filter_map(|(index, value)| {
                if value == "1" {
                    Some(index)
                } else {
                    None
                }
            })
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

        let result = planning::plan_all(
            &mut g,
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

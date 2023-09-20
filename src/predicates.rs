use crate::workflow::Graph;
use std::cell::RefCell;
use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;

type BinaryPredicate = dyn Fn(&Graph) -> bool;

#[derive(Default)]
pub struct BinaryPredicateSet {
    preds: Vec<Box<BinaryPredicate>>,
    // predicate location id (to identify sites' predicates)
    pred_loc: Vec<i32>,
}

impl BinaryPredicateSet {
    #[allow(dead_code)]
    pub fn eval(&self, g: &Graph) -> bool {
        for p in self.preds.iter() {
            if !p(g) {
                return false;
            }
        }

        true
    }

    pub fn len(&self) -> usize {
        self.preds.len()
    }

    #[allow(dead_code)]
    pub fn filtered_eval(&self, g: &Graph, filtered_in: i32) -> bool {
        for (i, p) in self.preds.iter().enumerate() {
            if self.pred_loc[i] != filtered_in {
                continue;
            }

            if !p(g) {
                return false;
            }
        }

        true
    }
}

pub struct ReadConstraintsResult {
    pub ui_set: BinaryPredicateSet,
    pub non_ui_set: BinaryPredicateSet,
    pub auth_sets: Vec<Vec<usize>>,
    pub node_priorities: Vec<usize>,
    pub num_users: usize,
    pub non_ui_nodes: Vec<usize>,
}

pub fn read_constraints<R: Read>(reader: R) -> std::io::Result<ReadConstraintsResult> {
    let mut ui_set = BinaryPredicateSet {
        preds: vec![],
        pred_loc: vec![],
    };

    let mut non_ui_set = BinaryPredicateSet {
        preds: vec![],
        pred_loc: vec![],
    };

    let mut non_ui_nodes: Vec<usize> = Vec::new();

    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    // Read the number of steps, users, and constraints
    let num_steps: usize = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let num_users: usize = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let _num_constraints: usize = lines
        .next()
        .unwrap()?
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut step_predicate_counts = vec![0; num_steps];

    // Read the line that specifies "Authorizations:"
    let _ = lines.next();

    // Read user authorizations
    let mut auth_sets: Vec<Vec<usize>> = vec![Vec::new(); num_steps];
    // Read user authorizations by user
    let mut temp_auth_sets: Vec<Vec<usize>> = Vec::with_capacity(num_users);
    let mut node_unauthorization_counts: Vec<usize> = vec![0; num_steps];
    for _ in 0..num_users {
        let line = lines.next().unwrap()?;
        let users: Vec<usize> = line
            .split(':')
            .nth(1)
            .unwrap()
            .trim()
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();

        assert_eq!(users.len(), num_steps, "Mismatched authorization length");
        // Update the node_authorization_counts based on the current user's authorizations
        for (node, &authorization) in users.iter().enumerate() {
            if authorization == 0 {
                node_unauthorization_counts[node] += 1;
            }
        }
        temp_auth_sets.push(users);
    }

    // Transpose to read by steps
    for step in 0..num_steps {
        for user in 0..num_users {
            if temp_auth_sets[user][step] == 1 {
                auth_sets[step].push(user);
            }
        }
    }

    // Read the line that specifies "Constraints:"
    let _ = lines.next();

    for line in lines {
        let parts: Vec<String> = line?.trim().split_whitespace().map(String::from).collect();

        match parts.get(0).map(String::as_str) {
            Some("sod") => {
                // Get the node IDs from the third and fourth positions
                let x: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;
                let y: usize = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;

                step_predicate_counts[x] += 1;
                step_predicate_counts[y] += 1;

                ui_set.preds.push(Box::new(move |g: &Graph| {
                    g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] != g.nodes_id[y]
                }));
                ui_set.pred_loc.push(-1);
            }
            Some("bod") => {
                // Get the node IDs from the third and fourth positions
                let x: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;
                let y: usize = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;

                step_predicate_counts[x] += 10;
                step_predicate_counts[y] += 10;

                ui_set.preds.push(Box::new(move |g: &Graph| {
                    g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] == g.nodes_id[y]
                }));
                ui_set.pred_loc.push(-1);
            }
            Some("at") if parts.get(1).map(String::as_str) == Some("most") => {
                // Handle the "at most" constraint
                let max_count: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                let nodes: Vec<usize> = parts[4..]
                    .iter()
                    .filter_map(|s| s.parse::<usize>().ok().map(|x| x - 1))
                    .collect();

                for &node in &nodes {
                    step_predicate_counts[node] += 10;
                }

                // Hack, preallocate and capture it in the closure
                let shared_vec = Rc::new(RefCell::new(Vec::with_capacity(max_count)));

                ui_set.preds.push(Box::new(move |g: &Graph| {
                    let mut unique_ids = shared_vec.borrow_mut();
                    unique_ids.clear();

                    for &node in &nodes {
                        let id = g.nodes_id[node];
                        if id != -1 && !unique_ids.contains(&id) {
                            unique_ids.push(id);
                            if unique_ids.len() > max_count {
                                return false;
                            }
                        }
                    }
                    true
                }));
                ui_set.pred_loc.push(-1);
            }
            Some("assignment-dependent") => {
                let scope_indices: Vec<usize> = vec![
                    parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                    parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                ];

                for index in scope_indices.iter() {
                    if !non_ui_nodes.contains(index) {
                        non_ui_nodes.push(*index);
                    }
                }

                let and_index = parts.iter().position(|s| s == "and").unwrap_or(0);
                let users_set1: Vec<i32> = parts[5..and_index]
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                let users_set2: Vec<i32> = parts[and_index + 1..]
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                step_predicate_counts[scope_indices[0]] += 1;
                step_predicate_counts[scope_indices[1]] += 1;

                non_ui_set.preds.push(Box::new(move |g: &Graph| {
                    let user1 = g.nodes_id[scope_indices[0]];
                    let user2 = g.nodes_id[scope_indices[1]];
                    user1 == -1
                        || user2 == -1
                        || !users_set1.contains(&user1)
                        || (users_set1.contains(&user1) && users_set2.contains(&user2))
                }));
                non_ui_set.pred_loc.push(-1);
            }
            // Add other predicates as needed
            // ...
            _ => {
                // Unknown predicate or line format
                // Handle as necessary, e.g., log a warning or continue to the next line
            }
        }
    }

    let mut node_priorities = vec![0; num_steps];
    for node in 0..num_steps {
        // Assign higher priority to nodes with lower authorization counts
        // You can add weights here as needed
        let authorization_weight = 0.5; // Adjust the weight as per your requirements
        let predicate_weight = 2.5; // Adjust the weight as per your requirements

        let authorization_priority =
            authorization_weight * node_unauthorization_counts[node] as f64;
        let predicate_priority = predicate_weight * step_predicate_counts[node] as f64;

        node_priorities[node] = (authorization_priority + predicate_priority) as usize;
    }

    Ok(ReadConstraintsResult {
        ui_set,
        non_ui_set,
        auth_sets,
        node_priorities,
        num_users,
        non_ui_nodes,
    })
}

pub fn generate_mock_ui_predicates() -> BinaryPredicateSet {
    let mut v: Vec<Box<BinaryPredicate>> = Vec::new();
    v.push(Box::new(|g: &Graph| {
        // policy: don't allow a step and its child step to be same:
        for (v, adj) in g.adjacency_list.iter().enumerate() {
            if g.nodes_id[v] == -1 {
                continue;
            }
            for neigh in adj.iter() {
                if g.nodes_id[v] == g.nodes_id[*neigh] {
                    return false;
                }
            }
        }

        true
    }));

    let mut ids: Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i);
    }

    BinaryPredicateSet {
        preds: v,
        pred_loc: ids,
    }
}

#[allow(dead_code)]
pub fn generate_mock_ud_predicates() -> BinaryPredicateSet {
    let mut v: Vec<Box<BinaryPredicate>> = Vec::new();

    v.push(Box::new(|_| true));
    v.push(Box::new(|g: &Graph| {
        g.nodes_id.len() < 2 || g.nodes_id[2] == -1 || g.nodes_id[2] == 1
    }));
    v.push(Box::new(|g: &Graph| {
        g.nodes_id.len() < 1 || g.nodes_id[1] == -1 || g.nodes_id[1] == 2
    }));

    let mut ids: Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i);
    }

    BinaryPredicateSet {
        preds: v,
        pred_loc: ids,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_constraints() {
        let content = "\
            #Steps: 18
            #Users: 3
            #Constraints: 3
            Authorizations:
            user 1: 0 0 0 0 0 0 1 1 0 0 0 0 0 0 1 0 0 1
            user 2: 0 0 1 1 0 1 0 0 0 1 0 0 1 1 0 0 1 1
            user 3: 1 1 1 0 0 1 1 0 0 1 0 0 0 0 1 1 0 0
            Constraints:
            sod scope 5 17
            at most 3 scope 1 4 7 12 14
            assignment-dependent scope 12 14 users 1 2 3 and 1 2 3
            ";

        let cursor = Cursor::new(content);
        // let mut binary_preds = BinaryPredicateSet::default();
        let ReadConstraintsResult {
            ui_set: _,
            non_ui_set: _,
            auth_sets,
            node_priorities: _,
            num_users: _,
            non_ui_nodes: _,
        } = read_constraints(cursor).unwrap();

        assert_eq!(auth_sets.len(), 18);
        assert_eq!(auth_sets[0], vec![2]);
        assert_eq!(auth_sets[1], vec![2]);
        assert_eq!(auth_sets[2], vec![1, 2]);
        assert_eq!(auth_sets[16], vec![1]);

        // Add additional assertions here to verify the constraints (preds and pred_loc).
    }
}

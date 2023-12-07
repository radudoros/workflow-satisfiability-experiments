use crate::back_jumping_predicate::ScopedPredicate;
use crate::workflow::Graph;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read};
use std::rc::Rc;

#[derive(Default)]
pub struct BinaryPredicateSet {
    // preds: Vec<Box<BinaryPredicate>>,
    pub preds: Vec<ScopedPredicate>, // predicate location id (to identify sites' predicates)
                                     // pred_loc: Vec<i32>,
}

impl BinaryPredicateSet {
    #[allow(dead_code)]
    pub fn eval(&self, g: &Graph) -> bool {
        for p in self.preds.iter() {
            if !p.eval(g) {
                return false;
            }
        }

        true
    }

    pub fn eval_idx(&self, g: &Graph, idx: usize) -> Result<(), Option<usize>> {
        let mut max_prev: Option<usize> = None;

        for p in self.preds.iter() {
            if !p.eval_smart(g, idx) {
                let prev = p.get_prev(idx);
                max_prev = match (max_prev, prev) {
                    (Some(max), Some(current)) => Some(max.max(current)),
                    (None, Some(current)) => Some(current),
                    (current, None) => current,
                    _ => None,
                };
            }
        }

        if let Some(prev) = max_prev {
            Err(Some(prev))
        } else {
            Ok(())
        }
    }

    pub fn len(&self) -> usize {
        self.preds.len()
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
    let mut ui_set = BinaryPredicateSet { preds: vec![] };

    let mut non_ui_set = BinaryPredicateSet { preds: vec![] };

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

                ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
                        g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] != g.nodes_id[y]
                    }),
                    scope: vec![x, y],
                });
            }
            Some("bod") => {
                // Get the node IDs from the third and fourth positions
                let x: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;
                let y: usize = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;

                step_predicate_counts[x] += 100;
                step_predicate_counts[y] += 100;

                ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
                        g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] == g.nodes_id[y]
                    }),
                    scope: vec![x, y],
                });
            }
            Some("at") if parts.get(1).map(String::as_str) == Some("most") => {
                // Handle the "at most" constraint
                let max_count: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                let nodes: Vec<usize> = parts[4..]
                    .iter()
                    .filter_map(|s| s.parse::<usize>().ok().map(|x| x - 1))
                    .collect();

                for &node in &nodes {
                    step_predicate_counts[node] += 100;
                }

                let scope_arg = nodes.clone();

                // Hack, preallocate and capture it in the closure
                let shared_vec = Rc::new(RefCell::new(Vec::with_capacity(max_count)));

                ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
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
                    }),
                    scope: scope_arg,
                });
            }
            Some("assignment-dependent") => {
                let scope_indices: Vec<usize> = vec![
                    parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                    parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                ];

                for index in &scope_indices {
                    if !non_ui_nodes.contains(index) {
                        non_ui_nodes.push(*index);
                    }
                }

                let and_index = parts.iter().position(|s| s == "and").unwrap_or(0);
                let users_set1: Vec<i32> = parts[5..and_index]
                    .iter()
                    .filter_map(|s| s.parse::<i32>().ok().map(|num| num - 1))
                    .collect();
                let users_set2: Vec<i32> = parts[and_index + 1..]
                    .iter()
                    .filter_map(|s| s.parse::<i32>().ok().map(|num| num - 1))
                    .collect();

                step_predicate_counts[scope_indices[0]] += 5;
                step_predicate_counts[scope_indices[1]] += 5;

                let cache: Rc<RefCell<Option<(i32, i32, bool)>>> = Rc::new(RefCell::new(None));

                let scope_arg = scope_indices.clone();

                non_ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
                        let user1 = g.nodes_id[scope_indices[0]];
                        let user2 = g.nodes_id[scope_indices[1]];

                        // Check cache
                        if let Some((cached_u1, cached_u2, result)) = *cache.borrow() {
                            if cached_u1 == user1 && cached_u2 == user2 {
                                return result;
                            }
                        }

                        let result = user1 == -1
                            || user2 == -1
                            || !users_set1.contains(&user1)
                            || (users_set1.contains(&user1) && users_set2.contains(&user2));

                        // Update cache
                        if user1 != -1 && user2 != -1 && result {
                            *cache.borrow_mut() = Some((user1, user2, result));
                        }
                        result
                    }),
                    scope: scope_arg,
                });
            }
            Some("wang-li") => {
                let scope_indices: Vec<usize> = vec![
                    parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                    parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                ];

                let user_sets_start = parts.iter().position(|s| s == "groups").unwrap_or(0) + 1;
                let user_sets: Vec<Vec<i32>> = parts[user_sets_start..]
                    .split(|s| s.starts_with("("))
                    .filter(|group| !group.is_empty())
                    .map(|group| {
                        group
                            .iter()
                            .map(|s| s.trim_end_matches(')'))
                            .filter_map(|s| s.parse().ok())
                            .map(|x: i32| x - 1)
                            .collect()
                    })
                    .collect();

                for index in &scope_indices {
                    if !non_ui_nodes.contains(index) {
                        non_ui_nodes.push(*index);
                    }
                }

                for &si in &scope_indices {
                    step_predicate_counts[si] += 5;
                }

                let scope_arg = scope_indices.clone();

                non_ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
                        let mut crt_set: Option<usize> = None;
                        for &s in &scope_indices {
                            let node_id = g.nodes_id[s];
                            if node_id == -1 {
                                continue;
                            }

                            match crt_set {
                                None => {
                                    // This is the first assigned item; find the first set containing the user.
                                    crt_set =
                                        user_sets.iter().position(|set| set.contains(&node_id));
                                    if crt_set.is_none() {
                                        // If no set contains the user, the constraint is violated.
                                        return false;
                                    }
                                }
                                Some(set_idx) => {
                                    // All further nodes must belong to the same set as the first.
                                    if !user_sets[set_idx].contains(&node_id) {
                                        return false;
                                    }
                                }
                            }
                        }
                        true
                    }),
                    scope: scope_arg,
                });
            }
            Some("sual") => {
                // Handle the SUAL constraint
                let limit: usize = parts
                    .get(parts.iter().position(|s| s == "limit").unwrap_or(0) + 1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let users_start = parts.iter().position(|s| s == "users").unwrap_or(0) + 1;
                let super_users: Vec<usize> = parts[users_start..]
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .map(|x: usize| x - 1)
                    .collect();

                // Locate and parse the scope
                let scope_start = parts.iter().position(|s| s == "scope").unwrap_or(0) + 1;
                let scope_end = parts
                    .iter()
                    .position(|s| s == "limit")
                    .unwrap_or(parts.len());
                let scope_indices: Vec<usize> = parts[scope_start..scope_end]
                    .iter()
                    .filter_map(|s| s.parse().ok())
                    .map(|x: usize| x - 1)
                    .collect();

                for index in &scope_indices {
                    if !non_ui_nodes.contains(index) {
                        non_ui_nodes.push(*index);
                    }
                }

                let scope_arg = scope_indices.clone();

                non_ui_set.preds.push(ScopedPredicate {
                    pred: Box::new(move |g: &Graph| {
                        let mut assigned_users = Vec::new();
                        let mut assigned_steps = 0;

                        for &step in &scope_indices {
                            let user = g.nodes_id[step];
                            if user != -1 {
                                assigned_users.push(user as usize);
                                assigned_steps += 1;
                            }
                        }

                        if assigned_steps < scope_indices.len() {
                            return true;
                        }

                        let unique_users_count =
                            assigned_users.iter().collect::<HashSet<_>>().len();

                        if unique_users_count <= limit {
                            // All assigned users should be super users
                            for user in assigned_users {
                                if !super_users.contains(&user) {
                                    return false;
                                }
                            }
                        }

                        true
                    }),
                    scope: scope_arg,
                });
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
    let mut v: Vec<ScopedPredicate> = Vec::new();
    v.push(ScopedPredicate {
        pred: Box::new(|g: &Graph| {
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
        }),
        scope: vec![],
    });

    BinaryPredicateSet { preds: v }
}

#[allow(dead_code)]
pub fn generate_mock_ud_predicates() -> BinaryPredicateSet {
    let mut v: Vec<ScopedPredicate> = Vec::new();

    v.push(ScopedPredicate {
        pred: Box::new(|_| true),
        scope: vec![],
    });
    v.push(ScopedPredicate {
        pred: Box::new(|g: &Graph| {
            g.nodes_id.len() < 2 || g.nodes_id[2] == -1 || g.nodes_id[2] == 1
        }),
        scope: vec![],
    });
    v.push(ScopedPredicate {
        pred: Box::new(|g: &Graph| {
            g.nodes_id.len() < 1 || g.nodes_id[1] == -1 || g.nodes_id[1] == 2
        }),
        scope: vec![],
    });

    let mut ids: Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i);
    }

    BinaryPredicateSet { preds: v }
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

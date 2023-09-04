use crate::workflow::graph;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::collections::HashSet;

type binary_pred = dyn Fn(&graph) -> bool;
type weight_pred = dyn Fn(&graph) -> f64;

#[derive(Default)]
pub struct binary_predicates {
    preds: Vec<Box<binary_pred>>,
    // predicate location id (to identify sites' predicates)
    pred_loc: Vec<i32>,
}

impl binary_predicates {
    #[allow(dead_code)]
    pub fn eval(&self, g: &graph) -> bool {
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
    pub fn filtered_eval(&self, g: &graph, filtered_in: i32) -> bool {
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

    pub fn read_constraints<R: Read>(&mut self, reader: R) -> std::io::Result<(Vec<Vec<usize>>)> {
        // let file = File::open(filename)?;
        let reader = BufReader::new(reader);

        let mut lines = reader.lines();

        // Read the number of steps, users, and constraints
        let num_steps: usize = lines.next().unwrap()?.split_whitespace().nth(1).unwrap().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let num_users: usize = lines.next().unwrap()?.split_whitespace().nth(1).unwrap().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let _num_constraints: usize = lines.next().unwrap()?.split_whitespace().nth(1).unwrap().parse().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
        // Read the line that specifies "Authorizations:"
        let _ = lines.next();

        // Read user authorizations
        let mut auth_sets: Vec<Vec<usize>> = vec![Vec::new(); num_steps];
        // Read user authorizations by user
        let mut temp_auth_sets: Vec<Vec<usize>> = Vec::with_capacity(num_users);
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

                    self.preds.push(Box::new(move |g: &graph| g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] != g.nodes_id[y]));
                    self.pred_loc.push(-1);
                },
                Some("bod") => {
                    // Get the node IDs from the third and fourth positions
                    let x: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;
                    let y: usize = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1;

                    self.preds.push(Box::new(move |g: &graph| g.nodes_id[x] == -1 || g.nodes_id[y] == -1 || g.nodes_id[x] == g.nodes_id[y]));
                    self.pred_loc.push(-1);
                },
                Some("at") if parts.get(1).map(String::as_str) == Some("most") => {
                    // Handle the "at most" constraint
                    let max_count: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let nodes: Vec<usize> = parts[4..].iter().filter_map(|s| s.parse::<usize>().ok().map(|x| x - 1)).collect();

                    self.preds.push(Box::new(move |g: &graph| {
                        let unique_ids: HashSet<_> = nodes.iter().map(|&node| g.nodes_id[node]).filter(|&id| id != -1).collect();
                        unique_ids.len() <= max_count
                    }));
                    self.pred_loc.push(-1);
                },
                Some("assignment-dependent") => {
                    let scope_indices: Vec<usize> = vec![
                        parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                        parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0) - 1,
                    ];

                    let and_index = parts.iter().position(|s| s == "and").unwrap_or(0);
                    let users_set1: Vec<i32> = parts[5..and_index].iter().filter_map(|s| s.parse().ok()).collect();
                    let users_set2: Vec<i32> = parts[and_index + 1..].iter().filter_map(|s| s.parse().ok()).collect();

                    self.preds.push(Box::new(move |g: &graph| {
                        let user1 = g.nodes_id[scope_indices[0]];
                        let user2 = g.nodes_id[scope_indices[1]];
                        user1 == -1 || user2 == -1 || !users_set1.contains(&user1)
                            || (users_set1.contains(&user1) && users_set2.contains(&user2))
                    }));
                    self.pred_loc.push(-1);
                },
                // Add other predicates as needed
                // ...
                _ => {
                    // Unknown predicate or line format
                    // Handle as necessary, e.g., log a warning or continue to the next line
                },
            }
        }

        Ok(auth_sets)
    }

    pub fn read_sod_from_file(&mut self, filename: &str) -> std::io::Result<()> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let parts: Vec<usize> = line?
                .trim()
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let x = parts[0];
            let y = parts[1];
            self.preds
                .push(Box::new(move |g: &graph| g.nodes_id[x] != g.nodes_id[y]));
            self.pred_loc.push(-1); // assuming default value for the example
        }
        Ok(())
    }

    pub fn read_bod_from_file(&mut self, filename: &str) -> std::io::Result<()> {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let parts: Vec<usize> = line?
                .trim()
                .split_whitespace()
                .map(|s| s.parse().unwrap())
                .collect();
            let x = parts[0];
            let y = parts[1];
            self.preds
                .push(Box::new(move |g: &graph| g.nodes_id[x] == g.nodes_id[y]));
            self.pred_loc.push(-1); // assuming default value for the example
        }
        Ok(())
    }

    pub fn generate_k_different(&mut self, v: Vec<usize>, k: usize) {
        self.preds.push(Box::new(move |g: &graph| {
            let mut unique_labels = std::collections::HashSet::new();
            let mut cnt_unset = 0;

            for &n in v.iter() {
                if g.nodes_id[n] == -1 {
                    cnt_unset += 1;
                }
                unique_labels.insert(g.nodes_id[n]);
            }

            unique_labels.len() + cnt_unset == k
        }));
    }
}

#[allow(dead_code)]
pub struct weight_predicates {
    preds: Vec<Box<weight_pred>>,
    // predicate location id (to identify sites' predicates)
    pred_loc: Vec<i32>,
}

impl weight_predicates {
    #[allow(dead_code)]
    pub fn eval(&self, g: &graph) -> f64 {
        let mut sum: f64 = 0.0;
        for p in self.preds.iter() {
            sum += p(g);
        }

        sum
    }

    #[allow(dead_code)]
    pub fn filtered_eval(&self, g: &graph, filtered_in: i32) -> f64 {
        let mut sum: f64 = 0.0;
        for (i, p) in self.preds.iter().enumerate() {
            if self.pred_loc[i] != filtered_in {
                continue;
            }
            for p in self.preds.iter() {
                sum += p(g);
            }
        }

        sum
    }
}

pub fn generate_mock_ui_predicates() -> binary_predicates {
    let mut v: Vec<Box<binary_pred>> = Vec::new();
    v.push(Box::new(|g: &graph| {
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

    binary_predicates {
        preds: v,
        pred_loc: ids,
    }
}

#[allow(dead_code)]
pub fn generate_mock_ud_predicates() -> binary_predicates {
    let mut v: Vec<Box<binary_pred>> = Vec::new();

    v.push(Box::new(|_| true));
    v.push(Box::new(|g: &graph| {
        g.nodes_id.len() < 2 || g.nodes_id[2] == -1 || g.nodes_id[2] == 1
    }));
    v.push(Box::new(|g: &graph| {
        g.nodes_id.len() < 1 || g.nodes_id[1] == -1 || g.nodes_id[1] == 2
    }));

    let mut ids: Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i);
    }

    binary_predicates {
        preds: v,
        pred_loc: ids,
    }
}

#[allow(dead_code)]
pub fn generate_mock_weight_predicates() -> weight_predicates {
    let mut v: Vec<Box<weight_pred>> = Vec::new();
    v.push(Box::new(|g: &graph| {
        // policy: penalize when parent is same as child
        let mut w: f64 = 0.0;
        for (v, adj) in g.adjacency_list.iter().enumerate() {
            if g.nodes_id[v] == -1 {
                continue;
            }
            for neigh in adj.iter() {
                if g.nodes_id[v] == g.nodes_id[*neigh] {
                    w += 0.1;
                }
            }
        }

        w
    }));

    v.push(Box::new(|g: &graph| {
        // policy: penalize when parent has 2 children the same as itself
        let mut w: f64 = 0.0;
        for (v, adj) in g.adjacency_list.iter().enumerate() {
            if g.nodes_id[v] == -1 {
                continue;
            }
            let mut found_one = false;
            for neigh in adj.iter() {
                if g.nodes_id[v] != g.nodes_id[*neigh] {
                    continue;
                }

                if !found_one {
                    found_one = true;
                } else {
                    w += 0.3;
                }
            }
        }

        w
    }));

    let mut ids: Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i);
    }

    weight_predicates {
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
        let mut binary_preds = binary_predicates::default();
        let auth_sets = binary_preds.read_constraints(cursor).unwrap();

        assert_eq!(auth_sets.len(), 3);
        assert_eq!(auth_sets[0], vec![0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1]);
        assert_eq!(auth_sets[1], vec![0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1]);
        assert_eq!(auth_sets[2], vec![1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0]);

        // Add additional assertions here to verify the constraints (preds and pred_loc).
    }
}

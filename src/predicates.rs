use crate::workflow::graph;
use std::fs::File;
use std::io::{BufRead, BufReader};

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

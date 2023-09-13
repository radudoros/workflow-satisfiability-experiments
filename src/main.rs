#![allow(non_snake_case)]

use std::vec;

use planner::planning::planning::plan_all;
use planner::predicates::binary_predicates;
use planner::workflow::graph;

use std::fs::File;
use std::io::Cursor;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn old_run() {
    // Read graph from a file
    let mut g = match graph::from_file("graph.txt") {
        Ok(graph) => graph,
        Err(err) => {
            eprintln!("Error reading the graph: {}", err);
            return;
        }
    };
    g.print();

    // Read authentication sets
    let auth_sets = match planner::planning::read_auth_sets("auth_sets.txt") {
        Ok(auth) => auth,
        Err(err) => {
            eprintln!("Error reading authentication sets: {}", err);
            return;
        }
    };

    // Read Separation of Duty and Binding of Duty constraints into predicates
    let mut ui_preds = planner::predicates::binary_predicates::default();

    if let Err(err) = ui_preds.read_sod_from_file("sod.txt") {
        eprintln!("Error reading SoD constraints: {}", err);
        return;
    }

    if let Err(err) = ui_preds.read_bod_from_file("bod.txt") {
        eprintln!("Error reading BoD constraints: {}", err);
        return;
    }

    let mut ud_preds = planner::predicates::binary_predicates::default();
    let ud_scope = vec![3, 4, 5];
    ud_preds.generate_k_different(ud_scope.clone(), 2);

    let pattern_length = g.nodes_id.len();
    let node_priorities: Vec<usize> = (0..pattern_length).collect(); // [0, 1, 2, ..., k-1]

    let res = match plan_all(
        &mut g,
        &node_priorities,
        &auth_sets,
        &ud_preds,
        &ud_scope,
        &ui_preds,
        &vec![1, 2, 3],
        4,
    ) {
        Some(ans) => ans,
        None => {
            eprint!("No solutions here!");
            return;
        }
    };

    for v in res.iter() {
        print!(" {}", v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_ui() {
        let step_size = 20;
        let content = format!(
            "#Steps: {}\n\
        #Users: 7\n\
        #Constraints: 12\n\
        Authorizations:\n\
        user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1 1 0 1 1 0 1\n\
        user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0 1 1 0 0 1 0\n\
        user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1 0 1 0 0 1 1\n\
        user 4: 0 1 1 1 1 1 1 0 0 0 1 0 1 0 1 0 1 0 0 1\n\
        user 5: 0 0 0 0 1 1 1 1 1 0 1 1 1 0 0 0 0 1 1 0\n\
        user 6: 0 1 1 1 0 0 1 0 1 0 1 0 0 0 1 0 1 1 1 0\n\
        user 7: 0 1 0 1 1 1 1 1 1 0 0 1 0 1 0 1 1 0 1 1\n\
        Constraints:\n\
        sod scope 2 3\n\
        sod scope 1 2\n\
        sod scope 1 9\n\
        sod scope 1 10\n\
        bod scope 1 11\n\
        bod scope 3 4\n\
        sod scope 7 8\n\
        sod scope 11 12\n\
        sod scope 12 13\n\
        sod scope 14 15\n\
        bod scope 13 15\n\
        bod scope 9 10\n",
            step_size
        );

        let cursor = Cursor::new(content);
        let mut ui_preds = binary_predicates::default();
        let (auth_sets, node_priorities, ulen) = ui_preds.read_constraints(cursor).unwrap();

        let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
        node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

        let mut g = graph::new(step_size);

        let ud_preds = binary_predicates::default();
        let ud_scope = vec![];

        let res = match plan_all(
            &mut g,
            &node_indices,
            &auth_sets,
            &ud_preds,
            &ud_scope,
            &ui_preds,
            &vec![1, 2, 3],
            ulen,
        ) {
            Some(ans) => ans,
            None => {
                eprint!("No solutions here!");
                return;
            }
        };
        assert!(!res.is_empty(), "Result should not be empty");
    }

    #[test]
    fn test_benchmark_bt() {
        let step_size = 20;
        let content = format!(
            "#Steps: {}\n\
        #Users: 7\n\
        #Constraints: 12\n\
        Authorizations:\n\
        user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1 1 0 1 1 0 1\n\
        user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0 1 1 0 0 1 0\n\
        user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1 0 1 0 0 1 1\n\
        user 4: 0 1 1 1 1 1 1 0 0 0 1 0 1 0 1 0 1 0 0 1\n\
        user 5: 0 0 0 0 1 1 1 1 1 0 1 1 1 0 0 0 0 1 1 0\n\
        user 6: 0 1 1 1 0 0 1 0 1 0 1 0 0 0 1 0 1 1 1 0\n\
        user 7: 0 1 0 1 1 1 1 1 1 0 0 1 0 1 0 1 1 0 1 1\n\
        Constraints:\n\
        sod scope 2 3\n\
        sod scope 1 2\n\
        sod scope 1 9\n\
        sod scope 1 10\n\
        bod scope 1 11\n\
        bod scope 3 4\n\
        sod scope 7 8\n\
        sod scope 11 12\n\
        sod scope 12 13\n\
        sod scope 14 15\n\
        bod scope 13 15\n\
        bod scope 9 10\n",
            step_size
        );

        let cursor = Cursor::new(content);
        let mut ud_preds = binary_predicates::default();
        let (auth_sets, node_priorities, ulen) = ud_preds.read_constraints(cursor).unwrap();

        let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
        node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

        let mut g = graph::new(step_size);

        let ui_preds = binary_predicates::default();
        let ud_scope: Vec<usize> = (0..step_size).collect();

        let res = match plan_all(
            &mut g,
            &node_indices,
            &auth_sets,
            &ud_preds,
            &ud_scope,
            &ui_preds,
            &vec![1, 2, 3],
            ulen,
        ) {
            Some(ans) => ans,
            None => {
                eprint!("No solutions here!");
                return;
            }
        };
        assert!(!res.is_empty(), "Result should not be empty");
    }
}

fn main() {
    let filename = "instance99.txt";
    let path = Path::new(&filename);
    let file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    let reader = BufReader::new(file);

    let mut ui_preds = binary_predicates::default();
    let (auth_sets, node_priorities, ulen) = ui_preds.read_constraints(reader).unwrap();

    let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
    node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

    let step_size = auth_sets.len(); // Assuming step_size is the length of auth_sets
    let mut g = graph::new(step_size);

    let ud_preds = binary_predicates::default();
    let ud_scope = vec![];

    let res = match plan_all(
        &mut g,
        &node_indices,
        &auth_sets,
        &ud_preds,
        &ud_scope,
        &ui_preds,
        &vec![1, 2, 3],
        ulen,
    ) {
        Some(ans) => ans,
        None => {
            eprintln!("No solutions found!");
            return;
        }
    };

    if res.is_empty() {
        eprintln!("Result should not be empty");
        return;
    }

    // Do something with the result
    println!("Found a solution: {:?}", res);
}

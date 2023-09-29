#![allow(non_snake_case)]

use std::vec;

use planner::planning::planning::plan_all;
use planner::predicates::{read_constraints, ReadConstraintsResult};
use planner::workflow::Graph;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use planner::predicates::BinaryPredicateSet;

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
            assignment-dependent scope 19 20 users 3 and 7\n\
            sual scope 4 5 10 12 13 limit 3 users 3 4 5\n\
            wang-li scope 13 9 user groups (3 4) (6 7)
            bod scope 9 10\n",
            step_size
        );

        let cursor = Cursor::new(content);
        let ReadConstraintsResult {
            ui_set,
            non_ui_set,
            auth_sets,
            node_priorities,
            num_users,
            non_ui_nodes,
        } = read_constraints(cursor).unwrap();

        let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
        node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

        let mut g = Graph::new(step_size);

        let res = match plan_all(
            &mut g,
            &node_indices,
            &auth_sets,
            &non_ui_set,
            &non_ui_nodes,
            &ui_set,
            &vec![1, 2, 3],
            num_users,
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
        let ReadConstraintsResult {
            ui_set,
            non_ui_set: _,
            auth_sets,
            node_priorities,
            num_users,
            non_ui_nodes: _,
        } = read_constraints(cursor).unwrap();

        let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
        node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

        let mut g = Graph::new(step_size);

        let ui_preds = BinaryPredicateSet::default();
        let ud_scope: Vec<usize> = (0..step_size).collect();

        let res = match plan_all(
            &mut g,
            &node_indices,
            &auth_sets,
            &ui_set,
            &ud_scope,
            &ui_preds,
            &vec![1, 2, 3],
            num_users,
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
    // Collect command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        return;
    }

    // Get the filename argument
    let filename = &args[1];
    let path = Path::new(filename);
    let file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    let reader = BufReader::new(file);

    let ReadConstraintsResult {
        ui_set,
        non_ui_set,
        auth_sets,
        node_priorities,
        num_users,
        non_ui_nodes,
    } = read_constraints(reader).unwrap();

    let mut node_indices: Vec<usize> = (0..node_priorities.len()).collect();
    node_indices.sort_by_key(|&index| std::cmp::Reverse(node_priorities[index]));

    let step_size = auth_sets.len(); // Assuming step_size is the length of auth_sets
    let mut g = Graph::new(step_size);

    let res = match plan_all(
        &mut g,
        &node_indices,
        &auth_sets,
        &non_ui_set,
        &non_ui_nodes,
        &ui_set,
        &vec![1, 2, 3],
        num_users,
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

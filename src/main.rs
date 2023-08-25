#![allow(non_snake_case)]

use std::vec;

use planner::planning::planning::plan_all;
use planner::predicates::binary_predicates;
use planner::workflow::graph;

use std::io::Cursor;


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

    let res = match plan_all(
        &mut g,
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


fn naive_backtracking(g: &graph, bp: &binary_predicates, auth: &Vec<Vec<i32>>) {

}

fn combined_approach(g: &graph, bp: &binary_predicates, auth: &Vec<Vec<i32>>) {
    // Implement the combined approach
}

// fn benchmark_naive_backtracking(c: &mut Criterion) {
//     let input_data = /* prepare the input data */;
//     c.bench_function("Naive Backtracking", |b| {
//         b.iter(|| naive_backtracking(black_box(&input_data)))
//     });
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_ui() {
        // let step_size = 10;
        // let content = format!("#Steps: {}\n\
        // #Users: 3\n\
        // #Constraints: 3\n\
        // Authorizations:\n\
        // user 1: 1 0 1 1 0 1 0 1 1 0\n\
        // user 2: 0 1 0 1 1 0 1 0 1 1\n\
        // user 3: 1 1 0 1 0 0 1 0 1 1\n\
        // Constraints:\n\
        // sod scope 1 2\n\
        // sod scope 0 1\n\
        // bod scope 2 3\n\
        // sod scope 6 7\n\
        // bod scope 8 9\n", step_size);
        // let step_size = 20;
        // let content = format!("#Steps: {}\n\
        // #Users: 3\n\
        // #Constraints: 6\n\
        // Authorizations:\n\
        // user 1: 1 0 1 1 0 1 0 1 1 0 1 1 0 1 0 0 1 0 1 1\n\
        // user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 1 0 1 0 1 1 0\n\
        // user 3: 1 1 0 1 0 0 1 0 1 1 0 1 0 1 1 0 1 0 1 1\n\
        // Constraints:\n\
        // sod scope 1 2\n\
        // sod scope 0 1\n\
        // bod scope 2 3\n\
        // sod scope 6 7\n\
        // bod scope 8 9\n", step_size);

        let step_size = 14;
        let content = format!("#Steps: {}\n\
        #Users: 3\n\
        #Constraints: 6\n\
        Authorizations:\n\
        user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1\n\
        user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0\n\
        user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1\n\
        Constraints:\n\
        sod scope 1 2\n\
        sod scope 0 1\n\
        bod scope 2 3\n\
        sod scope 6 7\n\
        bod scope 8 9\n", step_size);

        let cursor = Cursor::new(content);
        let mut ui_preds = binary_predicates::default();
        let mut auth_sets = ui_preds.read_constraints(cursor).unwrap();
        auth_sets = (0..step_size)
        .map(|step| {
            auth_sets
                .iter()
                .enumerate()
                .filter(|(_, users)| users[step] == 1)
                .map(|(index, _)| index)
                .collect()
        })
        .collect();
    
        let mut g = graph::new(step_size);
    
        let ud_preds = binary_predicates::default();
        let ud_scope = vec![1];
    
        let res = match plan_all(
            &mut g,
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
        assert!(!res.is_empty(), "Result should not be empty");
    }

    #[test]
    fn test_benchmark_bt() {
        // let step_size = 10;
        // let content = format!("#Steps: {}\n\
        // #Users: 3\n\
        // #Constraints: 3\n\
        // Authorizations:\n\
        // user 1: 1 0 1 1 0 1 0 1 1 0\n\
        // user 2: 0 1 0 1 1 0 1 0 1 1\n\
        // user 3: 1 1 0 1 0 0 1 0 1 1\n\
        // Constraints:\n\
        // sod scope 1 2\n\
        // sod scope 0 1\n\
        // bod scope 2 3\n\
        // sod scope 6 7\n\
        // bod scope 8 9\n", step_size);
        // let step_size = 20;
        // let content = format!("#Steps: {}\n\
        // #Users: 3\n\
        // #Constraints: 6\n\
        // Authorizations:\n\
        // user 1: 1 0 1 1 0 1 0 1 1 0 1 1 0 1 0 0 1 0 1 1\n\
        // user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 1 0 1 0 1 1 0\n\
        // user 3: 1 1 0 1 0 0 1 0 1 1 0 1 0 1 1 0 1 0 1 1\n\
        // Constraints:\n\
        // sod scope 1 2\n\
        // sod scope 0 1\n\
        // bod scope 2 3\n\
        // sod scope 6 7\n\
        // bod scope 8 9\n", step_size);

        let step_size = 14;
        let content = format!("#Steps: {}\n\
        #Users: 3\n\
        #Constraints: 6\n\
        Authorizations:\n\
        user 1: 1 0 1 1 0 1 0 1 1 0 1 1 1 1\n\
        user 2: 0 1 0 1 1 0 1 0 1 1 1 0 1 0\n\
        user 3: 1 1 0 1 0 0 1 0 1 1 0 1 1 1\n\
        Constraints:\n\
        sod scope 1 2\n\
        sod scope 0 1\n\
        bod scope 2 3\n\
        sod scope 6 7\n\
        bod scope 8 9\n", step_size);

        let cursor = Cursor::new(content);
        let mut ud_preds = binary_predicates::default();
        let mut auth_sets = ud_preds.read_constraints(cursor).unwrap();
        auth_sets = (0..step_size)
        .map(|step| {
            auth_sets
                .iter()
                .enumerate()
                .filter(|(_, users)| users[step] == 1)
                .map(|(index, _)| index)
                .collect()
        })
        .collect();
    
        let mut g = graph::new(step_size);
    
        let ui_preds = binary_predicates::default();
        let ud_scope: Vec<usize> = (0..step_size).collect();
    
        let res = match plan_all(
            &mut g,
            &auth_sets,
            &ud_preds,
            &ud_scope,
            &ui_preds,
            &vec![1, 2, 3],
            3,
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

}

// 0. Read the GDPR paper
// 1. Generating (random or smart workset) policies
// 2. Work a bit on the analytical prooving of the combination of the algorithms
// 3. AST vs API/interface
//  
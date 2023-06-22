use crate::workflow::graph;

pub struct binary_predicates {
    preds: Vec<Box<dyn Fn(&graph) -> bool>>,
    // predicate location id (to identify sites' predicates)
    pred_loc: Vec<i32>
}

impl binary_predicates {
    pub fn eval(&self, g: &graph) -> bool {
        for p in self.preds.iter() {
            if !p(g) {
                return false;
            }
        }

        true
    }

    pub fn filtered_eval(&self, g: &graph, filtered_in: i32) -> bool {
        for (i, p) in self.preds.iter().enumerate() {
            if self.pred_loc[i] != filtered_in {continue;}
            
            if !p(g) {
                return false;
            }
        }

        true
    }
}

pub fn generate_mock_ui_predicates() -> binary_predicates {
    let mut v: Vec<Box<dyn Fn(&graph) -> bool>> = Vec::new();
    v.push(Box::new(|g:&graph| {
        // policy: don't allow a step and its child step to be same:
        for (v, adj) in g.adjacency_list.iter().enumerate() {
            if g.nodes_id[v] == -1 {continue;}
            for neigh in adj.iter() {
                if g.nodes_id[v] == g.nodes_id[*neigh] {return false;}
            }
        }

        return true;
    }));

    let mut ids:Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i)
    }

    binary_predicates { preds: v, pred_loc: ids }
}

pub fn generate_mock_ud_predicates() -> binary_predicates {
    let mut v: Vec<Box<dyn Fn(&graph) -> bool>> = Vec::new();

    v.push(Box::new(|_| true));
    v.push(Box::new(|g:&graph| {
        g.nodes_id.len() < 2 || g.nodes_id[2] == -1 || g.nodes_id[2] == 1
    }));
    v.push(Box::new(|g:&graph| {
        g.nodes_id.len() < 1 || g.nodes_id[1] == -1 || g.nodes_id[1] == 2
    }));

    let mut ids:Vec<i32> = Vec::new();
    for i in 0..v.len() as i32 {
        ids.push(i)
    }

    binary_predicates { preds: v, pred_loc: ids }
}

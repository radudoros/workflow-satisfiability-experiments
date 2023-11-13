use crate::workflow::Graph;

type BinaryPredicate = dyn Fn(&Graph) -> bool;

pub struct ScopedPredicate {
    pub pred: Box<BinaryPredicate>,
    pub scope: Vec<usize>, // these should be sorted by the order of exploration
}

impl ScopedPredicate {
    pub fn eval_smart(&self, g: &Graph, idx: usize) -> bool {
        if !self.scope.contains(&idx) {
            return true;
        }

        if !(self.pred)(g) {
            return false;
        }

        true
    }

    pub fn eval(&self, g: &Graph) -> bool {
        if !(self.pred)(g) {
            return false;
        }

        true
    }
    
    pub fn get_prev(&self, idx: usize) -> Option<usize> {
        for (i, node_idx) in self.scope.iter().enumerate() {
            if node_idx == &idx {
                return if i == 0 {None} else {Some(self.scope[i - 1])}
            }
        }

        None
    }
}

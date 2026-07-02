use sha2::{Sha256, Digest};
use std::collections::HashMap;

pub struct MerkleTree {
    leaves: Vec<String>,
    tree: Vec<Vec<String>>,
    root: String,
    leaf_map: HashMap<String, usize>,
}

impl MerkleTree {
    pub fn new(data: Vec<(i64, i128)>) -> Self {
        let mut leaves = Vec::new();
        let mut leaf_map = HashMap::new();
        
        for (i, (id, amount)) in data.iter().enumerate() {
            let leaf = Self::hash_leaf(*id, *amount);
            leaves.push(leaf.clone());
            leaf_map.insert(leaf, i);
        }

        let tree = Self::build_tree(&leaves);
        let root = tree.last().and_then(|level| level.first()).cloned().unwrap_or_default();

        Self {
            leaves,
            tree,
            root,
            leaf_map,
        }
    }

    fn hash_leaf(id: i64, amount: i128) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.to_be_bytes());
        hasher.update(amount.to_be_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn hash_pair(left: &str, right: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(left.as_bytes());
        hasher.update(right.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn build_tree(leaves: &[String]) -> Vec<Vec<String>> {
        let mut tree = Vec::new();
        let mut current_level = leaves.to_vec();

        tree.push(current_level.clone());

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                if i + 1 < current_level.len() {
                    next_level.push(Self::hash_pair(&current_level[i], &current_level[i + 1]));
                } else {
                    next_level.push(current_level[i].clone());
                }
            }
            
            tree.push(next_level.clone());
            current_level = next_level;
        }

        tree
    }

    pub fn get_root(&self) -> String {
        self.root.clone()
    }

    pub fn get_proof(&self, id: i64, amount: i128) -> Vec<String> {
        let leaf = Self::hash_leaf(id, amount);
        let mut proof = Vec::new();
        
        let mut index = match self.leaf_map.get(&leaf) {
            Some(idx) => *idx,
            None => return proof,
        };

        for level in 0..self.tree.len() - 1 {
            let level_size = self.tree[level].len();
            let sibling_index = if index % 2 == 0 {
                if index + 1 < level_size { index + 1 } else { index }
            } else {
                index - 1
            };

            if sibling_index < level_size {
                proof.push(self.tree[level][sibling_index].clone());
            }

            index /= 2;
        }

        proof
    }

    pub fn verify_proof(&self, leaf: &str, proof: &[String]) -> bool {
        let mut computed = leaf.to_string();
        
        for sibling in proof {
            computed = Self::hash_pair(&computed, sibling);
        }

        computed == self.root
    }
}
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize}; 
use near_sdk::collections::{
    LookupMap, 
    Vector
}; 
use near_sdk::{
    env, 
    near_bindgen, PanicOnDefault
};

use serde::{Serialize, Deserialize}; 

near_sdk::setup_alloc!();

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Manager {
}

type NodeId = String; 

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Node {
    pub id: NodeId, 
    pub title: String, 
    pub notes: String, 
    pub intrinsic_value: f64, 
    pub owner: String, 
    pub managers: Vector<Manager>, 
    pub flows_into: Vector<NodeId>, 
    pub flows_from: Vector<NodeId>
}

#[derive(Serialize)]
pub struct NodeView {
    pub id: NodeId, 
    pub title: String, 
    pub notes: String, 
    pub intrinsic_value: f64, 
    pub owner: String, 
}

// messages 
#[derive(Deserialize)]
pub struct NodeCreation {
    id: NodeId, 
    title: String, 
    notes: String, 
    intrinsic_value: f64, 
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Summits {
    nodes: LookupMap<String, Node>, 
}

#[near_bindgen]
impl Summits {
    #[init]
    pub fn new() -> Self {
        Self {
            nodes: LookupMap::new(b"nodes".to_vec())
        }
    }

    fn make_storage_key(key: &str, id: &NodeId) -> Vec<u8> {
        format!("node[{}].{}", id, key).as_bytes().to_vec() 
    }

    // #[payable]
    pub fn add_node(&mut self, node_creation: NodeCreation) -> Result<(), String> {
        let account_id = env::signer_account_id(); 
        if self.nodes.contains_key(&node_creation.id) {
            Err(format!("a node with id {} already exists", node_creation.id))
        } else {
            // if(node_creation.intrinsic_value > 0) {
            // }
            self.nodes.insert(&node_creation.id.clone(), &Node {
                id: node_creation.id.clone(), 
                title: node_creation.title, 
                notes: node_creation.notes, 
                intrinsic_value: node_creation.intrinsic_value, 
                owner: account_id, 
                managers: Vector::new(Self::make_storage_key("managers", &node_creation.id)), 
                flows_from: Vector::new(Self::make_storage_key("flows_from", &node_creation.id)), 
                flows_into: Vector::new(Self::make_storage_key("flows_into", &node_creation.id)), 
            });
            Ok(())
        }
    }
    pub fn update_node(&mut self) {
        
    }
    pub fn get_node(&self, node_id: String) -> Result<NodeView, String> {
        match self.nodes.get(&node_id) {
            Some(node) => Ok(NodeView {
                id: node.id, 
                title: node.title, 
                notes: node.notes, 
                owner: node.owner, 
                intrinsic_value: node.intrinsic_value
            }), 
            None => Err(
                format!("could not find node by provided id {}", node_id)
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

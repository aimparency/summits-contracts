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

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Serialize, Deserialize)]
pub struct FlowKey {
    from_id: NodeId, 
    into_id: NodeId
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
pub struct Flow {
    from_id: NodeId, 
    into_id: NodeId, 
    notes: String, 
    share: f32
}

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
#[derive(Serialize, Deserialize)]
pub struct NodeUpdate {
    id: NodeId, 
    title: Option<String>, 
    notes: Option<String>, 
    intrinsic_value: Option<f64>, 
}

#[derive(Serialize, Deserialize)]
pub struct FlowUpdate {
    from_id: NodeId, 
    into_id: NodeId, 
    notes: Option<String>, 
    share: Option<f32>, 
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Summits {
    nodes: LookupMap<String, Node>, 
    flows: LookupMap<FlowKey, Flow>, 
}

#[near_bindgen]
impl Summits {
    #[init]
    pub fn new() -> Self {
        Self {
            nodes: LookupMap::new(b"nodes".to_vec()), 
            flows: LookupMap::new(b"flows".to_vec())
        }
    }

    fn make_storage_key(key: &str, id: &NodeId) -> Vec<u8> {
        format!("node[{}].{}", id, key).as_bytes().to_vec() 
    }

    // #[payable]
    pub fn add_node(&mut self, node_creation: NodeUpdate) -> Result<(), String> {
        let account_id = env::signer_account_id(); 
        if self.nodes.contains_key(&node_creation.id) {
            Err(format!("a node with id {} already exists", node_creation.id))
        } else {
            // if(node_creation.intrinsic_value > 0) {
            // }
            self.nodes.insert(&node_creation.id.clone(), &Node {
                id: node_creation.id.clone(), 
                title: node_creation.title.unwrap_or("".to_string()), 
                notes: node_creation.notes.unwrap_or("".to_string()), 
                intrinsic_value: node_creation.intrinsic_value.unwrap_or(0.), 
                owner: account_id, 
                managers: Vector::new(Self::make_storage_key("managers", &node_creation.id)), 
                flows_from: Vector::new(Self::make_storage_key("flows_from", &node_creation.id)), 
                flows_into: Vector::new(Self::make_storage_key("flows_into", &node_creation.id)), 
            });
            Ok(())
        }
    }

    pub fn update_node(&mut self, node_update: NodeUpdate) -> Result<(), String> {
        match self.nodes.get(&node_update.id) {
            Some(mut node) => {
                if let Some(title) = node_update.title {
                    node.title = title
                }
                if let Some(notes) = node_update.notes {
                    node.notes = notes 
                }
                if let Some(intrinsic_value) = node_update.intrinsic_value {
                    node.intrinsic_value = intrinsic_value
                }
                Ok(())
            }, 
            None => Err(format!("could not find node with id {} for update", node_update.id))
        }
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), String> {
        match self.nodes.remove(&node_id) {
            Some(node) => {
                for from_id in node.flows_from.iter() {
                    self.remove_flow(FlowKey {
                        from_id,
                        into_id: node.id.clone()
                    }).ok();
                }
                for into_id in node.flows_into.iter() {
                    self.remove_flow(FlowKey {
                        from_id: node.id.clone(), 
                        into_id
                    }).ok();
                }
                Ok(())
            }
            None => Err(format!("could not find node with id {} for removal", node_id))
        }
    }

    pub fn add_flow(&mut self, flow_creation: FlowUpdate) -> Result<(), String> {
        match self.nodes.get(&flow_creation.from_id) {
            Some(mut from_node) => {
                match self.nodes.get(&flow_creation.into_id) {
                    Some(mut into_node) => {
                        let key = FlowKey {
                            from_id: flow_creation.from_id.clone(), 
                            into_id: flow_creation.into_id.clone()
                        };
                        self.flows.insert(&key, &Flow {
                            from_id: flow_creation.from_id, 
                            into_id: flow_creation.into_id, 
                            notes: flow_creation.notes.unwrap_or("".to_string()), 
                            share: flow_creation.share.unwrap_or(0.)
                        }); 
                        from_node.flows_into.push(&into_node.id); 
                        into_node.flows_from.push(&from_node.id); 
                        Ok(())
                    }, 
                    None => Err(
                        format!(
                            "could not add flow, can't find node with id {}", 
                            flow_creation.into_id
                        )
                    )
                }
            }, 
            None => Err(
                format!(
                    "could not add flow, can't find node with id {}", 
                    flow_creation.from_id
                )
            )
        }
    }

    pub fn remove_flow(&mut self, flow_key: FlowKey) -> Result<(), String> {
        if let Some(_) = self.flows.remove(&flow_key) {
            Ok(())
        } else {
            Err(format!(
                "flow not found, couldn't delete from {} to {}", 
                flow_key.from_id, 
                flow_key.into_id
            ))
        }
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

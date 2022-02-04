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
    pub account: String, 
    // permissions
}

type NodeId = String; 

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Serialize, Deserialize)]
pub struct FlowId {
    from: NodeId, 
    into: NodeId
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
pub struct Flow {
    id: FlowId, 
    dx: f32, 
    dy: f32, 
    notes: String, 
    share: f32
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Node {
    pub id: NodeId, 
    pub title: String, 
    pub notes: String, 
    pub deposit: u128, 
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
    pub deposit: u128, 
    pub owner: String, 
}

// messages 
#[derive(Serialize, Deserialize)]
pub struct NodeChanges {
    title: Option<String>, 
    notes: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct FlowUpdate {
    id: FlowId, 
    dx: Option<f32>, 
    dy: Option<f32>, 
    notes: Option<String>, 
    share: Option<f32>, 
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Summits {
    nodes: LookupMap<NodeId, Node>, 
    flows: LookupMap<FlowId, Flow>, 
    seven_summits: Vector<NodeId>
}

#[near_bindgen]
impl Summits {
    #[init]
    pub fn new() -> Self {
        Self {
            nodes: LookupMap::new(b"nodes".to_vec()), 
            flows: LookupMap::new(b"flows".to_vec()), 
            seven_summits: Vector::new(b"seven".to_vec()),
        }
    }

    fn make_storage_key(key: &str, id: &NodeId) -> Vec<u8> {
        format!("node[{}].{}", id, key).as_bytes().to_vec() 
    }

    // #[payable in gov coin]
    pub fn create_node(
        &mut self, 
        id: NodeId, 
        title: String, 
        notes: String, 
        deposit: u128 
        ) -> Result<(), String> {
        let account_id = env::signer_account_id(); 
        if self.nodes.contains_key(&id) {
            Err(format!("a node with id {} already exists", id))
        } else {
            // deduct funds according to node_creation.intrinsic_value
            
            self.nodes.insert(&id.clone(), &Node {
                id: id.clone(), 
                title, 
                notes, 
                deposit, 
                owner: account_id, 
                managers: Vector::new(Self::make_storage_key("managers", &id)), 
                flows_from: Vector::new(Self::make_storage_key("flows_from", &id)), 
                flows_into: Vector::new(Self::make_storage_key("flows_into", &id)), 
            });

            self.seven_summits_surpassing(id); 

            Ok(())
        }
    }

    fn seven_summits_surpassing(&mut self, id: NodeId) {
        if self.seven_summits.len() < 7 {
            self.seven_summits.push(&id)
        }
    }

    pub fn deposit_value (&mut self, node_id: NodeId, value: u128) -> Result<(), String> {
        match self.nodes.get(&node_id) {
            Some(mut node) => {
                // TODO: check funds, deduct 
                node.deposit += value; 
                self.nodes.insert(&node_id, &node); 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                node_id
            ))
        }
    }

    pub fn withdraw_value (&mut self, node_id: NodeId, value: u128) -> Result<(), String> {
        match self.nodes.get(&node_id) {
            Some(mut node) => {
                // TODO: payout funds
                node.deposit -= value; 
                self.nodes.insert(&node_id, &node); 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                node_id
            ))
        }
    }

    pub fn change_node(&mut self, node_id: NodeId, changes: NodeChanges) -> Result<(), String> {
        match self.nodes.get(&node_id) {
            Some(mut node) => {
                if let Some(title) = changes.title {
                    node.title = title
                }
                if let Some(notes) = changes.notes {
                    node.notes = notes 
                }
                self.nodes.insert(&node_id, &node); 
                Ok(())
            }, 
            None => Err(format!("could not find node with id {} for update", node_id))
        }
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> Result<(), String> {
        match self.nodes.remove(&node_id) {
            Some(node) => {
                for from_id in node.flows_from.iter() {
                    self.remove_flow(FlowId {
                        from: from_id, 
                        into: node.id.clone()
                    }).ok();
                }
                for into_id in node.flows_into.iter() {
                    self.remove_flow(FlowId {
                        from: node.id.clone(), 
                        into: into_id
                    }).ok();
                }
                Ok(())
            }
            None => Err(format!("could not find node with id {} for removal", node_id))
        }
    }

    pub fn create_flow(
        &mut self, 
        id: FlowId, 
        dx: f32, 
        dy: f32, 
        notes: String, 
        share: f32, 
    ) -> Result<(), String> {
        match self.nodes.get(&id.from) {
            Some(mut from_node) => {
                match self.nodes.get(&id.into) {
                    Some(mut into_node) => {
                        let id = FlowId {
                            from: id.from.clone(), 
                            into: id.into.clone()
                        };
                        self.flows.insert(&id, &Flow {
                            id: FlowId {
                                from: id.from.clone(), 
                                into: id.into.clone(), 
                            }, 
                            dx, 
                            dy, 
                            notes, 
                            share
                        }); 
                        from_node.flows_into.push(&into_node.id); 
                        into_node.flows_from.push(&from_node.id); 
                        self.nodes.insert(&from_node.id, &from_node); 
                        self.nodes.insert(&into_node.id, &into_node); 
                        Ok(())
                    }, 
                    None => Err(
                        format!(
                            "could not add flow, can't find node with id {}", 
                            id.into
                        )
                    )
                }
            }, 
            None => Err(
                format!(
                    "could not add flow, can't find node with id {}", 
                    id.from
                )
            )
        }
    }

    pub fn change_flow(&mut self, flow_update: FlowUpdate) -> Result<(), String> {
        match self.flows.get(&flow_update.id) {
            Some(mut flow) => {
                if let Some(dx) = flow_update.dx {
                    flow.dx = dx; 
                }
                if let Some(dy) = flow_update.dy {
                    flow.dy = dy; 
                }
                if let Some(notes) = flow_update.notes {
                    flow.notes = notes; 
                }
                if let Some(share) = flow_update.share {
                    flow.share = share; 
                }
                Ok(())
            }, 
            None => Err(format!( 
                "could not find flow from {} into {}", 
                flow_update.id.from, 
                flow_update.id.into
            ))
        }
    }

    pub fn remove_flow(&mut self, flow_id: FlowId) -> Result<(), String> {
        if let Some(_) = self.flows.remove(&flow_id) {
            Ok(())
        } else {
            Err(format!(
                "flow not found, couldn't delete from {} to {}", 
                flow_id.from, 
                flow_id.into
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
                deposit: node.deposit
            }), 
            None => Err(
                format!("could not find node by provided id {}", node_id)
            )
        }
    }

    pub fn get_node_flows(&self, node_id: String) -> Result<Vec<Flow>, String> {
        match self.nodes.get(&node_id) {
            Some(node) => {
                let mut result = vec![];
                for from_id in node.flows_from.iter() {
                    match self.flows.get(&FlowId {
                        from: from_id, 
                        into: node_id.clone()
                    }) {
                        Some(flow) => {
                            result.push(flow)
                        }, 
                        None => {
                        }
                    }
                }
                for into_id in node.flows_into.iter() {
                    match self.flows.get(&FlowId {
                        from: node_id.clone(), 
                        into: into_id
                    }) {
                        Some(flow) => {
                            result.push(flow)
                        }, 
                        None => {
                        }
                    }
                }
                Ok(result)
            }, 
            None => Err(format!(
                "could not find node {}", 
                node_id
            ))
        }
    }

    pub fn get_flow(&self, flow_id: FlowId) -> Result<Flow, String> {
        match self.flows.get(&flow_id) {
            Some(flow) => Ok(flow), 
            None => Err(format!(
                "could not find flow from {} to {}", 
                flow_id.from, 
                flow_id.into
            ))
        }
    }

    pub fn get_seven_summits(&self) -> Result<Vec<NodeId>, String> {
        Ok(self.seven_summits.iter().collect())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

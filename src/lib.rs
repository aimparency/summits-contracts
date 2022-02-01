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

type NodeKey = String; 

#[derive(BorshSerialize, BorshDeserialize)]
#[derive(Serialize, Deserialize)]
pub struct FlowKey {
    from_id: NodeKey, 
    into_id: NodeKey
}

#[derive(BorshDeserialize, BorshSerialize)]
#[derive(Serialize, Deserialize)]
pub struct Flow {
    from_id: NodeKey, 
    into_id: NodeKey, 
    dx: f32, 
    dy: f32, 
    notes: String, 
    share: f32
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Node {
    pub id: NodeKey, 
    pub title: String, 
    pub notes: String, 
    pub deposit: u128, 
    pub owner: String, 
    pub managers: Vector<Manager>, 
    pub flows_into: Vector<NodeKey>, 
    pub flows_from: Vector<NodeKey>
}

#[derive(Serialize)]
pub struct NodeView {
    pub id: NodeKey, 
    pub title: String, 
    pub notes: String, 
    pub deposit: u128, 
    pub owner: String, 
}

// messages 
#[derive(Serialize, Deserialize)]
pub struct NodeUpdate {
    id: NodeKey, 
    title: Option<String>, 
    notes: Option<String>
}


#[derive(Serialize, Deserialize)]
pub struct FlowCreation {
    key: FlowKey, 
    dx: f32, 
    dy: f32, 
    notes: String, 
    share: f32, 
}

#[derive(Serialize, Deserialize)]
pub struct FlowUpdate {
    key: FlowKey, 
    dx: Option<f32>, 
    dy: Option<f32>, 
    notes: Option<String>, 
    share: Option<f32>, 
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Summits {
    nodes: LookupMap<NodeKey, Node>, 
    flows: LookupMap<FlowKey, Flow>, 
    seven_summits: Vector<NodeKey>
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

    fn make_storage_key(key: &str, id: &NodeKey) -> Vec<u8> {
        format!("node[{}].{}", id, key).as_bytes().to_vec() 
    }

    // #[payable in gov coin]
    pub fn create_node_with_value(
        &mut self, 
        id: NodeKey, 
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

    // pub fn create_node_with_intial_flow( ... ) 

    fn seven_summits_surpassing(&mut self, id: NodeKey) {
        if self.seven_summits.len() < 7 {
            self.seven_summits.push(&id)
        }
    }

    pub fn deposit_value (&mut self, node_id: NodeKey, value: u128) -> Result<(), String> {
        match self.nodes.get(&node_id) {
            Some(mut node) => {
                // TODO: check funds, deduct 
                node.deposit += value; 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                node_id
            ))
        }
    }

    pub fn withdraw_value (&mut self, node_id: NodeKey, value: u128) -> Result<(), String> {
        match self.nodes.get(&node_id) {
            Some(mut node) => {
                // TODO: payout funds
                node.deposit -= value; 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                node_id
            ))
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
                Ok(())
            }, 
            None => Err(format!("could not find node with id {} for update", node_update.id))
        }
    }

    pub fn remove_node(&mut self, node_id: NodeKey) -> Result<(), String> {
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

    pub fn create_flow(&mut self, flow_creation: FlowCreation) -> Result<(), String> {
        match self.nodes.get(&flow_creation.key.from_id) {
            Some(mut from_node) => {
                match self.nodes.get(&flow_creation.key.into_id) {
                    Some(mut into_node) => {
                        let key = FlowKey {
                            from_id: flow_creation.key.from_id.clone(), 
                            into_id: flow_creation.key.into_id.clone()
                        };
                        self.flows.insert(&key, &Flow {
                            from_id: flow_creation.key.from_id, 
                            into_id: flow_creation.key.into_id, 
                            dx: flow_creation.dx, 
                            dy: flow_creation.dy, 
                            notes: flow_creation.notes, 
                            share: flow_creation.share
                        }); 
                        from_node.flows_into.push(&into_node.id); 
                        into_node.flows_from.push(&from_node.id); 
                        Ok(())
                    }, 
                    None => Err(
                        format!(
                            "could not add flow, can't find node with id {}", 
                            flow_creation.key.into_id
                        )
                    )
                }
            }, 
            None => Err(
                format!(
                    "could not add flow, can't find node with id {}", 
                    flow_creation.key.from_id
                )
            )
        }
    }

    pub fn update_flow(&mut self, flow_update: FlowUpdate) -> Result<(), String> {
        match self.flows.get(&flow_update.key) {
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
                flow_update.key.from_id, 
                flow_update.key.into_id
            ))
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
                    match self.flows.get(&FlowKey {
                        from_id, 
                        into_id: node_id.clone()
                    }) {
                        Some(flow) => {
                            result.push(flow)
                        }, 
                        None => {
                        }
                    }
                }
                for into_id in node.flows_into.iter() {
                    match self.flows.get(&FlowKey {
                        from_id: node_id.clone(), 
                        into_id
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

    pub fn get_flow(&self, flow_key: FlowKey) -> Result<Flow, String> {
        match self.flows.get(&flow_key) {
            Some(flow) => Ok(flow), 
            None => Err(format!(
                "could not find flow from {} to {}", 
                flow_key.from_id, 
                flow_key.into_id
            ))
        }
    }

    pub fn get_seven_summits(&self) -> Result<Vec<NodeKey>, String> {
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

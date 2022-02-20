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
    pub deposit: f64, // this may be u128 in the to represent currency
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
    pub deposit: f64, 
    pub owner: String, 
}

// messages 
#[derive(Serialize, Deserialize)]
pub struct NodeChanges {
    title: Option<String>, 
    notes: Option<String>, 
    deposit: Option<f64>
}

#[derive(Serialize, Deserialize)]
pub struct FlowChanges{
    dx: Option<f32>, 
    dy: Option<f32>, 
    notes: Option<String>, 
    share: Option<f32>, 
}

#[derive(Serialize, Deserialize)]
pub struct WrappedFlowChanges {
    id: FlowId,  
    changes: FlowChanges
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Summits {
    nodes: LookupMap<NodeId, Node>, 
    flows: LookupMap<FlowId, Flow>, 
    home_node_id: Option<NodeId>
}

#[near_bindgen]
impl Summits {
    #[init]
    pub fn new() -> Self {
        Self {
            nodes: LookupMap::new(b"nodes".to_vec()), 
            flows: LookupMap::new(b"flows".to_vec()), 
            home_node_id: None,
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
        deposit: f64
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
                owner: account_id.to_string(), 
                managers: Vector::new(Self::make_storage_key("managers", &id)), 
                flows_from: Vector::new(Self::make_storage_key("flows_from", &id)), 
                flows_into: Vector::new(Self::make_storage_key("flows_into", &id)), 
            });

            Ok(())
        }
    }

    pub fn deposit_value (&mut self, id: NodeId, value: f64) -> Result<(), String> {
        match self.nodes.get(&id) {
            Some(mut node) => {
                // TODO: check funds, deduct 
                node.deposit += value; 
                self.nodes.insert(&id, &node); 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                id
            ))
        }
    }

    pub fn withdraw_value (&mut self, id: NodeId, value: f64) -> Result<(), String> {
        match self.nodes.get(&id) {
            Some(mut node) => {
                // TODO: payout funds
                node.deposit -= value; 
                self.nodes.insert(&id, &node); 
                Ok(())
            }, 
            None => Err(format!(
                "could not find node with id {}", 
                id 
            ))
        }
    }

    pub fn change_node(
        &mut self, 
        id: NodeId, 
        changes: NodeChanges, 
    ) -> Result<(), String> {
        match self.nodes.get(&id) {
            Some(mut node) => {
                if let Some(title) = changes.title {
                    node.title = title
                }
                if let Some(notes) = changes.notes {
                    node.notes = notes 
                }
                if let Some(deposit) = changes.deposit {
                    node.deposit = deposit 
                }
                self.nodes.insert(&id, &node); 
                Ok(())
            }, 
            None => Err(format!("could not find node with id {} for update", id))
        }
    }

    pub fn remove_node(&mut self, id: NodeId) -> Result<(), String> {
        if let Some(home_id) = &self.home_node_id {
            if home_id.eq(&id) {
                return Err(format!(
                    "cannot remove home node, it's your anchor to the graph"  
                ))
            }
        }
        match self.nodes.remove(&id) {
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
                if let Some(home_id) = self.home_node_id.clone() {
                    if home_id == id {
                        self.home_node_id = None
                    }
                }
                Ok(())
            }
            None => Err(format!("could not find node with id {} for removal", id))
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

    pub fn change_flow(&mut self, id: FlowId, changes: FlowChanges) -> Result<(), String> {
        match self.flows.get(&id) {
            Some(mut flow) => {
                if let Some(dx) = changes.dx {
                    flow.dx = dx; 
                }
                if let Some(dy) = changes.dy {
                    flow.dy = dy; 
                }
                if let Some(notes) = changes.notes {
                    flow.notes = notes; 
                }
                if let Some(share) = changes.share {
                    flow.share = share; 
                }
                self.flows.insert(&id, &flow); 
                Ok(())
            }, 
            None => Err(format!( 
                "could not find flow from {} into {}", 
                id.from, 
                id.into
            ))
        }
    }
    pub fn bulk_change_flow(&mut self, bulk: Vec<WrappedFlowChanges>) -> Result<(), String> {
        let mut errors = Vec::new();
        for wrapped_change in bulk {
            match self.change_flow(
                wrapped_change.id, 
                wrapped_change.changes
            ) {
                Err(err) => {
                    errors.push(err) 
                }, 
                _ => ()
            }
        }
        if errors.len() == 0 {
            Ok(())
        } else {
            Err(errors.join(". "))
        }
    }

    pub fn remove_flow(&mut self, id: FlowId) -> Result<(), String> {
        if let Some(_) = self.flows.remove(&id) {
            Ok(())
        } else {
            Err(format!(
                "flow not found, couldn't delete from {} to {}", 
                id.from, 
                id.into
            ))
        }
    }

    pub fn get_node(&self, id: String) -> Result<NodeView, String> {
        match self.nodes.get(&id) {
            Some(node) => Ok(NodeView {
                id: node.id, 
                title: node.title, 
                notes: node.notes, 
                owner: node.owner, 
                deposit: node.deposit
            }), 
            None => Err(
                format!("could not find node by provided id {}", id)
            )
        }
    }

    pub fn get_node_flows(&self, id: String) -> Result<Vec<Flow>, String> {
        match self.nodes.get(&id) {
            Some(node) => {
                let mut result = vec![];
                for from_id in node.flows_from.iter() {
                    match self.flows.get(&FlowId {
                        from: from_id, 
                        into: id.clone()
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
                        from: id.clone(), 
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
                id 
            ))
        }
    }

    pub fn get_flow(&self, id: FlowId) -> Result<Flow, String> {
        match self.flows.get(&id) {
            Some(flow) => Ok(flow), 
            None => Err(format!(
                "could not find flow from {} to {}", 
                id.from, 
                id.into
            ))
        }
    }

    pub fn set_home_node_id(&mut self, id: NodeId) -> Result<(), String> {
        self.home_node_id = Some(id);
        Ok(())
    }

    pub fn get_home_node_id(&self) -> Result<NodeId, String> {
        match &self.home_node_id {
            Some(id) => {
                Ok(id.into())
            }, 
            None => {
                Err("Home node id not set".to_string()) 
            }
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

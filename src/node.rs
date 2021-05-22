use crate::blackboard::Blackboard;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
enum Status {
    Failure,
    Success,
    Running,
}

enum NodeType {
    Sequence,
    Condition,
    Action,
}

struct Node {
    id: u64,
    node_type: NodeType,
    children: Vec<Node>,
}

impl Node {
    fn new(id: u64, node_type: NodeType) -> Self {
        Node {
            id,
            node_type,
            children: Vec::new(),
        }
    }
}

pub struct Condition<'a> {
    cb: Box<dyn Fn(&Blackboard) -> bool + 'a>,
}

impl<'a> Condition<'a> {
    pub fn new(cb: impl Fn(&Blackboard) -> bool + 'a) -> Self {
        Condition { cb: Box::new(cb) }
    }

    pub fn evaluate(&self, blackboard: &Blackboard) -> bool {
        (self.cb)(blackboard)
    }
}

pub struct ConditionMap<'a> {
    conditions: HashMap<u64, Condition<'a>>,
}

impl<'a> ConditionMap<'a> {
    pub fn new() -> Self {
        Self {
            conditions: HashMap::new(),
        }
    }

    pub fn add_condition(&mut self, node_id: u64, condition: Condition<'a>) {
        self.conditions.insert(node_id, condition);
    }

    pub fn get_condition(&self, node_id: u64) -> &Condition<'a> {
        self.conditions.get(&node_id).unwrap()
    }
}

pub struct Action<'a> {
    cb: Box<dyn FnMut(&mut Blackboard) -> Status + 'a>,
}

impl<'a> Action<'a> {
    pub fn new(cb: impl FnMut(&mut Blackboard) -> Status + 'a) -> Self {
        Action { cb: Box::new(cb) }
    }

    pub fn execute(&mut self, blackboard: &mut Blackboard) -> Status {
        (self.cb)(blackboard)
    }
}

pub struct ActionMap<'a> {
    actions: HashMap<u64, Action<'a>>,
}

impl<'a> ActionMap<'a> {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn add_action(&mut self, node_id: u64, action: Action<'a>) {
        self.actions.insert(node_id, action);
    }

    pub fn get_action(&mut self, node_id: u64) -> &mut Action<'a> {
        self.actions.get_mut(&node_id).unwrap()
    }
}

fn tick(
    node: &Node,
    blackboard: &mut Blackboard,
    condition_map: &ConditionMap,
    action_map: &mut ActionMap,
) -> Status {
    match node.node_type {
        NodeType::Sequence => execute_sequence_node(node, blackboard, condition_map, action_map),
        NodeType::Condition => execute_condition_node(node, blackboard, condition_map),
        NodeType::Action => execute_action_node(node, blackboard, action_map),
    }
}

fn execute_sequence_node(
    node: &Node,
    blackboard: &mut Blackboard,
    condition_map: &ConditionMap,
    action_map: &mut ActionMap,
) -> Status {
    for child_node in node.children.iter() {
        let status = tick(&child_node, blackboard, condition_map, action_map);
        if status == Status::Running {
            return Status::Running;
        } else if status == Status::Failure {
            return Status::Failure;
        }
    }
    return Status::Success;
}

fn execute_condition_node(
    node: &Node,
    blackboard: &Blackboard,
    condition_map: &ConditionMap,
) -> Status {
    let condition = condition_map.get_condition(node.id);
    if condition.evaluate(blackboard) {
        return Status::Success;
    } else {
        return Status::Failure;
    }
}

fn execute_action_node(
    node: &Node,
    blackboard: &mut Blackboard,
    action_map: &mut ActionMap,
) -> Status {
    let action = action_map.get_action(node.id);
    action.execute(blackboard)
}

#[cfg(test)]
mod tests {
    use crate::blackboard::Blackboard;
    use crate::node::{tick, Condition, ConditionMap, Node, NodeType, Status};

    #[test]
    fn test_condition() {
        let blackboard = Blackboard::new();
        let condition = Condition::new(|_b| true);
        assert!(condition.evaluate(&blackboard));
    }

    #[test]
    fn test_condition_node_returning_true() {
        let node = Node::new(42, NodeType::Condition);

        let blackboard = Blackboard::new();
        let mut condition_map = ConditionMap::new();

        condition_map.add_condition(node.id, Condition::new(|_b| true));

        let status = tick(&node, &blackboard, &condition_map);

        assert_eq!(status, Status::Success);
    }

    #[test]
    fn test_condition_node_returning_false() {
        let node = Node::new(42, NodeType::Condition);

        let blackboard = Blackboard::new();
        let mut condition_map = ConditionMap::new();

        condition_map.add_condition(node.id, Condition::new(|_b| false));

        let status = tick(&node, &blackboard, &condition_map);

        assert_eq!(status, Status::Failure);
    }

    #[test]
    fn test_sequence_node_with_one_child() {
        let mut parent_node = Node::new(1, NodeType::Sequence);
        let child_node = Node::new(2, NodeType::Condition);

        let blackboard = Blackboard::new();
        let mut condition_map = ConditionMap::new();
        condition_map.add_condition(child_node.id, Condition::new(|_b| true));

        parent_node.children.push(child_node);

        let status = tick(&parent_node, &blackboard, &condition_map);

        assert_eq!(status, Status::Success);
    }

    #[test]
    fn test_sequence_node_with_two_children() {
        let mut parent_node = Node::new(1, NodeType::Sequence);
        let child_node1 = Node::new(2, NodeType::Condition);
        let child_node2 = Node::new(3, NodeType::Condition);

        let blackboard = Blackboard::new();
        let mut condition_map = ConditionMap::new();
        condition_map.add_condition(child_node1.id, Condition::new(|_b| true));
        condition_map.add_condition(child_node2.id, Condition::new(|_b| false));

        parent_node.children.push(child_node1);
        parent_node.children.push(child_node2);

        let status = tick(&parent_node, &blackboard, &condition_map);

        assert_eq!(status, Status::Failure);
    }
}

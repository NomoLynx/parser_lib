use crate::mermaid_flow::*;

fn is_node_name_function_name(node_name:&str) -> String {
    format!("is_{node_name}")
}

fn get_target_function_name(node_name:&str) -> String {
    format!("target_{node_name}")
}

fn get_var_name(node_id:NodeId) -> String {
    let var_name = "n";
    format!("{var_name}{}", node_id.0)
}

fn generate_logic_code_for_node(graph: &Graph<FCNode>, node_id:NodeId) -> String {
    let mut result = vec![];

    // visited set to track visited nodes, unvisited set to track unvisited nodes
    let mut visited_set = std::collections::HashSet::new();
    visited_set.insert(node_id);

    // define the target function
    result.push(format!("{}({}) <-- ", get_target_function_name(&graph.node(node_id).name), get_var_name(node_id)));

    // insert is_function for 1st node
    let rel_name = is_node_name_function_name(&graph.node(node_id).name);
    result.push(format!("{rel_name}({})", get_var_name(node_id)));

    // add all graph node's id to unvisited set
    let mut unvisited_set = std::collections::HashSet::new();
    for id in 0..graph.node_count() {
        let id = NodeId(id);
        if id != node_id {
            unvisited_set.insert(id);
        }
    }

    while unvisited_set.len() > 0 {
        // find all neighbours of visited nodes
        let mut neighbor_nodes = vec![];
        for visited_node in &visited_set {
            for node in graph.neighbor_nodes(*visited_node) {
                if !visited_set.contains(&node) {
                    neighbor_nodes.push((*visited_node, node));
                }
            }
        }

        if neighbor_nodes.len() == 0 {
            break;
        }

        // generate code for each neighbor node, and add them to visited set
        for (from, to) in neighbor_nodes {
            let from_node = graph.node(from);
            let from_node_var_name = get_var_name(from);
            let to_node_var_name = get_var_name(to);
            
            let rel_name = is_node_name_function_name(&from_node.name);
            let code = format!("{rel_name}({from_node_var_name}), link_function({from_node_var_name}, {to_node_var_name})");
            result.push(code);
            visited_set.insert(to);
            unvisited_set.remove(&to);
        }
    }

    let statement = result.join(", ");
    format!("{statement};")
}

pub fn get_ascent_logic_code(flowchart: &FlowChartProgram, item_name: &str, target_node_id: &str) -> Result<Vec<String>, String> {
    let mut result = vec![format!("relation node({item_name});")];

    let graph = FlowchartToGraph::new()
        .convert(flowchart.get_stmts());

    let nodes = graph.get_nodes();

    // declare all nodes for this graph
    for node in nodes {
        let node_name = &node.name;
        let code = format!("{node_name}({item_name});");
        result.push(code);
    }

    // define nodes rules for this graph
    for node in nodes {
        let rel_name = is_node_name_function_name(&node.name);
        let code = format!("{rel_name}(item) <-- node(item), if item.{rel_name}(item);");
        result.push(code);
    }

    // find target node with target_node_id and add get target relation
    let target_node = graph.get_node_id_by_name(target_node_id)
                                                    .ok_or_else(|| format!("Cannot find target node with id '{}'", target_node_id))?;
    result.push(format!("relation {}({item_name});", get_target_function_name(&graph.node(target_node).name)));

    // define relationship from target node to all its neighbors
    let target_code = generate_logic_code_for_node(&graph, target_node);
    result.push(target_code);

    Ok(result)
}
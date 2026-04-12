use rust_macro::ini2hash;

use crate::mermaid_flow::*;

fn get_lib_code(item_name: &str) -> String {
    let definition = format!("relation single_child_of({item_name}, {item_name});\nrelation child_of({item_name}, {item_name});");

    let code = r#"
    single_child_of(child, parent) <-- 
        node(parent), node(child),
        if parent.is_parent_of(child) && parent.get_children().len() == 1;

    child_of(child, parent) <-- node(parent), node(child), if parent.is_parent_of(child);
"#;

    format!("{}\n{}", definition, code)
}

fn is_node_name_function_name(node_name:&str) -> Result<String, String> {
    let is_function_mapping = ini2hash!("src/mermaid_flow/is_clang_function.ini");
    let fn_name = is_function_mapping.get(node_name)
            .ok_or(format!("cannot find function mapping for key = '{node_name}'"))?;
    let r = format!("{fn_name}");
    Ok(r)
}

fn get_target_function_name(node_name:&str) -> String {
    format!("target_{node_name}")
}

fn get_var_name(node_id:NodeId) -> String {
    let var_name = "n";
    format!("{var_name}{}", node_id.0)
}

/// Generate logic code for the target node and its neighbors, with the target node as the root of the logic code.
fn get_link_function_name(graph: &Graph<FCNode>, from: NodeId, to: NodeId) -> Option<String> {

    if let Some(edge) = graph.edge_between(from, to) {
        // edge exists from 'from' to 'to', so 'to' is child of 'from'
        let fn_name = if edge.data.contains(& "1".to_string()) { "single_child_of" } else { "child_of" };
        Some(format!("{fn_name}({}, {})", get_var_name(to), get_var_name(from)))
    }
    else if let Some(edge) = graph.edge_between(to, from) {
        // edge exists from 'to' to 'from', so 'from' is child of 'to'
        let fn_name = if edge.data.contains(& "1".to_string()) { "single_child_of" } else { "child_of" };
        Some(format!("{fn_name}({}, {})", get_var_name(from), get_var_name(to)))
    }
    else {
        return None;
    }
}

fn generate_logic_code_for_node(graph: &Graph<FCNode>, node_id:NodeId) -> Result<String, String> {
    let mut result = vec![];

    // visited set to track visited nodes, unvisited set to track unvisited nodes
    let mut visited_set = std::collections::HashSet::new();
    visited_set.insert(node_id);

    // define the target function
    let stmt_start = format!("{}({}) <-- ", get_target_function_name(&graph.node(node_id).name), get_var_name(node_id));

    // insert is_function for 1st node
    let rel_name = is_node_name_function_name(&graph.node(node_id).name)?;
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
            let to_node = graph.node(to);
            let to_node_var_name = get_var_name(to.clone());
            let link_function = get_link_function_name(graph, from, to)
                                            .ok_or(format!("Cannot find edge from {} to {}", from.0, to.0))?;
            
            let rel_name = is_node_name_function_name(&to_node.name)?;
            let code = format!("{rel_name}({to_node_var_name}), {link_function}");
            result.push(code);
            visited_set.insert(to);
            unvisited_set.remove(&to);
        }
    }

    let statement = result.join(", \n\t");
    let r = format!("{stmt_start}\n\t{statement};");
    Ok(r)
}

pub fn get_ascent_logic_code(flowchart: &FlowChartProgram, item_name: &str, target_node_id: &str) -> Result<Vec<String>, String> {
    let mut result = vec![format!("relation node({item_name});")];


    // add lib code for node relationship
    let lib_code = get_lib_code(item_name);
    result.push(lib_code);

    let graph = FlowchartToGraph::new()
        .convert(flowchart.get_stmts());

    let nodes = graph.get_nodes();

    // declare all nodes for this graph
    for node in nodes {
        let node_name = is_node_name_function_name(&node.name)?;
        let code = format!("relation {node_name}({item_name});");
        result.push(code);
    }

    // define nodes rules for this graph
    for node in nodes {
        let rel_name = is_node_name_function_name(&node.name)?;
        let code = format!("{rel_name}(item) <-- node(item), if item.{rel_name}();");
        result.push(code);
    }

    // find target node with target_node_id and add get target relation
    let target_node = graph.get_node_id_by_name(target_node_id)
                                                    .ok_or_else(|| format!("Cannot find target node with id '{}'", target_node_id))?;
    result.push(format!("relation {}({item_name});", get_target_function_name(&graph.node(target_node).name)));

    // define relationship from target node to all its neighbors
    let target_code = generate_logic_code_for_node(&graph, target_node)?;
    result.push(target_code);

    Ok(result)
}
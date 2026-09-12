use open62541::{ObjectNode, VariableNode, ua};
use open62541_sys::{
    UA_NS0ID_BASEDATAVARIABLETYPE, UA_NS0ID_FOLDERTYPE, UA_NS0ID_OBJECTSFOLDER, UA_NS0ID_ORGANIZES,
    UA_NS0ID_STRING,
};

fn main() -> std::process::ExitCode {
    if let Err(_) = run() {
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (server, runner) = open62541::Server::new();

    let object_node_id = server.add_object_node(ObjectNode {
        requested_new_node_id: None,
        parent_node_id: ua::NodeId::ns0(UA_NS0ID_OBJECTSFOLDER),
        reference_type_id: ua::NodeId::ns0(UA_NS0ID_ORGANIZES),
        browse_name: ua::QualifiedName::new(1, "SomeFolder"),
        type_definition: ua::NodeId::ns0(UA_NS0ID_FOLDERTYPE),
        attributes: ua::ObjectAttributes::default(),
    })?;

    let variable_node_id = server.add_variable_node(VariableNode {
        requested_new_node_id: None,
        parent_node_id: object_node_id,
        reference_type_id: ua::NodeId::ns0(UA_NS0ID_ORGANIZES),
        browse_name: ua::QualifiedName::new(1, "SomeVariable"),
        type_definition: ua::NodeId::ns0(UA_NS0ID_BASEDATAVARIABLETYPE),
        attributes: ua::VariableAttributes::default()
            .with_data_type(&ua::NodeId::ns0(UA_NS0ID_STRING)),
    })?;

    server.write_value(
        &variable_node_id,
        &ua::Variant::scalar(ua::String::new("Lorem Ipsum")?),
    )?;

    runner.run_until_interrupt()?;
    Ok(())
}

use open62541::{ua, DataType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AttributeId {
    NodeId,
    NodeClass,
    BrowseName,
    DisplayName,
    Description,
    WriteMask,
    UserWriteMask,
    IsAbstract,
    Symmetric,
    InverseName,
    ContainsNoLoops,
    EventNotifier,
    Value,
    DataType,
    ValueRank,
    ArrayDimensions,
    AccessLevel,
    UserAccessLevel,
    MinimumSamplingInterval,
    Historizing,
    Executable,
    UserExecutable,
    DataTypeDefinition,
    RolePermissions,
    UserRolePermissions,
    AccessRestrictions,
    AccessLevelEx,
}

impl From<ua::AttributeId> for AttributeId {
    fn from(value: ua::AttributeId) -> Self {
        use open62541_sys::UA_AttributeId;
        match value.into_raw() {
            UA_AttributeId::UA_ATTRIBUTEID_NODEID => AttributeId::NodeId,
            UA_AttributeId::UA_ATTRIBUTEID_NODECLASS => AttributeId::NodeClass,
            UA_AttributeId::UA_ATTRIBUTEID_BROWSENAME => AttributeId::BrowseName,
            UA_AttributeId::UA_ATTRIBUTEID_DISPLAYNAME => AttributeId::DisplayName,
            UA_AttributeId::UA_ATTRIBUTEID_DESCRIPTION => AttributeId::Description,
            UA_AttributeId::UA_ATTRIBUTEID_WRITEMASK => AttributeId::WriteMask,
            UA_AttributeId::UA_ATTRIBUTEID_USERWRITEMASK => AttributeId::UserWriteMask,
            UA_AttributeId::UA_ATTRIBUTEID_ISABSTRACT => AttributeId::IsAbstract,
            UA_AttributeId::UA_ATTRIBUTEID_SYMMETRIC => AttributeId::Symmetric,
            UA_AttributeId::UA_ATTRIBUTEID_INVERSENAME => AttributeId::InverseName,
            UA_AttributeId::UA_ATTRIBUTEID_CONTAINSNOLOOPS => AttributeId::ContainsNoLoops,
            UA_AttributeId::UA_ATTRIBUTEID_EVENTNOTIFIER => AttributeId::EventNotifier,
            UA_AttributeId::UA_ATTRIBUTEID_VALUE => AttributeId::Value,
            UA_AttributeId::UA_ATTRIBUTEID_DATATYPE => AttributeId::DataType,
            UA_AttributeId::UA_ATTRIBUTEID_VALUERANK => AttributeId::ValueRank,
            UA_AttributeId::UA_ATTRIBUTEID_ARRAYDIMENSIONS => AttributeId::ArrayDimensions,
            UA_AttributeId::UA_ATTRIBUTEID_ACCESSLEVEL => AttributeId::AccessLevel,
            UA_AttributeId::UA_ATTRIBUTEID_USERACCESSLEVEL => AttributeId::UserAccessLevel,
            UA_AttributeId::UA_ATTRIBUTEID_MINIMUMSAMPLINGINTERVAL => AttributeId::MinimumSamplingInterval,
            UA_AttributeId::UA_ATTRIBUTEID_HISTORIZING => AttributeId::Historizing,
            UA_AttributeId::UA_ATTRIBUTEID_EXECUTABLE => AttributeId::Executable,
            UA_AttributeId::UA_ATTRIBUTEID_USEREXECUTABLE => AttributeId::UserExecutable,
            UA_AttributeId::UA_ATTRIBUTEID_DATATYPEDEFINITION => AttributeId::DataTypeDefinition,
            UA_AttributeId::UA_ATTRIBUTEID_ROLEPERMISSIONS => AttributeId::RolePermissions,
            UA_AttributeId::UA_ATTRIBUTEID_USERROLEPERMISSIONS => AttributeId::UserRolePermissions,
            UA_AttributeId::UA_ATTRIBUTEID_ACCESSRESTRICTIONS => AttributeId::AccessRestrictions,
            UA_AttributeId::UA_ATTRIBUTEID_ACCESSLEVELEX => AttributeId::AccessLevelEx,
            _ => unreachable!(),
        }
    }
}

impl From<AttributeId> for ua::AttributeId {
    fn from(value: AttributeId) -> Self {
        
        match value {
            AttributeId::NodeId => ua::AttributeId::NODEID,
            AttributeId::NodeClass => ua::AttributeId::NODECLASS,
            AttributeId::BrowseName => ua::AttributeId::BROWSENAME,
            AttributeId::DisplayName => ua::AttributeId::DISPLAYNAME,
            AttributeId::Description => ua::AttributeId::DESCRIPTION,
            AttributeId::WriteMask => ua::AttributeId::WRITEMASK,
            AttributeId::UserWriteMask => ua::AttributeId::USERWRITEMASK,
            AttributeId::IsAbstract => ua::AttributeId::ISABSTRACT,
            AttributeId::Symmetric => ua::AttributeId::SYMMETRIC,
            AttributeId::InverseName => ua::AttributeId::INVERSENAME,
            AttributeId::ContainsNoLoops => ua::AttributeId::CONTAINSNOLOOPS,
            AttributeId::EventNotifier => ua::AttributeId::EVENTNOTIFIER,
            AttributeId::Value => ua::AttributeId::VALUE,
            AttributeId::DataType => ua::AttributeId::DATATYPE,
            AttributeId::ValueRank => ua::AttributeId::VALUERANK,
            AttributeId::ArrayDimensions => ua::AttributeId::ARRAYDIMENSIONS,
            AttributeId::AccessLevel => ua::AttributeId::ACCESSLEVEL,
            AttributeId::UserAccessLevel => ua::AttributeId::USERACCESSLEVEL,
            AttributeId::MinimumSamplingInterval => ua::AttributeId::MINIMUMSAMPLINGINTERVAL,
            AttributeId::Historizing => ua::AttributeId::HISTORIZING,
            AttributeId::Executable => ua::AttributeId::EXECUTABLE,
            AttributeId::UserExecutable => ua::AttributeId::USEREXECUTABLE,
            AttributeId::DataTypeDefinition => ua::AttributeId::DATATYPEDEFINITION,
            AttributeId::RolePermissions => ua::AttributeId::ROLEPERMISSIONS,
            AttributeId::UserRolePermissions => ua::AttributeId::USERROLEPERMISSIONS,
            AttributeId::AccessRestrictions => ua::AttributeId::ACCESSRESTRICTIONS,
            AttributeId::AccessLevelEx => ua::AttributeId::ACCESSLEVELEX,
        }
    }
}   

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_id() {
        let id : AttributeId = ua::AttributeId::DATATYPE.into();
        assert_eq!(id, AttributeId::DataType);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"data-type\"");
    }
}

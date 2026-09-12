use open62541::{ua, DataType};
use open62541_sys::UA_AttributeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u32)]
pub enum AttributeId {
    NodeId = 1,
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
        let val = value.into_raw().0;
        // SAFETY: val is guaranteed to be within the valid range of AttributeId
        unsafe {
            std::mem::transmute(val)
        }
    }
}

impl From<AttributeId> for ua::AttributeId {
    fn from(value: AttributeId) -> Self {
        // SAFETY: value is guaranteed to be within the valid range of ua::AttributeId
        unsafe { ua::AttributeId::from_raw(UA_AttributeId(value as _)) }
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
        assert_eq!(json, "\"DataType\"");
    }
}

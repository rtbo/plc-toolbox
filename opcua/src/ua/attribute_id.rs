use crate::ffi;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum AttributeId {
    Invalid                 = 0,
    NodeId                  = 1,
    NodeClass               = 2,
    BrowseName              = 3,
    DisplayName             = 4,
    Description             = 5,
    WriteMask               = 6,
    UserWriteMask           = 7,
    IsAbstract              = 8,
    Symmetric               = 9,
    InverseName             = 10,
    ContainsNoLoops         = 11,
    EventNotifier           = 12,
    Value                   = 13,
    DataType                = 14,
    ValueRank               = 15,
    ArrayDimensions         = 16,
    AccessLevel             = 17,
    UserAccessLevel         = 18,
    MinimumSamplingInterval = 19,
    Historizing             = 20,
    Executable              = 21,
    UserExecutable          = 22,
    DataTypeDefinition      = 23,
    RolePermissions         = 24,
    UserRolePermissions     = 25,
    AccessRestrictions      = 26,
    AccessLevelEx           = 27
}

impl AttributeId {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn as_raw(&self) -> ffi::UA_AttributeId {
        ffi::UA_AttributeId(*self as u32)
    }
    
    /// Tries to convert a raw `ffi::UA_AttributeId` into an `AttributeId`.
    /// Returns `None` if `raw` is not within the valid range of `ffi::UA_AttributeId`.
    pub fn try_from_raw(raw: ffi::UA_AttributeId) -> Option<Self> {
        if raw.0 > 27 {
            return None;
        }
        // SAFETY: We have asserted that `raw` is within the valid range of `ffi::UA_AttributeId`
        unsafe {
            Some(std::mem::transmute(raw.0))
        }
    }
}

use open62541::ua;

mod attr_id;

pub use attr_id::*;

pub struct ReferenceDescription<'a>(pub &'a ua::ReferenceDescription);

impl serde::Serialize for ReferenceDescription<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let rd = self.0;
        let mut state = serializer.serialize_struct("ReferenceDescription", 7)?;
        state.serialize_field("nodeId", &rd.node_id().to_string())?;
        state.serialize_field("browseName", &rd.browse_name().to_string())?;
        state.serialize_field("displayName", &rd.display_name().text())?;
        state.serialize_field("nodeClass", &rd.node_class().to_string())?;
        state.serialize_field("typeDefinition", &rd.type_definition().node_id())?;
        state.serialize_field("isForward", &rd.is_forward())?;
        state.serialize_field("referenceTypeId", &rd.reference_type_id())?;
        state.end()
    }
}

pub struct ReferenceDescriptions(pub Vec<ua::ReferenceDescription>);

impl serde::Serialize for ReferenceDescriptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let rds = &self.0;
        let mut seq = serializer.serialize_seq(Some(rds.len()))?;
        for rd in rds {
            seq.serialize_element(&ReferenceDescription(rd))?;
        }
        seq.end()
    }
}

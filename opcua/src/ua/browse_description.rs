impl super::BrowseDescription {
    pub fn with_node_id(mut self, node_id: super::NodeId) -> Self {
        use crate::DataType;

        self.raw.nodeId = node_id.into_raw();
        self
    }
}

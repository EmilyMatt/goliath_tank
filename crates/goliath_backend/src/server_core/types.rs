use goliath_common::core::NodeType;
use goliath_common::ClientConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync as TokioSync;

pub struct RegistrationTypeResponse {
    pub(crate) id: String,
    pub(crate) node_type: NodeType,
    pub(crate) ws_conn: ClientConnection,
}

pub type IdentifiedNode = (String, ClientConnection);

pub type IdentifiedNodeList = Arc<TokioSync::Mutex<HashMap<String, ClientConnection>>>;

//! Typescript Attribute Ids code generation for OPC/UA.
use tokio::io::{self, AsyncWriteExt};

use crate::parse::AttributeId;

pub async fn generate<W>(attr_ids: &[AttributeId], out: &mut W) -> tokio::io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    out.write(b"export const enum AttributeId {\n").await?;
    for attr_id in attr_ids {
        out.write(format!("    {} = {},\n", attr_id.name, attr_id.id).as_bytes())
            .await?;
    }
    out.write(b"}\n").await?;

    out.flush().await
}
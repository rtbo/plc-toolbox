//! OPC/UA Status codes generation for Typescript
use tokio::io::{self, AsyncWriteExt};

use crate::parse::StatusCode;

pub async fn generate<W>(status_codes: &[StatusCode], out: &mut W) -> tokio::io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    out.write(b"export const enum StatusCode {\n").await?;
    for code in status_codes {
        out.write(format!("    /** {} */\n", code.explanation).as_bytes())
            .await?;
        out.write(format!("    {} = 0x{:08x},\n", code.name, code.code).as_bytes())
            .await?;
    }
    out.write(b"}\n").await?;

    let post = include_bytes!("status_codes_post.ts");
    out.write(post).await?;
    
    out.flush().await
}

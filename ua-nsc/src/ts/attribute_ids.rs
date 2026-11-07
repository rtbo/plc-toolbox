//! Typescript Attribute Ids code generation for OPC/UA.
use std::io;

use crate::parse::AttributeId;

pub fn generate<W>(attr_ids: &[AttributeId], out: &mut W) -> std::io::Result<()>
where
    W: io::Write + Unpin,
{
    out.write_all(b"export const enum AttributeId {\n")?;
    for attr_id in attr_ids {
        out.write_all(format!("    {} = {},\n", attr_id.name, attr_id.id).as_bytes())?;
    }
    out.write_all(b"}\n")?;

    out.flush()
}

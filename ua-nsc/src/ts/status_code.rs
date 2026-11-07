//! OPC/UA Status codes generation for Typescript
use std::io;

use crate::parse::StatusCode;

pub fn generate<W>(status_code: &[StatusCode], out: &mut W) -> std::io::Result<()>
where
    W: io::Write + Unpin,
{
    out.write_all(b"export const enum StatusCodeId {\n")?;
    for code in status_code {
        out.write_all(format!("    /** {} */\n", code.explanation).as_bytes())?;
        out.write_all(format!("    {} = 0x{:08x},\n", code.name, code.code).as_bytes())?;
    }
    out.write_all(b"}\n")?;

    out.flush()
}

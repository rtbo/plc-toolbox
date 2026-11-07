//! OPC/UA Status codes generation for Rust
use std::io;

use crate::codegen::CodeGen;
use crate::parse::StatusCode;

pub fn generate<W>(status_code: &[StatusCode], out: &mut W) -> std::io::Result<()>
where
    W: io::Write + Unpin,
{
    let mut cg = super::RsCodeGen::new(out);

    cg.write_line("#[allow(non_camel_case_types)]")?;
    cg.write_line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    cg.write_line("#[repr(u32)]")?;
    cg.start_block("pub enum StatusCode ")?;
    for code in status_code {
        cg.write_line(&format!("/// {}", code.explanation))?;
        cg.write_line(&format!("{} = 0x{:08x},", code.name, code.code))?;
    }
    cg.end_block()?;

    cg.write_line("")?;
    cg.start_block("impl TryFrom<u32> for StatusCode ")?;
    cg.write_line("type Error = ();")?;
    cg.start_block("fn try_from(code: u32) -> Result<Self, Self::Error> ")?;
    cg.start_block("match code ")?;
    for code in status_code {
        cg.write_line(&format!(
            "0x{:08x} => Ok(StatusCode::{}),",
            code.code, code.name
        ))?;
    }
    cg.write_line("_ => Err(()),")?;
    cg.end_block()?; // match
    cg.end_block()?; // fn try_from
    cg.end_block()?; // impl TryFrom<u32> for StatusCode

    cg.write_line("")?;
    cg.start_block("impl StatusCode ")?;
    cg.start_block("pub fn name(&self) -> &'static str ")?;
    cg.start_block("match self ")?;
    for code in status_code {
        cg.write_line(&format!("StatusCode::{} => \"{}\",", code.name, code.name))?;
    }
    cg.end_block()?; // match
    cg.end_block()?; // fn name

    cg.write_line("")?;
    cg.start_block("pub fn explanation(&self) -> &'static str ")?;
    cg.start_block("match self ")?;
    for code in status_code {
        cg.write_line(&format!(
            "StatusCode::{} => \"{}\",",
            code.name, code.explanation
        ))?;
    }
    cg.end_block()?; // match
    cg.end_block()?; // fn explanation
    cg.end_block()?; // impl StatusCode

    cg.flush()
}

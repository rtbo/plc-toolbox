//! OPC/UA Status codes generation for Rust
use tokio::io;

use crate::codegen::CodeGen;
use crate::parse::StatusCode;

pub async fn generate<W>(status_code: &[StatusCode], out: &mut W) -> tokio::io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    let mut cg = super::RsCodeGen::new(out);

    cg.write_line("#[allow(non_camel_case_types)]").await?;
    cg.write_line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .await?;
    cg.write_line("#[repr(u32)]").await?;
    cg.start_block("pub enum StatusCode ").await?;
    for code in status_code {
        cg.write_line(&format!("/// {}", code.explanation)).await?;
        cg.write_line(&format!("{} = 0x{:08x},", code.name, code.code))
            .await?;
    }
    cg.end_block().await?;

    cg.write_line("").await?;
    cg.start_block("impl TryFrom<u32> for StatusCode ").await?;
    cg.write_line("type Error = ();").await?;
    cg.start_block("pub fn try_from(code: u32) -> Result<Self, Self::Error> ")
        .await?;
    cg.start_block("match code ").await?;
    for code in status_code {
        cg.write_line(&format!(
            "0x{:08x} => Ok(StatusCode::{}),",
            code.code, code.name
        ))
        .await?;
    }
    cg.write_line("_ => Err(()),").await?;
    cg.end_block().await?; // match
    cg.end_block().await?; // fn try_from
    cg.end_block().await?; // impl TryFrom<u32> for StatusCode

    cg.write_line("").await?;
    cg.start_block("impl StatusCode ").await?;
    cg.start_block("pub fn name(&self) -> &'static str ")
        .await?;
    cg.start_block("match self ").await?;
    for code in status_code {
        cg.write_line(&format!("StatusCode::{} => \"{}\",", code.name, code.name))
            .await?;
    }
    cg.end_block().await?; // match
    cg.end_block().await?; // fn name

    cg.write_line("").await?;
    cg.start_block("pub fn explanation(&self) -> &'static str ")
        .await?;
    cg.start_block("match self ").await?;
    for code in status_code {
        cg.write_line(&format!(
            "StatusCode::{} => \"{}\",",
            code.name, code.explanation
        ))
        .await?;
    }
    cg.end_block().await?; // match
    cg.end_block().await?; // fn explanation
    cg.end_block().await?; // impl StatusCode

    cg.flush().await
}

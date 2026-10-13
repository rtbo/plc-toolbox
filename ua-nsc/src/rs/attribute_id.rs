//! Rust Attribute Ids code generation for OPC/UA.
use tokio::io;

use crate::codegen::CodeGen;
use crate::parse::AttributeId;

pub async fn generate<W>(attr_ids: &[AttributeId], out: &mut W) -> tokio::io::Result<()>
where
    W: io::AsyncWrite + Unpin,
{
    let mut cg = super::RsCodeGen::new(out);

    cg.write_line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")
        .await?;
    cg.write_line("#[repr(u32)]").await?;
    cg.start_block("pub enum AttributeId ").await?;
    for attr_id in attr_ids {
        cg.write_line(&format!("{} = {},", attr_id.name, attr_id.id))
            .await?;
    }
    cg.end_block().await?; // pub enum AttributeId

    cg.write_line("").await?;
    cg.start_block("impl TryFrom<u32> for AttributeId ").await?;
    cg.write_line("type Error = ();").await?;
    cg.start_block("fn try_from(id: u32) -> Result<Self, Self::Error> ")
        .await?;
    cg.start_block("match id ").await?;
    for attr_id in attr_ids {
        cg.write_line(&format!(
            "{} => Ok(AttributeId::{}),",
            attr_id.id, attr_id.name
        ))
        .await?;
    }
    cg.write_line("_ => Err(()),").await?;
    cg.end_block().await?; // match
    cg.end_block().await?; // fn try_from
    cg.end_block().await?; // impl TryFrom<u32> for AttributeId

    cg.write_line("").await?;
    cg.start_block("impl AttributeId ").await?;
    cg.start_block("pub fn name(&self) -> &'static str ")
        .await?;
    cg.start_block("match self ").await?;
    for attr_id in attr_ids {
        cg.write_line(&format!(
            "AttributeId::{} => \"{}\",",
            attr_id.name, attr_id.name
        ))
        .await?;
    }
    cg.end_block().await?; // match
    cg.end_block().await?; // fn name
    cg.end_block().await?; // impl AttributeId

    cg.flush().await
}

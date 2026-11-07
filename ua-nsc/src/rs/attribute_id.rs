//! Rust Attribute Ids code generation for OPC/UA.
use std::io;

use crate::codegen::CodeGen;
use crate::parse::AttributeId;

pub fn generate<W>(attr_ids: &[AttributeId], out: &mut W) -> std::io::Result<()>
where
    W: io::Write + Unpin,
{
    let mut cg = super::RsCodeGen::new(out);

    cg.write_line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    cg.write_line("#[repr(u32)]")?;
    cg.start_block("pub enum AttributeId ")?;
    for attr_id in attr_ids {
        cg.write_line(&format!("{} = {},", attr_id.name, attr_id.id))?;
    }
    cg.end_block()?; // pub enum AttributeId

    cg.write_line("")?;
    cg.start_block("impl TryFrom<u32> for AttributeId ")?;
    cg.write_line("type Error = ();")?;
    cg.start_block("fn try_from(id: u32) -> Result<Self, Self::Error> ")?;
    cg.start_block("match id ")?;
    for attr_id in attr_ids {
        cg.write_line(&format!(
            "{} => Ok(AttributeId::{}),",
            attr_id.id, attr_id.name
        ))?;
    }
    cg.write_line("_ => Err(()),")?;
    cg.end_block()?; // match
    cg.end_block()?; // fn try_from
    cg.end_block()?; // impl TryFrom<u32> for AttributeId

    cg.write_line("")?;
    cg.start_block("impl AttributeId ")?;
    cg.start_block("pub fn name(&self) -> &'static str ")?;
    cg.start_block("match self ")?;
    for attr_id in attr_ids {
        cg.write_line(&format!(
            "AttributeId::{} => \"{}\",",
            attr_id.name, attr_id.name
        ))?;
    }
    cg.end_block()?; // match
    cg.end_block()?; // fn name
    cg.end_block()?; // impl AttributeId

    cg.flush()
}

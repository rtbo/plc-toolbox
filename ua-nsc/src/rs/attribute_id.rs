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
    cg.define_block("pub enum AttributeId ", |cg| {
        for attr_id in attr_ids {
            cg.write_line(&format!("{} = {},", attr_id.name, attr_id.id))?;
        }
        Ok(())
    })?;

    cg.write_line("")?;
    cg.define_block("impl TryFrom<u32> for AttributeId ", |cg| {
        cg.write_line("type Error = ();")?;
        cg.define_block("fn try_from(id: u32) -> Result<Self, Self::Error> ", |cg| {
            cg.define_block("match id ", |cg| {
                for attr_id in attr_ids {
                    cg.write_line(&format!(
                        "{} => Ok(AttributeId::{}),",
                        attr_id.id, attr_id.name
                    ))?;
                }
                cg.write_line("_ => Err(()),")
            })
        })
    })?;

    cg.write_line("")?;
    cg.define_block("impl AttributeId ", |cg| {
        cg.define_block("pub fn name(&self) -> &'static str ", |cg| {
            cg.define_block("match self ", |cg| {
                for attr_id in attr_ids {
                    cg.write_line(&format!(
                        "AttributeId::{} => \"{}\",",
                        attr_id.name, attr_id.name
                    ))?;
                }
                Ok(())
            })
        })
    })?;

    cg.flush()
}

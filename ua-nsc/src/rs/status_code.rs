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
    cg.define_block("pub enum StatusCode ", |cg| {
        for code in status_code {
            cg.write_line(&format!("/// {}", code.explanation))?;
            cg.write_line(&format!("{} = 0x{:08x},", code.name, code.code))?;
        }
        Ok(())
    })?;

    cg.write_line("")?;
    cg.define_block("impl TryFrom<u32> for StatusCode ", |cg| {
        cg.write_line("type Error = ();")?;
        cg.define_block(
            "fn try_from(code: u32) -> Result<Self, Self::Error> ",
            |cg| {
                cg.define_block("match code ", |cg| {
                    for code in status_code {
                        cg.write_line(&format!(
                            "0x{:08x} => Ok(StatusCode::{}),",
                            code.code, code.name
                        ))?;
                    }
                    cg.write_line("_ => Err(()),")
                })
            },
        )
    })?;

    cg.write_line("")?;
    cg.define_block("impl StatusCode ", |cg| {
        cg.define_block("pub fn name(&self) -> &'static str ", |cg| {
            cg.define_block("match self ", |cg| {
                for code in status_code {
                    cg.write_line(&format!("StatusCode::{} => \"{}\",", code.name, code.name))?;
                }
                Ok(())
            })
        })?;

        cg.write_line("")?;
        cg.define_block("pub fn explanation(&self) -> &'static str ", |cg| {
            cg.define_block("match self ", |cg| {
                for code in status_code {
                    cg.write_line(&format!(
                        "StatusCode::{} => \"{}\",",
                        code.name, code.explanation
                    ))?;
                }
                Ok(())
            })
        })
    })?;

    cg.flush()
}

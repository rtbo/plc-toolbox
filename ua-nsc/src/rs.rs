use std::io;

use crate::codegen::CodeGen;

pub mod attribute_id;
pub mod status_code;
pub mod types;

struct RsCodeGen<W>
where
    W: io::Write + Unpin,
{
    output: W,
    indent: u32,
}

impl<W> CodeGen<W> for RsCodeGen<W>
where
    W: io::Write + Unpin,
{
    fn output_mut(&mut self) -> &mut W {
        &mut self.output
    }
}

impl<W> RsCodeGen<W>
where
    W: io::Write + Unpin,
{
    fn new(output: W) -> Self {
        RsCodeGen { output, indent: 0 }
    }

    fn write_line(&mut self, txt: &str) -> io::Result<()> {
        self.write_indentedline(self.indent, txt)
    }

    fn start_block(&mut self, prefix: &str) -> io::Result<()> {
        self.write_indentedline(self.indent, &format!("{}{{", prefix))?;
        self.indent += 1;
        Ok(())
    }

    fn end_block(&mut self) -> io::Result<()> {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.write_indentedline(self.indent, "}")?;
        Ok(())
    }

    fn _write_comment(&mut self, doc: &str) -> io::Result<()> {
        for line in doc.lines() {
            self.write_indentedline(self.indent, &format!("// {}", line))?;
        }
        Ok(())
    }

    fn write_doc_comment(&mut self, doc: &str) -> io::Result<()> {
        for line in doc.lines() {
            self.write_indentedline(self.indent, &format!("/// {}", line))?;
        }
        Ok(())
    }

    fn write_module_doc_comment(&mut self, doc: &str) -> io::Result<()> {
        for line in doc.lines() {
            self.write_indentedline(self.indent, &format!("//! {}", line))?;
        }
        Ok(())
    }
}

pub use std::io;

pub mod attribute_ids;
pub mod node_ids;
pub mod status_code;
pub mod types;

use crate::codegen::CodeGen;

struct TsCodeGen<W>
where
    W: io::Write + Unpin,
{
    output: W,
    indent: u32,
}

impl<W> CodeGen<W> for TsCodeGen<W>
where
    W: io::Write + Unpin,
{
    fn output_mut(&mut self) -> &mut W {
        &mut self.output
    }
}

impl<W> TsCodeGen<W>
where
    W: io::Write + Unpin,
{
    fn new(output: W) -> Self {
        TsCodeGen { output, indent: 0 }
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
}

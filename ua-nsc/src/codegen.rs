use std::io::{self, Write};

pub trait CodeGen<W>
where
    W: Write + Unpin,
{
    fn output_mut(&mut self) -> &mut W;

    fn write_indentedline(&mut self, indent: u32, txt: &str) -> io::Result<()> {
        let out = self.output_mut();
        for _ in 0..indent {
            out.write_all(b"    ")?;
        }
        out.write_all(txt.as_bytes())?;
        out.write_all(b"\n")?;
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output_mut().flush()
    }
}

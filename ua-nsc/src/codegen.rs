use tokio::io::{self, AsyncWriteExt};

pub trait CodeGen<W>
where
    W: AsyncWriteExt + Unpin,
{
    fn output_mut(&mut self) -> &mut W;

    async fn write_indentedline(&mut self, indent: u32, txt: &str) -> io::Result<()> {
        let out = self.output_mut();
        for _ in 0..indent {
            out.write_all(b"    ").await?;
        }
        out.write_all(txt.as_bytes()).await?;
        out.write_all(b"\n").await?;
        Ok(())
    }
}

use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::mem::size_of;

pub async fn read<R>(reader: &mut R) -> std::io::Result<Vec<u8>>
where
    R: AsyncRead + Unpin,
{
    // println!("waiting to read!");
    let mut len_buf = [0u8; size_of::<usize>()];
    reader.read_exact(&mut len_buf).await?;
    println!("got length of payload");
    let len = usize::from_le_bytes(len_buf);

    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    println!("got payload");
    Ok(buf)
}

pub async fn write<W>(writer: &mut W, data: &[u8]) -> std::io::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let len_bytes = (data.len() as usize).to_le_bytes();
    writer.write_all(&len_bytes).await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

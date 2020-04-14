#![feature(async_closure)]
#![feature(type_ascription)]

use primal::Primes;
use tokio::stream::Stream;

pub fn primes_unbounded() -> impl Stream<Item = usize> {
    utils::spawn_stream_from_iterator(Primes::all())
}

mod utils {
    use tokio::stream::Stream;
    use tokio::sync::mpsc;
    use tokio::task::spawn_blocking;

    pub fn spawn_stream_from_iterator<T: Send + 'static>(
        it: impl Iterator<Item = T> + Send + 'static,
    ) -> impl Stream<Item = T> + 'static {
        #[allow(unused_mut)]
        let (mut s, r) = mpsc::channel(1);
        spawn_blocking(async move || -> Result<(), mpsc::error::SendError<T>> {
            for x in it {
                s.send(x).await?;
            }
            Ok(())
        });
        r
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::{self, stdout, AsyncWriteExt};
    use tokio::join;
    use tokio::task::{spawn, yield_now};
    use tokio::time::{delay_for, Duration};

    #[tokio::test]
    async fn it_works() -> io::Result<()> {
        let handle1 = spawn(async {
            stdout().write(b"1\n").await?;
            delay_for(Duration::from_secs(10)).await;
            stdout().write(b"2\n").await?;
            Ok(()): io::Result<()>
        });
        let handle2 = spawn(async {
            delay_for(Duration::from_secs(5)).await;
            stdout().write(b"I\n").await?;
            delay_for(Duration::from_secs(5)).await;
            stdout().write(b"II\n").await?;
            Ok(()): io::Result<()>
        });
        stdout().write(b"A\n").await?;
        delay_for(Duration::from_secs(5));
        stdout().write(b"B\n").await?;
        join!(handle1, handle2);
        stdout().write(b"C\n").await?;
        //drop(handle);
        stdout().write(b"D\n").await?;
        Ok(())
    }
}

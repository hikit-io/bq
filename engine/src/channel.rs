use tokio::sync::{broadcast, mpsc, oneshot};

pub struct Mpsc<Data> {
    pub(crate) tx: mpsc::UnboundedSender<Data>,
    pub(crate) rx: Option<mpsc::UnboundedReceiver<Data>>,
}

impl<Data> Default for Mpsc<Data> {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx: tx,
            rx: Some(rx),
        }
    }
}

#[derive(Clone)]
pub struct Broadcast<Data>
where
    Data: Clone,
{
    pub(crate) tx: broadcast::Sender<Data>,
}

impl<Data> Default for Broadcast<Data>
where
    Data: Clone,
{
    fn default() -> Self {
        let (tx, _rx) = broadcast::channel(1024);
        Self { tx }
    }
}

pub struct Oneshot<Data> {
    pub tx: Option<oneshot::Sender<Data>>,
    pub rx: Option<oneshot::Receiver<Data>>,
}

impl<Data> Default for Oneshot<Data> {
    fn default() -> Self {
        let (tx, rx) = oneshot::channel();
        Self { tx:Some(tx), rx:Some(rx) }
    }
}

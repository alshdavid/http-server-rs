use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
pub struct BroadcastChannel<T: Clone + Send + 'static> {
  tx: UnboundedSender<T>,
  rrx: Subscribable<T>,
}

impl<T: Clone + Send + 'static> BroadcastChannel<T> {
  pub fn new() -> Self {
    let (tx, rrx) = channel_broadcast();
    Self { tx, rrx }
  }

  pub fn send(
    &self,
    value: T,
  ) -> Result<(), SendError<T>> {
    self.tx.send(value)
  }

  pub fn subscribe(&self) -> UnboundedReceiver<T> {
    self.rrx.subscribe()
  }
}

#[derive(Clone)]
pub struct Subscribable<T: Clone + Send> {
  tx_subscribe: UnboundedSender<UnboundedSender<T>>,
}

impl<T: Clone + Send> Subscribable<T> {
  pub fn subscribe(&self) -> UnboundedReceiver<T> {
    let (tx_subscription, rx_subscription) = unbounded_channel::<T>();
    self.tx_subscribe.send(tx_subscription).unwrap();
    rx_subscription
  }
}

pub fn channel_broadcast<T: Clone + Send + 'static>() -> (UnboundedSender<T>, Subscribable<T>) {
  let (tx_t, mut rx_t) = unbounded_channel::<T>();
  let (tx_subscribe, mut rx_subscribe) = unbounded_channel::<UnboundedSender<T>>();

  tokio::task::spawn(async move {
    let mut senders = Vec::<Option<UnboundedSender<T>>>::new();
    while let Some(data) = rx_t.recv().await {
      for sender_opt in senders.iter_mut() {
        let Some(sender) = sender_opt else {
          continue;
        };
        if sender.send(data.clone()).is_err() {
          sender_opt.take();
        }
      }
      while let Ok(sender) = rx_subscribe.try_recv() {
        if sender.send(data.clone()).is_ok() {
          senders.push(Some(sender));
        }
      }
    }
  });

  (tx_t, Subscribable { tx_subscribe })
}

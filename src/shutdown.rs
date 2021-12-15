use tokio::sync::broadcast;

#[derive(Debug)]
pub struct Shutdown {
    shutdown: bool,
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown { shutdown: false, notify }
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    // receive the shutdown notice, wait if necessary
    pub async fn recv(&mut self) {
        if self.shutdown { return; }

        // if an error is received because all senders are gone, then shutdown anyway
        let _ = self.notify.recv().await;

        self.shutdown = true;
    }

}
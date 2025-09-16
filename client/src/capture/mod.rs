pub mod jpeg;
pub mod mic;
pub mod screen;
/*/
#[derive(Clone)]
pub struct CaptureState {
    active: Arc<Mutex<HashSet<CaptureType>>>,
}
impl CaptureState {
    pub fn new() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    pub async fn capturing(&self, capture_type: CaptureType) -> bool {
        let lock = self.active.lock().await;
        lock.contains(&capture_type)
    }

    pub async fn start(&self, capture_type: CaptureType) {
        let mut lock = self.active.lock().await;
        lock.insert(capture_type);
    }
    pub async fn end(&self, capture_type: CaptureType) {
        let mut lock = self.active.lock().await;
        lock.remove(&capture_type);
    }
}
 */

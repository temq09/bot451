use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use async_trait::async_trait;

use crate::bot_error::BotError;
use crate::worker::page_loader::PageLoader;

pub(crate) struct ThrottlePageLoader {
    timeout: Duration,
    worker: Box<dyn PageLoader>,
    shared: Arc<Shared>,
}

struct Shared {
    state: Mutex<State>,
    purge_timeout: Duration,
}

struct State {
    request_time: HashMap<String, u64>,
    shutdown: bool,
}

impl Shared {
    fn is_shutdown(&self) -> bool {
        self.state.lock().unwrap().shutdown
    }

    fn clear(&self) {
        self.state.lock().unwrap().request_time.clear();
    }

    fn shutdown_clear_task(&self) {
        self.state.lock().unwrap().shutdown = true
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            request_time: HashMap::new(),
            shutdown: false,
        }
    }
}

impl Drop for ThrottlePageLoader {
    fn drop(&mut self) {
        self.shared.shutdown_clear_task()
    }
}

impl ThrottlePageLoader {
    pub(crate) fn new(timeout: Duration, worker: Box<dyn PageLoader>) -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            purge_timeout: Duration::from_secs(60),
        });

        tokio::spawn(purge_time(shared.clone()));

        ThrottlePageLoader {
            timeout,
            worker,
            shared,
        }
    }
}

#[async_trait]
impl PageLoader for ThrottlePageLoader {
    async fn load_page(&self, url: String, chat_id: String) -> Result<(), BotError> {
        if can_request(
            &self.shared.state,
            &chat_id,
            self.timeout,
            current_time_sec(),
        ) {
            self.worker.load_page(url, chat_id).await
        } else {
            println!("Throttle request for {}", chat_id);
            Err(BotError::ThrottleError)
        }
    }
}

fn current_time_sec() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn can_request(state: &Mutex<State>, chat_id: &str, timeout: Duration, current_time: u64) -> bool {
    let mut state_locked = state.lock().unwrap();

    let can_request = if !state_locked.shutdown {
        let last_request_time = state_locked.request_time.get(chat_id).unwrap_or(&0);
        current_time - *last_request_time > timeout.as_secs()
    } else {
        false
    };

    if can_request {
        let _ = state_locked
            .request_time
            .insert(chat_id.to_string(), current_time);
    };

    can_request
}

async fn purge_time(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
        shared.clear();
        tokio::time::sleep(shared.purge_timeout).await;
    }
    println!("The clear state task shut down")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;

    use crate::bot_error::BotError;
    use crate::worker::page_loader::PageLoader;
    use crate::worker::throttled_page_loader::{can_request, Shared, State, ThrottlePageLoader};

    #[test]
    fn test_can_request() {
        let state = Mutex::new(State::default());
        let chat_id = "chat_1";
        let timeout = Duration::from_secs(10);

        assert!(
            can_request(&state, chat_id, timeout, 100),
            "initial state should allow to request"
        );
        assert!(
            !can_request(&state, chat_id, timeout, 101),
            "can request should be false when timeout not exceed"
        );
        assert!(
            can_request(&state, chat_id, timeout, 111),
            "can request should be true when timeout exceed"
        );
        assert!(
            !can_request(&state, chat_id, timeout, 115),
            "can request should be false when timeout not exceed"
        );

        assert!(
            can_request(&state, "chat_2", timeout, 115),
            "new user should be able to request"
        );

        state.lock().unwrap().shutdown = true;
        assert!(
            !can_request(&state, "chat_3", timeout, 115),
            "can request should be false once  shutdown"
        );
    }

    #[test]
    fn test_default_state() {
        let state = State::default();

        assert!(!state.shutdown);
        assert_eq!(state.request_time.len(), 0);
    }

    #[tokio::test]
    async fn test_throttled_page_loader() -> Result<(), BotError> {
        let requests = Arc::new(Mutex::new(HashMap::new()));
        let test_loader = Box::new(TestPageLoader {
            load_page_requests: requests.clone(),
        });
        let shared = Arc::new(Shared {
            state: Mutex::new(State::default()),
            purge_timeout: Duration::from_secs(60),
        });
        let throttled_loader = ThrottlePageLoader {
            timeout: Duration::from_secs(10),
            worker: test_loader,
            shared,
        };

        throttled_loader
            .load_page("url_1".to_string(), "chat_1".to_string())
            .await?;

        assert_eq!(
            requests.lock().unwrap().get("url_1"),
            Some(&"chat_1".to_string())
        );

        Ok(())
    }

    struct TestPageLoader {
        load_page_requests: Arc<Mutex<HashMap<String, String>>>,
    }

    #[async_trait]
    impl PageLoader for TestPageLoader {
        async fn load_page(&self, url: String, chat_id: String) -> Result<(), BotError> {
            self.load_page_requests.lock().unwrap().insert(url, chat_id);
            Ok(())
        }
    }
}

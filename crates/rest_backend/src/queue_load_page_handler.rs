use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use api::{PageData, PagePersistent, PageResult, PageUploader, PageWorker};

use crate::load_page_handler::{clear_data, save_to_cache};

type ChatQueue = Arc<Mutex<HashMap<String, VecDeque<String>>>>;

pub struct QueuePageHandler {
    page_loader: Box<dyn PageWorker>,
    page_uploader: Box<dyn PageUploader>,
    cache: Arc<dyn PagePersistent>,
    queue: ChatQueue,
}

impl QueuePageHandler {
    pub(crate) fn new(
        loader: Box<dyn PageWorker>,
        page_uploader: Box<dyn PageUploader>,
        cache: Arc<dyn PagePersistent + 'static>,
    ) -> Self {
        QueuePageHandler {
            page_loader: loader,
            page_uploader,
            cache,
            queue: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn load_page_for_user(
        &self,
        page_url: String,
        chat_id: String,
    ) -> anyhow::Result<()> {
        println!("Load page for {}, user id: {}", page_url, chat_id);
        let already_in_progress = add_to_queue(&page_url, &chat_id, self.queue.clone());

        println!(
            "Loading for page {} already in progress: {}",
            page_url, already_in_progress
        );
        if already_in_progress {
            return Ok(());
        }

        let result = self
            .page_loader
            .submit_page_generation(PageData::from_url(page_url.clone()))
            .await?;

        let file_id = self.page_uploader.send_page(&chat_id, &result).await?;
        if let Some(file_id) = &file_id {
            println!("Saving file id {} to cache", file_id);
            save_to_cache(file_id, &result, &self.cache, page_url.clone()).await;
        }

        let chat_id_queue = get_chat_ids(&page_url, self.queue.clone());

        let tg_result = prepare_result(file_id, &result);
        send_result(tg_result, &chat_id, chat_id_queue, &self.page_uploader).await;

        clear_data(result).await;
        Ok(())
    }
}

fn add_to_queue(page_url: &str, chat_id: &str, queue: ChatQueue) -> bool {
    // todo if an error happens what should be done?
    let mut queue_lock = queue.lock().unwrap();

    let entry = queue_lock.entry(page_url.to_string());
    match entry {
        Entry::Occupied(mut queue) => {
            queue.get_mut().push_back(chat_id.to_string());
            true
        }
        Entry::Vacant(queue) => {
            let mut deque: VecDeque<String> = VecDeque::new();
            deque.push_back(chat_id.to_string());
            queue.insert(deque);
            false
        }
    }
}

fn get_chat_ids(page_url: &str, queue: ChatQueue) -> VecDeque<String> {
    queue
        .lock()
        .unwrap()
        .remove(page_url)
        .unwrap_or_else(|| VecDeque::new())
}

fn prepare_result(file_id: Option<String>, result: &PageResult) -> PageResult {
    return file_id.map_or_else(|| result.clone(), |id| PageResult::TelegramId(id));
}

async fn send_result(
    tg_result: PageResult,
    chat_id: &str,
    chat_id_queue: VecDeque<String>,
    page_uploader: &Box<dyn PageUploader>,
) {
    println!(
        "Sending result to the rest of the queue, size: {}",
        chat_id_queue.len()
    );
    for chat_id_from_queue in chat_id_queue {
        if chat_id_from_queue != chat_id {
            let _ = page_uploader
                .send_page(&chat_id_from_queue, &tg_result)
                .await;
        }
    }
}

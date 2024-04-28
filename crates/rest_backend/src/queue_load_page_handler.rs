use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use api::{PageData, PagePersistent, PageResult, PageUploader, PageWorker};

use crate::load_page_handler::{clear_data, save_to_cache};

pub struct QueuePageHandler {
    page_loader: Box<dyn PageWorker>,
    page_uploader: Box<dyn PageUploader>,
    cache: Arc<dyn PagePersistent>,
    queue: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
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
        // todo if an error happens what should be done?
        let mut queue_lock = self.queue.lock().unwrap();

        let entry = queue_lock.entry(page_url.clone());
        let already_in_progress = match entry {
            Entry::Occupied(mut queue) => {
                queue.get_mut().push_back(chat_id.clone());
                true
            }
            Entry::Vacant(queue) => {
                let mut deque: VecDeque<String> = VecDeque::new();
                deque.push_back(chat_id.clone());
                queue.insert(deque);
                false
            }
        };
        drop(queue_lock);

        if already_in_progress {
            println!("Loading for page {} already in progress", page_url);
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

        let mut queue_lock = self.queue.lock().unwrap();
        let chat_id_queue = queue_lock.remove(&page_url).unwrap_or(VecDeque::new());
        drop(queue_lock);

        let tg_result =
            file_id.map_or_else({ || result.clone() }, { |id| PageResult::TelegramId(id) });
        for chat_id_from_queue in chat_id_queue {
            if chat_id_from_queue != chat_id {
                println!("Sending page {} to {}", page_url, chat_id_from_queue);
                let _ = self
                    .page_uploader
                    .send_page(&chat_id_from_queue, &tg_result)
                    .await;
            }
        }

        clear_data(result).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_text() {
        assert_eq!(1, 1)
    }
}

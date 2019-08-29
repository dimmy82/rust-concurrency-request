use failure::Fail;
use futures::{future, Future};
use reqwest::r#async::Client as AsyncClient;
use reqwest::{Client, Url};
use std::thread;
use std::thread::ThreadId;
use std::time::{Duration, SystemTime};
use tokio_threadpool::{Builder, SpawnHandle};

pub type ResultWithError<T> = std::result::Result<T, ErrorWrapper>;

#[derive(Fail, Debug)]
pub enum ErrorWrapper {
    #[fail(display = "http request error: {:?}", error)]
    HttpRequestError { error: reqwest::Error },
}

impl From<reqwest::Error> for ErrorWrapper {
    fn from(error: reqwest::Error) -> Self {
        ErrorWrapper::HttpRequestError { error }
    }
}

fn main() {
    if true {
        request_by_multi_thread()
    } else {
        request_by_main_thread()
    }
}

fn request_by_main_thread() {
    output1("START SINGLE THREAD");
    let start_at = SystemTime::now();

    let client = Client::new();

    (0..1000)
        .collect::<Vec<i32>>()
        .iter()
        .for_each(|_| match send_request(&client) {
            Ok(text) => output1(text.as_str()),
            Err(error) => output1(format!("{:?}", error).as_str()),
        });

    let spent_time = start_at.elapsed().unwrap_or(Duration::new(0, 0));
    output1(format!("END: {}", spent_time.as_millis()).as_str())
}

fn request_by_multi_thread() {
    output1("START MULTI THREAD");
    let start_at = SystemTime::now();

    let client = AsyncClient::new();
    let thread_pool_size = 10;
    let thread_pool = Builder::new().pool_size(thread_pool_size).build();

    let mut handles = Vec::<SpawnHandle<(ThreadId, String), ErrorWrapper>>::new();
    while handles.iter().count() <= 1000 {
        let cloned_client = client.clone();
        handles.push(
            thread_pool.spawn_handle(future::lazy(move || send_request_async(&cloned_client))),
        );
    }

    handles.iter_mut().for_each(|handle| match handle.wait() {
        Ok((thread_id, text)) => output2(thread_id, text.as_str()),
        Err(error) => output1(format!("{:?}", error).as_str()),
    });

    thread_pool.shutdown_now();

    let spent_time = start_at.elapsed().unwrap_or(Duration::new(0, 0));
    output1(format!("END: {}", spent_time.as_millis()).as_str())
}

fn send_request(client: &Client) -> ResultWithError<String> {
    let mut response = client
        .get(Url::parse("http://localhost:9000/timestamp").unwrap())
        .send()?;
    Ok(response.text()?)
}

fn send_request_async(
    client: &AsyncClient,
) -> impl Future<Item = (ThreadId, String), Error = ErrorWrapper> {
    client
        .get(Url::parse("http://localhost:9000/timestamp").unwrap())
        .send()
        .and_then(|mut response| response.text())
        .map(|text| (thread::current().id(), text))
        .from_err()
}

fn output2(thread_id: ThreadId, text: &str) {
    println!("[{:?}] => {}", thread_id, text);
}

fn output1(text: &str) {
    output2(thread::current().id(), text);
}

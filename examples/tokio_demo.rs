use std::sync::{Mutex, Arc};
use std::io;
use std::io::Write;
use tokio::task::JoinHandle;
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let tasks: Arc<Mutex<HashMap<u64, JoinHandle<()>>>> = Arc::new(Mutex::new(HashMap::new()));
    loop {
        print!("input: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        let req = input.trim();

        match req {
            "q" => break,
            "pop" => {
            }
            "l" => {
                println!("{:#?}", tasks);
            },
            _ => {
                let num: u64 = req.parse().unwrap();
                let choice = {
                    let tasks_lock = tasks.lock().unwrap();
                    tasks_lock.contains_key(&num)
                };
                if choice {
                    {
                        let mut tasks_lock = tasks.lock().unwrap();
                        let handle = tasks_lock.remove(&num).unwrap();
                        handle.abort();
                    }
                } else {
                    let tasks_clone = tasks.clone();
                    let handle = tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(num)).await;
                        println!("output {}", num);
                        {
                            let mut tasks_lock = tasks_clone.lock().unwrap();
                            tasks_lock.remove(&num);
                        }
                    });
                    {
                        let mut tasks_lock = tasks.lock().unwrap();
                        tasks_lock.insert(num, handle);
                    }
                }
            }
        }
    }
}

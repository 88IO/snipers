#[tokio::main]
async fn main() {
    let mut v = Vec::new();

    println!("push futures");
    for i in 0..1_000 {
        v.push(tokio::spawn(async move {
            a(i).await
        }));
    }

    println!("start wait");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    println!("end wait");

    for vi in v {
        println!("{}", vi.await.unwrap());
    }
}

async fn a(n: usize) -> String {
    println!("start {n}");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    format!("end {n}")
}

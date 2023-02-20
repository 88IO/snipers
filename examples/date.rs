use chrono::FixedOffset;
use chrono::Utc;
use chrono::DateTime;

fn main() {
    let o = FixedOffset::east_opt(3600 * 9).unwrap();
    let n = Utc::now().date_naive().and_hms_opt(10, 10, 0).unwrap();
    let t = DateTime::<FixedOffset>::from_local(n, o);

    let a = Utc::today()
        .with_timezone(&o)
        .and_hms_opt(10, 10, 0).unwrap();

    println!("{}", a);
    println!("{}", t);
}

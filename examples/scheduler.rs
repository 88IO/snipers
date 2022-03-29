use std::collections::{BTreeMap, HashMap, HashSet};
use serenity::model::guild::Member;
use serenity::model::id::GuildId;
use std::hash::{Hash, Hasher};
use chrono::{Utc, DateTime, FixedOffset};

#[derive(Debug, Hash, PartialEq)]
pub enum EventType {
    Disconnect,
    Notification3Min,
}

pub struct Job {
    member: Member,
    event_type: EventType,
    timezone: FixedOffset
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.member.user.hash(state);
        self.member.guild_id.hash(state);
        self.event_type.hash(state);
    }
}
impl PartialEq for Job {
    fn eq(&self, other: &Self) -> bool {
        self.member.user == other.member.user
            && self.member.guild_id == other.member.guild_id
            && self.event_type == other.event_type
    }
}
impl Eq for Job {}

impl Job {
    fn new(member: Member, event_type: EventType, timezone: FixedOffset) -> Self {
        Job { member, event_type, timezone }
    }
}

struct Setting {
    timezone: FixedOffset
}

struct Scheduler {
    jobs: BTreeMap<DateTime<FixedOffset>, HashSet<Job>>,
    settings: HashMap<GuildId, Setting>
}

impl Scheduler {
    fn new() -> Self {
        Scheduler {
            jobs: BTreeMap::new(),
            settings: HashMap::new()
        }
    }
    fn add(&mut self, datetime: DateTime<FixedOffset>, member: Member, event_type: EventType) -> bool {
        let new_job = Job::new(member, event_type, datetime.timezone());
        self.jobs.entry(datetime).or_insert(HashSet::new()).insert(new_job)
    }
    fn run_pending(&mut self) {
    }
}

fn main() {
    let mut btree = BTreeMap::new();

    let offset = FixedOffset::east(3600 * 9);

    let a = Utc::now().with_timezone(&offset);
    let b = Utc::now().with_timezone(&offset);
    let bb = b.clone();
    let c = Utc::now().with_timezone(&FixedOffset::east(3600));

    println!("old bb timezone: {}", bb);
    let bb = bb.with_timezone(&FixedOffset::east(0));
    println!("new bb timezone: {}", bb);
    println!("diff: {}", bb == b);

    btree.insert(b, "b".to_string());
    btree.insert(a, "a".to_string());
    btree.insert(c, "c".to_string());

    println!("{:?}", btree);

    btree.entry(bb).or_insert("bb".to_string()).push_str("bb");

    let d = Utc::now().with_timezone(&FixedOffset::east(3600));
    btree.entry(d).or_insert(String::new()).push_str("d");

    println!("{:?}", btree);
}

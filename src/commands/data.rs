use std::{
    collections::{HashMap, HashSet},
    sync::Arc
};
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;
use chrono::{DateTime, FixedOffset};
use crate::job::Job;

pub struct TimeZone;

impl TypeMapKey for TimeZone {
    type Value = Arc<RwLock<FixedOffset>>;
}

pub struct JobStore;

impl TypeMapKey for JobStore {
    type Value = Arc<RwLock<HashMap<DateTime<FixedOffset>, HashSet<Job>>>>;
}


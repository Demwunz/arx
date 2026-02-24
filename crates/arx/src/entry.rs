use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Clarification,
    Decision,
    Override,
    Blocker,
    Assumption,
    Risk,
    Defer,
    Tombstone,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EntryType::Clarification => "clarification",
            EntryType::Decision => "decision",
            EntryType::Override => "override",
            EntryType::Blocker => "blocker",
            EntryType::Assumption => "assumption",
            EntryType::Risk => "risk",
            EntryType::Defer => "defer",
            EntryType::Tombstone => "tombstone",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EntryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clarification" => Ok(EntryType::Clarification),
            "decision" => Ok(EntryType::Decision),
            "override" => Ok(EntryType::Override),
            "blocker" => Ok(EntryType::Blocker),
            "assumption" => Ok(EntryType::Assumption),
            "risk" => Ok(EntryType::Risk),
            "defer" => Ok(EntryType::Defer),
            "tombstone" => Ok(EntryType::Tombstone),
            _ => Err(format!("invalid entry type: {s}")),
        }
    }
}

impl EntryType {
    pub fn all() -> &'static [&'static str] {
        &[
            "clarification",
            "decision",
            "override",
            "blocker",
            "assumption",
            "risk",
            "defer",
            "tombstone",
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    Human,
    Planner,
    Executor,
    Reviewer,
    System,
}

impl fmt::Display for ActorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ActorType::Human => "human",
            ActorType::Planner => "planner",
            ActorType::Executor => "executor",
            ActorType::Reviewer => "reviewer",
            ActorType::System => "system",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for ActorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "human" => Ok(ActorType::Human),
            "planner" => Ok(ActorType::Planner),
            "executor" => Ok(ActorType::Executor),
            "reviewer" => Ok(ActorType::Reviewer),
            "system" => Ok(ActorType::System),
            _ => Err(format!("invalid actor type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryState {
    Active,
    Superseded,
    Reversed,
}

impl fmt::Display for EntryState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EntryState::Active => "active",
            EntryState::Superseded => "superseded",
            EntryState::Reversed => "reversed",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for EntryState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(EntryState::Active),
            "superseded" => Ok(EntryState::Superseded),
            "reversed" => Ok(EntryState::Reversed),
            _ => Err(format!("invalid entry state: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub actor: ActorType,
    pub date: DateTime<Utc>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip, default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(rename = "reversed_by", skip_serializing_if = "Option::is_none")]
    pub reversed_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntryWithState {
    pub entry: Entry,
    pub state: EntryState,
}

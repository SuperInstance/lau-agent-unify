//! # lau-agent-unify
//!
//! A single type system that treats room ensigns and git-native agents as the same thing.
//! An agent is an agent whether it lives in a room or a repo.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// AgentType
// ---------------------------------------------------------------------------

/// What kind of agent this is.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    /// Lives in a PLATO room, fast, focused.
    RoomEnsign,
    /// Lives in a git repo, state in refs, memory in orphan branches.
    GitNative,
    /// Child shell, sandboxed.
    ZeroClaw,
    /// GPU-accelerated child shell.
    CUDAClaw,
    /// The captain / club manager.
    Hermes,
    /// Anything else.
    Custom(String),
}

impl AgentType {
    pub fn is_sandboxed(&self) -> bool {
        matches!(self, Self::ZeroClaw)
    }

    pub fn needs_room(&self) -> bool {
        matches!(self, Self::RoomEnsign)
    }

    pub fn label(&self) -> String {
        match self {
            Self::RoomEnsign => "RoomEnsign".into(),
            Self::GitNative => "GitNative".into(),
            Self::ZeroClaw => "ZeroClaw".into(),
            Self::CUDAClaw => "CUDAClaw".into(),
            Self::Hermes => "Hermes".into(),
            Self::Custom(s) => s.clone(),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentLocation
// ---------------------------------------------------------------------------

/// Where an agent lives.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentLocation {
    Room { room_id: String, shell_id: String },
    Repo { repo_path: String, branch: Option<String> },
    Shell { shell_id: String, universe_path: String },
    Remote { address: String, protocol: String },
}

impl AgentLocation {
    pub fn describe(&self) -> String {
        match self {
            Self::Room { room_id, shell_id } => {
                format!("room {room_id} (shell {shell_id})")
            }
            Self::Repo { repo_path, branch } => match branch {
                Some(b) => format!("repo {repo_path} on {b}"),
                None => format!("repo {repo_path}"),
            },
            Self::Shell { shell_id, universe_path } => {
                format!("shell {shell_id} at {universe_path}")
            }
            Self::Remote { address, protocol } => {
                format!("{protocol}://{address}")
            }
        }
    }

    pub fn is_local(&self) -> bool {
        !matches!(self, Self::Remote { .. })
    }
}

// ---------------------------------------------------------------------------
// AgentCapability
// ---------------------------------------------------------------------------

/// What an agent can do.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    Chat,
    Code,
    Vision,
    Audio,
    Math,
    Creative,
    Hardware { devices: Vec<String> },
    Compute { gpu: bool },
    Network { protocols: Vec<String> },
    FileSystem { paths: Vec<String> },
    Git,
    Provenance,
    Delegate,
    Escalate,
}

impl AgentCapability {
    pub fn label(&self) -> String {
        match self {
            Self::Chat => "Chat".into(),
            Self::Code => "Code".into(),
            Self::Vision => "Vision".into(),
            Self::Audio => "Audio".into(),
            Self::Math => "Math".into(),
            Self::Creative => "Creative".into(),
            Self::Hardware { devices } => format!("Hardware({})", devices.join(",")),
            Self::Compute { gpu: true } => "Compute(GPU)".into(),
            Self::Compute { gpu: false } => "Compute(CPU)".into(),
            Self::Network { protocols } => format!("Network({})", protocols.join(",")),
            Self::FileSystem { paths } => format!("FileSystem({})", paths.join(",")),
            Self::Git => "Git".into(),
            Self::Provenance => "Provenance".into(),
            Self::Delegate => "Delegate".into(),
            Self::Escalate => "Escalate".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// AgentStatus
// ---------------------------------------------------------------------------

/// The agent's current state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Dormant,
    Waking,
    Orienting,
    Ready,
    Working { task: String, progress: f64 },
    YellowAlert,
    RedAlert { reason: String },
    StandingDown { report: String },
    Error(String),
    Offline,
}

impl AgentStatus {
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Waking | Self::Orienting | Self::Ready | Self::Working { .. }
        )
    }

    pub fn needs_attention(&self) -> bool {
        matches!(
            self,
            Self::YellowAlert | Self::RedAlert { .. } | Self::Error(_)
        )
    }
}

// ---------------------------------------------------------------------------
// UnifiedAgent
// ---------------------------------------------------------------------------

/// An agent, regardless of where it lives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedAgent {
    pub id: String,
    pub name: String,
    pub agent_type: AgentType,
    pub location: AgentLocation,
    pub status: AgentStatus,
    pub capabilities: Vec<AgentCapability>,
    pub model: Option<String>,
    pub conservation_budget: f64,
    pub conservation_used: f64,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub connections: Vec<String>,
}

impl UnifiedAgent {
    pub fn new(name: &str, agent_type: AgentType, location: AgentLocation) -> Self {
        let id = format!(
            "{}-{}",
            agent_type.label().to_lowercase(),
            name.to_lowercase().replace(' ', "-")
        );
        Self {
            id,
            name: name.into(),
            agent_type,
            location,
            status: AgentStatus::Dormant,
            capabilities: Vec::new(),
            model: None,
            conservation_budget: 0.0,
            conservation_used: 0.0,
            parent: None,
            children: Vec::new(),
            connections: Vec::new(),
        }
    }

    pub fn add_capability(&mut self, cap: AgentCapability) {
        if !self.capabilities.contains(&cap) {
            self.capabilities.push(cap);
        }
    }

    pub fn grant_connection(&mut self, target: &str) {
        if !self.connections.contains(&target.to_string()) {
            self.connections.push(target.into());
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status, AgentStatus::Ready)
    }

    pub fn can_afford(&self, cost: f64) -> bool {
        self.conservation_budget - self.conservation_used >= cost
    }

    pub fn spend(&mut self, cost: f64) -> Result<(), String> {
        if !self.can_afford(cost) {
            return Err(format!(
                "insufficient budget: need {cost}, have {}",
                self.conservation_budget - self.conservation_used
            ));
        }
        self.conservation_used += cost;
        Ok(())
    }

    pub fn describe(&self) -> String {
        let caps: Vec<String> = self.capabilities.iter().map(|c| c.label()).collect();
        format!(
            "[{}] {} ({}) @ {} — {:?} | caps: [{}] | budget: {}/{}{}",
            self.id,
            self.name,
            self.agent_type.label(),
            self.location.describe(),
            self.status,
            caps.join(", "),
            self.conservation_used,
            self.conservation_budget,
            self.model
                .as_ref()
                .map(|m| format!(" | model: {m}"))
                .unwrap_or_default(),
        )
    }
}

// ---------------------------------------------------------------------------
// AgentRegistry
// ---------------------------------------------------------------------------

/// Knows all agents across all locations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentRegistry {
    pub agents: HashMap<String, UnifiedAgent>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, agent: UnifiedAgent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    pub fn unregister(&mut self, id: &str) {
        self.agents.remove(id);
    }

    pub fn get(&self, id: &str) -> Option<&UnifiedAgent> {
        self.agents.get(id)
    }

    pub fn find_by_type(&self, agent_type: &AgentType) -> Vec<&UnifiedAgent> {
        self.agents
            .values()
            .filter(|a| &a.agent_type == agent_type)
            .collect()
    }

    pub fn find_by_capability(&self, cap: &AgentCapability) -> Vec<&UnifiedAgent> {
        self.agents
            .values()
            .filter(|a| a.capabilities.contains(cap))
            .collect()
    }

    pub fn find_by_location(&self, location_type: &str) -> Vec<&UnifiedAgent> {
        self.agents
            .values()
            .filter(|a| match location_type {
                "room" => matches!(a.location, AgentLocation::Room { .. }),
                "repo" => matches!(a.location, AgentLocation::Repo { .. }),
                "shell" => matches!(a.location, AgentLocation::Shell { .. }),
                "remote" => matches!(a.location, AgentLocation::Remote { .. }),
                _ => false,
            })
            .collect()
    }

    pub fn find_available(&self) -> Vec<&UnifiedAgent> {
        self.agents
            .values()
            .filter(|a| a.is_available())
            .collect()
    }

    /// Cheapest agent that has **all** the requested capabilities.
    pub fn find_cheapest(&self, caps: &[AgentCapability]) -> Option<&UnifiedAgent> {
        self.agents
            .values()
            .filter(|a| caps.iter().all(|c| a.capabilities.contains(c)))
            .filter(|a| a.is_available())
            .min_by(|a, b| {
                let ra = a.conservation_budget - a.conservation_used;
                let rb = b.conservation_budget - b.conservation_used;
                rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    pub fn route_to(&self, agent_id: &str) -> Option<AgentLocation> {
        self.agents.get(agent_id).map(|a| a.location.clone())
    }

    pub fn count(&self) -> usize {
        self.agents.len()
    }

    pub fn count_by_type(&self) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        for agent in self.agents.values() {
            *map.entry(agent.agent_type.label()).or_insert(0) += 1;
        }
        map
    }
}

// ---------------------------------------------------------------------------
// AgentMessage
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: String,
    pub to: String,
    pub payload: String,
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// BridgeError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BridgeError {
    AgentNotFound(String),
    NotConnected(String, String),
    LocationUnreachable(String),
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgentNotFound(id) => write!(f, "agent not found: {id}"),
            Self::NotConnected(a, b) => write!(f, "{a} is not connected to {b}"),
            Self::LocationUnreachable(id) => write!(f, "location unreachable for {id}"),
        }
    }
}

impl std::error::Error for BridgeError {}

// ---------------------------------------------------------------------------
// InterShellConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterShellConfig {
    pub local_id: String,
    pub default_protocol: String,
}

// ---------------------------------------------------------------------------
// AgentBridge
// ---------------------------------------------------------------------------

/// Connects agents across locations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBridge {
    pub registry: AgentRegistry,
    pub inter_shell: Option<InterShellConfig>,
    /// Tracks which agents are connected to which.
    connections: HashMap<String, Vec<String>>,
}

impl AgentBridge {
    pub fn new(registry: AgentRegistry) -> Self {
        Self {
            registry,
            inter_shell: None,
            connections: HashMap::new(),
        }
    }

    pub fn connect(&mut self, from: &str, to: &str) -> Result<(), BridgeError> {
        if !self.registry.agents.contains_key(from) {
            return Err(BridgeError::AgentNotFound(from.into()));
        }
        if !self.registry.agents.contains_key(to) {
            return Err(BridgeError::AgentNotFound(to.into()));
        }
        self.connections
            .entry(from.into())
            .or_default()
            .push(to.into());
        // Bidirectional — also add reverse
        self.connections
            .entry(to.into())
            .or_default()
            .push(from.into());
        Ok(())
    }

    pub fn disconnect(&mut self, from: &str, to: &str) {
        if let Some(list) = self.connections.get_mut(from) {
            list.retain(|id| id != to);
        }
        if let Some(list) = self.connections.get_mut(to) {
            list.retain(|id| id != from);
        }
    }

    pub fn send(
        &mut self,
        from: &str,
        to: &str,
        message: AgentMessage,
    ) -> Result<(), BridgeError> {
        if !self.registry.agents.contains_key(from) {
            return Err(BridgeError::AgentNotFound(from.into()));
        }
        if !self.registry.agents.contains_key(to) {
            return Err(BridgeError::AgentNotFound(to.into()));
        }
        let connected = self
            .connections
            .get(from)
            .map(|v| v.contains(&to.to_string()))
            .unwrap_or(false);
        if !connected {
            return Err(BridgeError::NotConnected(from.into(), to.into()));
        }
        // Check that the target location is reachable (local or remote with protocol)
        let target = self.registry.get(to).unwrap();
        if let AgentLocation::Remote { protocol, .. } = &target.location {
            if let Some(cfg) = &self.inter_shell {
                if cfg.default_protocol != *protocol {
                    return Err(BridgeError::LocationUnreachable(to.into()));
                }
            }
        }
        // In a real implementation the message would be dispatched here.
        let _ = message;
        Ok(())
    }

    pub fn broadcast(
        &mut self,
        from: &str,
        message: AgentMessage,
    ) -> Result<(), BridgeError> {
        if !self.registry.agents.contains_key(from) {
            return Err(BridgeError::AgentNotFound(from.into()));
        }
        let targets: Vec<String> = self
            .connections
            .get(from)
            .cloned()
            .unwrap_or_default();
        if targets.is_empty() {
            return Ok(());
        }
        for to in &targets {
            if self.registry.agents.contains_key(to) {
                let _ = self.send(from, to, message.clone());
            }
        }
        Ok(())
    }

    pub fn connections_of(&self, agent_id: &str) -> Vec<String> {
        self.connections
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- AgentType tests --

    #[test]
    fn agent_type_label() {
        assert_eq!(AgentType::RoomEnsign.label(), "RoomEnsign");
        assert_eq!(AgentType::GitNative.label(), "GitNative");
        assert_eq!(AgentType::ZeroClaw.label(), "ZeroClaw");
        assert_eq!(AgentType::CUDAClaw.label(), "CUDAClaw");
        assert_eq!(AgentType::Hermes.label(), "Hermes");
        assert_eq!(AgentType::Custom("Bob".into()).label(), "Bob");
    }

    #[test]
    fn agent_type_is_sandboxed() {
        assert!(AgentType::ZeroClaw.is_sandboxed());
        assert!(!AgentType::RoomEnsign.is_sandboxed());
        assert!(!AgentType::GitNative.is_sandboxed());
        assert!(!AgentType::Hermes.is_sandboxed());
    }

    #[test]
    fn agent_type_needs_room() {
        assert!(AgentType::RoomEnsign.needs_room());
        assert!(!AgentType::GitNative.needs_room());
        assert!(!AgentType::ZeroClaw.needs_room());
    }

    // -- AgentLocation tests --

    #[test]
    fn location_room_describe() {
        let loc = AgentLocation::Room {
            room_id: "r1".into(),
            shell_id: "s1".into(),
        };
        assert_eq!(loc.describe(), "room r1 (shell s1)");
    }

    #[test]
    fn location_repo_describe() {
        let loc = AgentLocation::Repo {
            repo_path: "/tmp/repo".into(),
            branch: Some("main".into()),
        };
        assert_eq!(loc.describe(), "repo /tmp/repo on main");
        let loc2 = AgentLocation::Repo {
            repo_path: "/tmp/repo".into(),
            branch: None,
        };
        assert_eq!(loc2.describe(), "repo /tmp/repo");
    }

    #[test]
    fn location_shell_describe() {
        let loc = AgentLocation::Shell {
            shell_id: "sh1".into(),
            universe_path: "/universe".into(),
        };
        assert_eq!(loc.describe(), "shell sh1 at /universe");
    }

    #[test]
    fn location_remote_describe() {
        let loc = AgentLocation::Remote {
            address: "10.0.0.1".into(),
            protocol: "ssh".into(),
        };
        assert_eq!(loc.describe(), "ssh://10.0.0.1");
    }

    #[test]
    fn location_is_local() {
        assert!(AgentLocation::Room {
            room_id: "r".into(),
            shell_id: "s".into()
        }
        .is_local());
        assert!(AgentLocation::Repo {
            repo_path: "/r".into(),
            branch: None
        }
        .is_local());
        assert!(!AgentLocation::Remote {
            address: "10.0.0.1".into(),
            protocol: "ssh".into()
        }
        .is_local());
    }

    // -- AgentCapability tests --

    #[test]
    fn capability_labels() {
        assert_eq!(AgentCapability::Chat.label(), "Chat");
        assert_eq!(AgentCapability::Code.label(), "Code");
        assert_eq!(AgentCapability::Vision.label(), "Vision");
        assert_eq!(
            AgentCapability::Hardware {
                devices: vec!["gpio".into()]
            }
            .label(),
            "Hardware(gpio)"
        );
        assert_eq!(AgentCapability::Compute { gpu: true }.label(), "Compute(GPU)");
        assert_eq!(AgentCapability::Compute { gpu: false }.label(), "Compute(CPU)");
        assert_eq!(
            AgentCapability::Network {
                protocols: vec!["http".into(), "mqtt".into()]
            }
            .label(),
            "Network(http,mqtt)"
        );
        assert_eq!(AgentCapability::Git.label(), "Git");
        assert_eq!(AgentCapability::Escalate.label(), "Escalate");
    }

    // -- AgentStatus tests --

    #[test]
    fn status_is_active() {
        assert!(AgentStatus::Waking.is_active());
        assert!(AgentStatus::Orienting.is_active());
        assert!(AgentStatus::Ready.is_active());
        assert!(AgentStatus::Working {
            task: "x".into(),
            progress: 0.5
        }
        .is_active());
        assert!(!AgentStatus::Dormant.is_active());
        assert!(!AgentStatus::Offline.is_active());
    }

    #[test]
    fn status_needs_attention() {
        assert!(AgentStatus::YellowAlert.needs_attention());
        assert!(AgentStatus::RedAlert {
            reason: "fire".into()
        }
        .needs_attention());
        assert!(AgentStatus::Error("oops".into()).needs_attention());
        assert!(!AgentStatus::Ready.needs_attention());
        assert!(!AgentStatus::Dormant.needs_attention());
    }

    // -- UnifiedAgent tests --

    #[test]
    fn agent_new() {
        let a = UnifiedAgent::new(
            "Alpha",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r1".into(),
                shell_id: "s1".into(),
            },
        );
        assert_eq!(a.id, "roomensign-alpha");
        assert_eq!(a.name, "Alpha");
        assert_eq!(a.agent_type, AgentType::RoomEnsign);
        assert_eq!(a.status, AgentStatus::Dormant);
        assert!(a.capabilities.is_empty());
        assert!(a.model.is_none());
        assert_eq!(a.conservation_budget, 0.0);
    }

    #[test]
    fn agent_add_capability_no_duplicates() {
        let mut a = UnifiedAgent::new(
            "X",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        a.add_capability(AgentCapability::Chat);
        a.add_capability(AgentCapability::Chat);
        a.add_capability(AgentCapability::Code);
        assert_eq!(a.capabilities.len(), 2);
    }

    #[test]
    fn agent_grant_connection_no_duplicates() {
        let mut a = UnifiedAgent::new(
            "X",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        a.grant_connection("target-1");
        a.grant_connection("target-1");
        assert_eq!(a.connections.len(), 1);
    }

    #[test]
    fn agent_is_available() {
        let mut a = UnifiedAgent::new(
            "X",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        assert!(!a.is_available());
        a.status = AgentStatus::Ready;
        assert!(a.is_available());
    }

    #[test]
    fn agent_budget() {
        let mut a = UnifiedAgent::new(
            "X",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        a.conservation_budget = 100.0;
        assert!(a.can_afford(50.0));
        assert!(a.can_afford(100.0));
        assert!(!a.can_afford(100.1));
        assert!(a.spend(30.0).is_ok());
        assert_eq!(a.conservation_used, 30.0);
        assert!(a.can_afford(70.0));
        assert!(!a.can_afford(71.0));
        assert!(a.spend(71.0).is_err());
    }

    #[test]
    fn agent_describe() {
        let mut a = UnifiedAgent::new(
            "Bravo",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r1".into(),
                shell_id: "s1".into(),
            },
        );
        a.add_capability(AgentCapability::Chat);
        a.model = Some("gpt-4".into());
        let desc = a.describe();
        assert!(desc.contains("Bravo"));
        assert!(desc.contains("RoomEnsign"));
        assert!(desc.contains("Chat"));
        assert!(desc.contains("gpt-4"));
    }

    // -- AgentRegistry tests --

    fn sample_agent(name: &str, atype: AgentType, loc: AgentLocation) -> UnifiedAgent {
        UnifiedAgent::new(name, atype, loc)
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        let id = a.id.clone();
        reg.register(a);
        assert!(reg.get(&id).is_some());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn registry_unregister() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        let id = a.id.clone();
        reg.register(a);
        reg.unregister(&id);
        assert!(reg.get(&id).is_none());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn registry_find_by_type() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        ));
        reg.register(sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r2".into(),
                shell_id: "s2".into(),
            },
        ));
        reg.register(sample_agent(
            "C",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        ));
        assert_eq!(reg.find_by_type(&AgentType::RoomEnsign).len(), 2);
        assert_eq!(reg.find_by_type(&AgentType::GitNative).len(), 1);
        assert_eq!(reg.find_by_type(&AgentType::Hermes).len(), 0);
    }

    #[test]
    fn registry_find_by_capability() {
        let mut reg = AgentRegistry::new();
        let mut a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        a.add_capability(AgentCapability::Code);
        a.add_capability(AgentCapability::Vision);
        reg.register(a);
        let b = sample_agent(
            "B",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        reg.register(b);
        assert_eq!(reg.find_by_capability(&AgentCapability::Code).len(), 1);
        assert_eq!(reg.find_by_capability(&AgentCapability::Vision).len(), 1);
        assert_eq!(reg.find_by_capability(&AgentCapability::Audio).len(), 0);
    }

    #[test]
    fn registry_find_by_location() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        ));
        reg.register(sample_agent(
            "B",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        ));
        reg.register(sample_agent(
            "C",
            AgentType::ZeroClaw,
            AgentLocation::Shell {
                shell_id: "sh".into(),
                universe_path: "/u".into(),
            },
        ));
        assert_eq!(reg.find_by_location("room").len(), 1);
        assert_eq!(reg.find_by_location("repo").len(), 1);
        assert_eq!(reg.find_by_location("shell").len(), 1);
        assert_eq!(reg.find_by_location("remote").len(), 0);
        assert_eq!(reg.find_by_location("bogus").len(), 0);
    }

    #[test]
    fn registry_find_available() {
        let mut reg = AgentRegistry::new();
        let mut a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        a.status = AgentStatus::Ready;
        reg.register(a);
        reg.register(sample_agent(
            "B",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        ));
        assert_eq!(reg.find_available().len(), 1);
    }

    #[test]
    fn registry_find_cheapest() {
        let mut reg = AgentRegistry::new();
        let mut a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        a.add_capability(AgentCapability::Code);
        a.conservation_budget = 50.0;
        a.status = AgentStatus::Ready;
        reg.register(a);

        let mut b = sample_agent(
            "B",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        b.add_capability(AgentCapability::Code);
        b.conservation_budget = 200.0;
        b.status = AgentStatus::Ready;
        reg.register(b);

        let cheapest = reg.find_cheapest(&[AgentCapability::Code]).unwrap();
        assert_eq!(cheapest.id, "gitnative-b");
    }

    #[test]
    fn registry_find_cheapest_none_available() {
        let reg = AgentRegistry::new();
        assert!(reg
            .find_cheapest(&[AgentCapability::Code])
            .is_none());
    }

    #[test]
    fn registry_route_to() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r1".into(),
                shell_id: "s1".into(),
            },
        );
        let id = a.id.clone();
        reg.register(a);
        let route = reg.route_to(&id).unwrap();
        assert_eq!(
            route,
            AgentLocation::Room {
                room_id: "r1".into(),
                shell_id: "s1".into()
            }
        );
        assert!(reg.route_to("nonexistent").is_none());
    }

    #[test]
    fn registry_count_by_type() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        ));
        reg.register(sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r2".into(),
                shell_id: "s2".into(),
            },
        ));
        reg.register(sample_agent(
            "C",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "sh".into(),
                universe_path: "/u".into(),
            },
        ));
        let counts = reg.count_by_type();
        assert_eq!(counts.get("RoomEnsign"), Some(&2));
        assert_eq!(counts.get("Hermes"), Some(&1));
    }

    // -- AgentBridge tests --

    #[test]
    fn bridge_connect_and_send() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s1".into(),
                universe_path: "/u".into(),
            },
        );
        let b = sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        reg.register(a);
        reg.register(b);

        let mut bridge = AgentBridge::new(reg);
        assert!(bridge.connect(&a_id, &b_id).is_ok());
        let msg = AgentMessage {
            from: a_id.clone(),
            to: b_id.clone(),
            payload: "hello".into(),
            timestamp: 1,
        };
        assert!(bridge.send(&a_id, &b_id, msg).is_ok());
    }

    #[test]
    fn bridge_connect_missing_agent() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        let a_id = a.id.clone();
        reg.register(a);
        let mut bridge = AgentBridge::new(reg);
        assert!(matches!(
            bridge.connect(&a_id, "ghost"),
            Err(BridgeError::AgentNotFound(_))
        ));
    }

    #[test]
    fn bridge_send_not_connected() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        let b = sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        reg.register(a);
        reg.register(b);
        let mut bridge = AgentBridge::new(reg);
        let msg = AgentMessage {
            from: a_id.clone(),
            to: b_id.clone(),
            payload: "hello".into(),
            timestamp: 1,
        };
        assert!(matches!(
            bridge.send(&a_id, &b_id, msg),
            Err(BridgeError::NotConnected(_, _))
        ));
    }

    #[test]
    fn bridge_disconnect() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        let b = sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        reg.register(a);
        reg.register(b);
        let mut bridge = AgentBridge::new(reg);
        bridge.connect(&a_id, &b_id).unwrap();
        bridge.disconnect(&a_id, &b_id);
        assert!(bridge.connections_of(&a_id).is_empty());
        assert!(bridge.connections_of(&b_id).is_empty());
    }

    #[test]
    fn bridge_broadcast() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        let b = sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r1".into(),
                shell_id: "s1".into(),
            },
        );
        let c = sample_agent(
            "C",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        let c_id = c.id.clone();
        reg.register(a);
        reg.register(b);
        reg.register(c);
        let mut bridge = AgentBridge::new(reg);
        bridge.connect(&a_id, &b_id).unwrap();
        bridge.connect(&a_id, &c_id).unwrap();
        let msg = AgentMessage {
            from: a_id.clone(),
            to: "all".into(),
            payload: "ping".into(),
            timestamp: 1,
        };
        assert!(bridge.broadcast(&a_id, msg).is_ok());
        assert_eq!(bridge.connections_of(&a_id).len(), 2);
    }

    #[test]
    fn bridge_connections_of() {
        let bridge = AgentBridge::new(AgentRegistry::new());
        assert!(bridge.connections_of("nobody").is_empty());
    }

    // -- Serde round-trip tests --

    #[test]
    fn serde_agent_type() {
        let orig = AgentType::Custom("Foo".into());
        let json = serde_json::to_string(&orig).unwrap();
        let back: AgentType = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_agent_location() {
        let orig = AgentLocation::Repo {
            repo_path: "/tmp".into(),
            branch: Some("main".into()),
        };
        let json = serde_json::to_string(&orig).unwrap();
        let back: AgentLocation = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_agent_capability() {
        let orig = AgentCapability::Network {
            protocols: vec!["http".into(), "mqtt".into()],
        };
        let json = serde_json::to_string(&orig).unwrap();
        let back: AgentCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_agent_status() {
        let orig = AgentStatus::Working {
            task: "building".into(),
            progress: 0.42,
        };
        let json = serde_json::to_string(&orig).unwrap();
        let back: AgentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_unified_agent() {
        let mut orig = UnifiedAgent::new(
            "Delta",
            AgentType::CUDAClaw,
            AgentLocation::Shell {
                shell_id: "gpu1".into(),
                universe_path: "/gpu".into(),
            },
        );
        orig.add_capability(AgentCapability::Compute { gpu: true });
        orig.add_capability(AgentCapability::Code);
        orig.conservation_budget = 500.0;
        orig.model = Some("llama-3".into());
        orig.parent = Some("hermes".into());

        let json = serde_json::to_string(&orig).unwrap();
        let back: UnifiedAgent = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_registry() {
        let mut reg = AgentRegistry::new();
        reg.register(sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        ));
        let json = serde_json::to_string(&reg).unwrap();
        let back: AgentRegistry = serde_json::from_str(&json).unwrap();
        assert_eq!(reg.count(), back.count());
    }

    #[test]
    fn serde_bridge_error() {
        let orig = BridgeError::NotConnected("a".into(), "b".into());
        let json = serde_json::to_string(&orig).unwrap();
        let back: BridgeError = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn serde_agent_message() {
        let orig = AgentMessage {
            from: "a".into(),
            to: "b".into(),
            payload: "hi".into(),
            timestamp: 12345,
        };
        let json = serde_json::to_string(&orig).unwrap();
        let back: AgentMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(orig, back);
    }

    #[test]
    fn bridge_send_agent_not_found() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        reg.register(a);
        let mut bridge = AgentBridge::new(reg);
        let msg = AgentMessage {
            from: "ghost".into(),
            to: "also-ghost".into(),
            payload: "hello".into(),
            timestamp: 1,
        };
        assert!(matches!(
            bridge.send("ghost", "also-ghost", msg),
            Err(BridgeError::AgentNotFound(_))
        ));
    }

    #[test]
    fn registry_find_cheapest_missing_capability() {
        let mut reg = AgentRegistry::new();
        let mut a = sample_agent(
            "A",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        a.add_capability(AgentCapability::Chat);
        a.status = AgentStatus::Ready;
        reg.register(a);
        // A only has Chat, not Code
        assert!(reg.find_cheapest(&[AgentCapability::Code]).is_none());
    }

    #[test]
    fn agent_spend_exact_budget() {
        let mut a = sample_agent(
            "X",
            AgentType::GitNative,
            AgentLocation::Repo {
                repo_path: "/r".into(),
                branch: None,
            },
        );
        a.conservation_budget = 100.0;
        assert!(a.spend(100.0).is_ok());
        assert!(!a.can_afford(0.01));
    }

    #[test]
    fn bridge_error_display() {
        let e = BridgeError::AgentNotFound("x".into());
        assert_eq!(format!("{e}"), "agent not found: x");
        let e = BridgeError::NotConnected("a".into(), "b".into());
        assert_eq!(format!("{e}"), "a is not connected to b");
        let e = BridgeError::LocationUnreachable("x".into());
        assert_eq!(format!("{e}"), "location unreachable for x");
    }

    #[test]
    fn agent_describe_no_model() {
        let a = sample_agent(
            "Z",
            AgentType::RoomEnsign,
            AgentLocation::Room {
                room_id: "r".into(),
                shell_id: "s".into(),
            },
        );
        assert!(!a.describe().contains("model:"));
    }

    #[test]
    fn inter_shell_config_serde() {
        let cfg = InterShellConfig {
            local_id: "me".into(),
            default_protocol: "ssh".into(),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: InterShellConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }

    #[test]
    fn bridge_send_remote_protocol_mismatch() {
        let mut reg = AgentRegistry::new();
        let a = sample_agent(
            "A",
            AgentType::Hermes,
            AgentLocation::Shell {
                shell_id: "s".into(),
                universe_path: "/u".into(),
            },
        );
        let mut b = sample_agent(
            "B",
            AgentType::RoomEnsign,
            AgentLocation::Remote {
                address: "10.0.0.1".into(),
                protocol: "grpc".into(),
            },
        );
        b.status = AgentStatus::Ready;
        let a_id = a.id.clone();
        let b_id = b.id.clone();
        reg.register(a);
        reg.register(b);
        let mut bridge = AgentBridge::new(reg);
        bridge.inter_shell = Some(InterShellConfig {
            local_id: "local".into(),
            default_protocol: "ssh".into(),
        });
        bridge.connect(&a_id, &b_id).unwrap();
        let msg = AgentMessage {
            from: a_id.clone(),
            to: b_id.clone(),
            payload: "hi".into(),
            timestamp: 1,
        };
        // Protocol mismatch: bridge expects ssh, target is grpc
        assert!(matches!(
            bridge.send(&a_id, &b_id, msg),
            Err(BridgeError::LocationUnreachable(_))
        ));
    }
}

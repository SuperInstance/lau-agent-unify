# lau-agent-unify

> Unified type system for room ensigns and git-native agents

## What This Does

Unified type system for room ensigns and git-native agents. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-agent-unify
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_agent_unify::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum AgentType 
    pub fn is_sandboxed(&self) -> bool 
    pub fn needs_room(&self) -> bool 
    pub fn label(&self) -> String 
pub enum AgentLocation 
    pub fn describe(&self) -> String 
    pub fn is_local(&self) -> bool 
pub enum AgentCapability 
    pub fn label(&self) -> String 
pub enum AgentStatus 
    pub fn is_active(&self) -> bool 
    pub fn needs_attention(&self) -> bool 
pub struct UnifiedAgent 
    pub fn new(name: &str, agent_type: AgentType, location: AgentLocation) -> Self 
    pub fn add_capability(&mut self, cap: AgentCapability) 
    pub fn grant_connection(&mut self, target: &str) 
    pub fn is_available(&self) -> bool 
    pub fn can_afford(&self, cost: f64) -> bool 
    pub fn spend(&mut self, cost: f64) -> Result<(), String> 
    pub fn describe(&self) -> String 
pub struct AgentRegistry 
    pub fn new() -> Self 
    pub fn register(&mut self, agent: UnifiedAgent) 
    pub fn unregister(&mut self, id: &str) 
    pub fn get(&self, id: &str) -> Option<&UnifiedAgent> 
    pub fn find_by_type(&self, agent_type: &AgentType) -> Vec<&UnifiedAgent> 
    pub fn find_by_capability(&self, cap: &AgentCapability) -> Vec<&UnifiedAgent> 
    pub fn find_by_location(&self, location_type: &str) -> Vec<&UnifiedAgent> 
    pub fn find_available(&self) -> Vec<&UnifiedAgent> 
    pub fn find_cheapest(&self, caps: &[AgentCapability]) -> Option<&UnifiedAgent> 
    pub fn route_to(&self, agent_id: &str) -> Option<AgentLocation> 
    pub fn count(&self) -> usize 
    pub fn count_by_type(&self) -> HashMap<String, usize> 
pub struct AgentMessage 
pub enum BridgeError 
pub struct InterShellConfig 
pub struct AgentBridge 
    pub fn new(registry: AgentRegistry) -> Self 
    pub fn connect(&mut self, from: &str, to: &str) -> Result<(), BridgeError> 
    pub fn disconnect(&mut self, from: &str, to: &str) 
    pub fn send(
    pub fn broadcast(
    pub fn connections_of(&self, agent_id: &str) -> Vec<String> 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**48 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT

/*!
# cuda-memory-fabric

Multi-layer memory for agents.

Humans don't have one memory system. We have four:

1. **Working memory** — what's in your head right now (7±2 items, ~20s decay)
2. **Episodic memory** — what happened to you (when, where, who)
3. **Semantic memory** — what you know (facts, concepts, relationships)
4. **Procedural memory** — how to do things (skills, habits, reflexes)

An agent needs the same layered architecture. Working memory for the current
task, episodic for experience, semantic for knowledge, procedural for skills.
Each layer has different capacity, decay, and access patterns.
*/

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// Memory entry with confidence and timestamp
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: u64,
    pub content: String,
    pub confidence: f64,
    pub created: u64,
    pub last_accessed: u64,
    pub access_count: u32,
    pub tags: Vec<String>,
    pub layer: MemoryLayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryLayer {
    Working,     // fast, small, decays in seconds
    Episodic,    // medium, experiences with context
    Semantic,    // slow, large, facts and concepts
    Procedural,  // skills and habits
}

/// Working memory — bounded ring buffer with decay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkingMemory {
    pub items: VecDeque<WorkingItem>,
    pub capacity: usize,
    pub decay_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkingItem {
    pub content: String,
    pub tick: u64,
    pub confidence: f64,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self { WorkingMemory { items: VecDeque::with_capacity(capacity), capacity, decay_ticks: 20 } }

    pub fn push(&mut self, item: WorkingItem) {
        if self.items.len() >= self.capacity { self.items.pop_front(); }
        self.items.push_back(item);
    }

    pub fn recall(&self, query: &str) -> Vec<&WorkingItem> {
        self.items.iter()
            .filter(|i| i.content.contains(query) || i.content.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    pub fn decay(&mut self, current_tick: u64) {
        self.items.retain(|i| current_tick - i.tick < self.decay_ticks as u64 * 1000);
    }

    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

/// Episodic memory — time-indexed experiences
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpisodicMemory {
    pub episodes: Vec<Episode>,
    pub capacity: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Episode {
    pub id: u64,
    pub what: String,
    pub when: u64,         // timestamp
    pub where_loc: String, // spatial context
    pub who: Vec<String>,  // other agents involved
    pub outcome: String,   // what happened
    pub confidence: f64,
    pub emotional_valence: f64, // -1.0 (bad) to 1.0 (good)
    pub importance: f64,   // 0-1, affects retention
}

impl EpisodicMemory {
    pub fn new(capacity: usize) -> Self { EpisodicMemory { episodes: vec![], capacity } }

    pub fn store(&mut self, episode: Episode) {
        if self.episodes.len() >= self.capacity {
            // Evict least important
            if let Some(idx) = self.episodes.iter().enumerate().min_by_key(|(_, e)| (e.importance * 1000.0) as u64).map(|(i, _)| i) {
                self.episodes.remove(idx);
            }
        }
        self.episodes.push(episode);
    }

    /// Recall recent episodes
    pub fn recent(&self, n: usize) -> Vec<&Episode> {
        self.episodes.iter().rev().take(n).collect()
    }

    /// Recall episodes involving specific agent
    pub fn involving(&self, agent: &str) -> Vec<&Episode> {
        self.episodes.iter().filter(|e| e.who.iter().any(|a| a == agent)).collect()
    }

    /// Recall episodes at a location
    pub fn at_location(&self, loc: &str) -> Vec<&Episode> {
        self.episodes.iter().filter(|e| e.where_loc == loc).collect()
    }

    /// Recall similar episodes (substring match on what)
    pub fn similar(&self, what: &str) -> Vec<&Episode> {
        self.episodes.iter()
            .filter(|e| e.what.contains(what) || e.outcome.contains(what))
            .collect()
    }

    /// Forgetting curve — decay confidence based on time since creation
    pub fn apply_forgetting(&mut self, current_time: u64, half_life_ms: u64) {
        for ep in &mut self.episodes {
            let age = current_time.saturating_sub(ep.when);
            let decay = 0.5_f64.powf(age as f64 / half_life_ms as f64);
            ep.confidence *= decay;
        }
        self.episodes.retain(|e| e.confidence > 0.01);
    }
}

/// Semantic memory — knowledge graph of facts and concepts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub facts: HashMap<String, Fact>,
    pub concepts: HashMap<String, Concept>,
    pub relations: Vec<Relation>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fact {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub source: String,
    pub created: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Concept {
    pub name: String,
    pub description: String,
    pub related: Vec<String>,
    pub confidence: f64,
    pub usage_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relation {
    pub from: String,
    pub relation_type: String,
    pub to: String,
    pub confidence: f64,
}

impl SemanticMemory {
    pub fn new() -> Self { SemanticMemory { facts: HashMap::new(), concepts: HashMap::new(), relations: vec![] } }

    pub fn add_fact(&mut self, fact: Fact) {
        let key = format!("{}:{}:{}", fact.subject, fact.predicate, fact.object);
        self.facts.insert(key, fact);
    }

    pub fn query_fact(&self, subject: &str, predicate: &str) -> Vec<&Fact> {
        self.facts.values()
            .filter(|f| f.subject == subject && f.predicate == predicate)
            .collect()
    }

    pub fn add_concept(&mut self, concept: Concept) {
        let name = concept.name.clone();
        self.concepts.insert(name, concept);
    }

    pub fn get_concept(&self, name: &str) -> Option<&Concept> {
        self.concepts.get(name)
    }

    pub fn add_relation(&mut self, rel: Relation) {
        self.relations.push(rel);
    }

    /// Traverse relations from a concept
    pub fn traverse(&self, from: &str, max_depth: usize) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(from.to_string(), 0)];
        let mut results = vec![];

        while let Some((node, depth)) = queue.pop() {
            if depth > max_depth { continue; }
            if visited.contains(&node) { continue; }
            visited.insert(node.clone());
            results.push(node.clone());

            for rel in &self.relations {
                if rel.from == node && !visited.contains(&rel.to) {
                    queue.push((rel.to.clone(), depth + 1));
                }
            }
        }
        results
    }

    /// Consolidation — merge similar facts, boost frequently accessed concepts
    pub fn consolidate(&mut self) {
        for concept in self.concepts.values_mut() {
            // Concepts used more get higher confidence
            let boost = 1.0 + concept.usage_count as f64 * 0.01;
            concept.confidence = (concept.confidence * boost).min(1.0);
        }
    }
}

/// Procedural memory — learned skills and behaviors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProceduralMemory {
    pub skills: HashMap<String, Skill>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub steps: Vec<String>,
    pub confidence: f64,      // how well-learned
    pub practice_count: u32,
    pub last_practiced: u64,
    pub automaticity: f64,    // 0=conscious, 1=automatic (no working memory needed)
    pub context: Vec<String>, // when to use this skill
}

impl ProceduralMemory {
    pub fn new() -> Self { ProceduralMemory { skills: HashMap::new() } }

    pub fn learn(&mut self, skill: Skill) {
        self.skills.insert(skill.name.clone(), skill);
    }

    pub fn practice(&mut self, name: &str) {
        if let Some(skill) = self.skills.get_mut(name) {
            skill.practice_count += 1;
            skill.last_practiced = now();
            // Power law of practice: skill improves logarithmically
            skill.confidence = (0.5 + 0.5 * (1.0 - 1.0 / (1.0 + skill.practice_count as f64 * 0.1))).min(1.0);
            skill.automaticity = (skill.automaticity + 0.05).min(1.0);
        }
    }

    pub fn recall(&self, context: &str) -> Vec<&Skill> {
        self.skills.values()
            .filter(|s| s.context.iter().any(|c| c.contains(context) || context.contains(c)))
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&Skill> { self.skills.get(name) }
}

/// The full memory fabric — all four layers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryFabric {
    pub working: WorkingMemory,
    pub episodic: EpisodicMemory,
    pub semantic: SemanticMemory,
    pub procedural: ProceduralMemory,
}

impl MemoryFabric {
    pub fn new() -> Self {
        MemoryFabric { working: WorkingMemory::new(7), episodic: EpisodicMemory::new(1000), semantic: SemanticMemory::new(), procedural: ProceduralMemory::new() }
    }

    /// Full consolidation pass
    pub fn consolidate(&mut self, current_time: u64) {
        self.working.decay(current_time);
        self.episodic.apply_forgetting(current_time, 3600_000); // 1 hour half-life
        self.semantic.consolidate();
    }

    /// Search all layers for a query
    pub fn search(&self, query: &str) -> Vec<MemorySearchResult> {
        let mut results = vec![];

        // Working
        for item in &self.working.items {
            if item.content.contains(query) {
                results.push(MemorySearchResult { layer: MemoryLayer::Working, content: item.content.clone(), confidence: item.confidence });
            }
        }

        // Episodic
        for ep in &self.episodic.episodes {
            if ep.what.contains(query) || ep.outcome.contains(query) {
                results.push(MemorySearchResult { layer: MemoryLayer::Episodic, content: ep.what.clone(), confidence: ep.confidence });
            }
        }

        // Semantic
        if let Some(c) = self.semantic.get_concept(query) {
            results.push(MemorySearchResult { layer: MemoryLayer::Semantic, content: c.description.clone(), confidence: c.confidence });
        }
        for fact in self.semantic.facts.values() {
            if fact.subject.contains(query) || fact.object.contains(query) {
                results.push(MemorySearchResult { layer: MemoryLayer::Semantic, content: format!("{} {} {}", fact.subject, fact.predicate, fact.object), confidence: fact.confidence });
            }
        }

        // Procedural
        for skill in self.procedural.skills.values() {
            if skill.name.contains(query) || skill.context.iter().any(|c| c.contains(query)) {
                results.push(MemorySearchResult { layer: MemoryLayer::Procedural, content: skill.name.clone(), confidence: skill.confidence });
            }
        }

        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        results
    }
}

#[derive(Clone, Debug)]
pub struct MemorySearchResult {
    pub layer: MemoryLayer,
    pub content: String,
    pub confidence: f64,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_push_recall() {
        let mut wm = WorkingMemory::new(3);
        wm.push(WorkingItem { content: "find food".into(), tick: 1000, confidence: 0.8 });
        wm.push(WorkingItem { content: "avoid danger".into(), tick: 1001, confidence: 0.9 });
        let r = wm.recall("food");
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_working_capacity() {
        let mut wm = WorkingMemory::new(2);
        wm.push(WorkingItem { content: "a".into(), tick: 1, confidence: 1.0 });
        wm.push(WorkingItem { content: "b".into(), tick: 2, confidence: 1.0 });
        wm.push(WorkingItem { content: "c".into(), tick: 3, confidence: 1.0 });
        assert_eq!(wm.len(), 2); // "a" evicted
    }

    #[test]
    fn test_working_decay() {
        let mut wm = WorkingMemory::new(10);
        wm.decay_ticks = 5;
        wm.push(WorkingItem { content: "old".into(), tick: 1000, confidence: 1.0 });
        wm.push(WorkingItem { content: "new".into(), tick: 5900, confidence: 1.0 });
        wm.decay(6000); // old is 5000ms old (5 * 1000), new is 100ms old
        assert_eq!(wm.len(), 1);
    }

    #[test]
    fn test_episodic_store_recall() {
        let mut em = EpisodicMemory::new(100);
        em.store(Episode { id: 1, what: "found water".into(), when: 1000, where_loc: "lake".into(), who: vec!["scout".into()], outcome: "success".into(), confidence: 0.9, emotional_valence: 0.8, importance: 0.7 });
        assert_eq!(em.recent(1).len(), 1);
    }

    #[test]
    fn test_episodic_involving() {
        let mut em = EpisodicMemory::new(100);
        em.store(Episode { id: 1, what: "met".into(), when: 1, where_loc: "x".into(), who: vec!["alice".into()], outcome: "ok".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        em.store(Episode { id: 2, what: "met".into(), when: 2, where_loc: "y".into(), who: vec!["bob".into()], outcome: "ok".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        assert_eq!(em.involving("alice").len(), 1);
    }

    #[test]
    fn test_episodic_forgetting() {
        let mut em = EpisodicMemory::new(100);
        em.store(Episode { id: 1, what: "old".into(), when: 0, where_loc: "x".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        em.apply_forgetting(100000, 10000); // 100s, half-life 10s = ~3 halvings
        assert!(em.episodes[0].confidence < 0.2);
    }

    #[test]
    fn test_semantic_fact() {
        let mut sm = SemanticMemory::new();
        sm.add_fact(Fact { subject: "water".into(), predicate: "boils_at".into(), object: "100C".into(), confidence: 0.95, source: "physics".into(), created: 0 });
        let results = sm.query_fact("water", "boils_at");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].object, "100C");
    }

    #[test]
    fn test_semantic_traverse() {
        let mut sm = SemanticMemory::new();
        sm.add_relation(Relation { from: "A".into(), relation_type: "related".into(), to: "B".into(), confidence: 0.8 });
        sm.add_relation(Relation { from: "B".into(), relation_type: "related".into(), to: "C".into(), confidence: 0.8 });
        let results = sm.traverse("A", 2);
        assert!(results.contains(&"A".to_string()));
        assert!(results.contains(&"B".to_string()));
        assert!(results.contains(&"C".to_string()));
    }

    #[test]
    fn test_procedural_practice() {
        let mut pm = ProceduralMemory::new();
        pm.learn(Skill { name: "fish".into(), steps: vec!["cast".into(), "wait".into(), "reel".into()], confidence: 0.3, practice_count: 0, last_practiced: 0, automaticity: 0.0, context: vec!["water".into()] });
        pm.practice("fish");
        pm.practice("fish");
        pm.practice("fish");
        let skill = pm.get("fish").unwrap();
        assert!(skill.practice_count == 3);
        assert!(skill.confidence > 0.3);
    }

    #[test]
    fn test_procedural_recall() {
        let mut pm = ProceduralMemory::new();
        pm.learn(Skill { name: "navigate".into(), steps: vec![], confidence: 0.8, practice_count: 10, last_practiced: 0, automaticity: 0.9, context: vec!["maze".into()] });
        let results = pm.recall("maze");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_fabric_search() {
        let mut fabric = MemoryFabric::new();
        fabric.working.push(WorkingItem { content: "current task: build".into(), tick: 1, confidence: 0.9 });
        fabric.semantic.add_fact(Fact { subject: "rust".into(), predicate: "is".into(), object: "language".into(), confidence: 0.95, source: "self".into(), created: 0 });
        let results = fabric.search("rust");
        assert!(results.len() >= 1);
    }

    #[test]
    fn test_episodic_eviction() {
        let mut em = EpisodicMemory::new(2);
        em.store(Episode { id: 1, what: "first".into(), when: 1, where_loc: "x".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        em.store(Episode { id: 2, what: "second".into(), when: 2, where_loc: "x".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.9 });
        em.store(Episode { id: 3, what: "third".into(), when: 3, where_loc: "x".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.7 });
        assert_eq!(em.episodes.len(), 2); // first evicted (lowest importance)
    }

    #[test]
    fn test_semantic_concept() {
        let mut sm = SemanticMemory::new();
        sm.add_concept(Concept { name: "agent".into(), description: "autonomous entity".into(), related: vec!["fleet".into()], confidence: 0.9, usage_count: 5 });
        let c = sm.get_concept("agent").unwrap();
        assert_eq!(c.description, "autonomous entity");
    }

    #[test]
    fn test_episodic_at_location() {
        let mut em = EpisodicMemory::new(100);
        em.store(Episode { id: 1, what: "event".into(), when: 1, where_loc: "dock".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        em.store(Episode { id: 2, what: "event".into(), when: 2, where_loc: "forest".into(), who: vec![], outcome: "".into(), confidence: 0.9, emotional_valence: 0.0, importance: 0.5 });
        assert_eq!(em.at_location("dock").len(), 1);
    }
}

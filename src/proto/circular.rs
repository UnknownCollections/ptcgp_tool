use crate::proto::message::ProtoMessage;
use crate::unity::generated::CIl2Cpp::TypeIndex;
use hashbrown::HashMap;
use nohash_hasher::IntSet;
use petgraph::algo::tarjan_scc;
use petgraph::graph::Graph;

/// A group of protocol messages that are strongly connected based on type dependencies.
///
/// This struct groups together messages that depend on one another (i.e. forming a cycle)
/// so they can be processed as a single unit.
#[derive(Clone, PartialEq)]
pub struct ProtoMessageGroup(#[doc(hidden)] pub Vec<ProtoMessage>);

/// A collection of `ProtoMessageGroup`s.
pub type ProtoMessageGroups = Vec<ProtoMessageGroup>;

/// Groups protocol messages into sets based on circular dependencies.
///
/// This function builds a dependency graph where each node represents a message (using its
/// index in the input vector) and each directed edge represents a dependency from one message
/// to another (determined via used types). It then applies Tarjan's algorithm to identify
/// strongly connected components (SCCs), which represent groups of mutually dependent messages.
///
/// # Arguments
///
/// * `messages` - A vector of `ProtoMessage` instances to be grouped.
///
/// # Returns
///
/// A vector of `ProtoMessageGroup`, where each group contains messages that are interdependent.
///
/// # Panics
///
/// This function panics if an expected message is missing from the temporary mapping.
pub fn messages_to_message_groups(messages: Vec<ProtoMessage>) -> ProtoMessageGroups {
    // 1. Build a graph whose nodes are indices into the `messages` vector.
    let mut graph = Graph::<usize, ()>::new();
    let mut node_indices = Vec::new();

    for (i, msg) in messages.iter().enumerate() {
        // Add the index (i) as the node payload.
        let idx = graph.add_node(i);
        node_indices.push((msg.type_index, idx));
    }

    // Build a lookup mapping from TypeIndex to the corresponding Graph NodeIndex.
    let index_map: HashMap<TypeIndex, _> = node_indices.into_iter().collect();

    // 2. Add directed edges based on type dependencies.
    for node_idx in graph.node_indices() {
        let msg_idx = graph[node_idx];
        let msg = &messages[msg_idx];
        for used in msg.get_used_types() {
            if let Some(&target_idx) = index_map.get(&used) {
                // Create an edge from the current message to the dependent message.
                graph.add_edge(node_idx, target_idx, ());
            }
        }
    }

    // 3. Identify strongly connected components (SCCs) in the graph.
    let sccs = tarjan_scc(&graph);

    // 4. Convert the messages vector into a map for efficient extraction.
    let mut messages_map: HashMap<usize, ProtoMessage> = messages.into_iter().enumerate().collect();

    // 5. For each SCC, extract the messages to form a group.
    let groups = sccs
        .into_iter()
        .map(|component| {
            let group_messages: Vec<ProtoMessage> = component
                .into_iter()
                .map(|node_idx| {
                    let msg_idx = graph[node_idx];
                    messages_map.remove(&msg_idx).expect("Message not found")
                })
                .collect();
            ProtoMessageGroup(group_messages)
        })
        .collect();

    debug_assert!(messages_map.is_empty(), "Some messages were not processed");

    groups
}

impl ProtoMessageGroup {
    /// Returns the set of type indices used by any message in the group.
    ///
    /// This is computed by merging the used types from each message.
    pub fn get_used_types(&self) -> IntSet<TypeIndex> {
        let mut used_types = IntSet::default();
        for msg in &self.0 {
            used_types.extend(msg.get_used_types());
        }
        used_types
    }

    /// Returns the set of type indices contained within the group.
    ///
    /// This includes the type index for each message and any nested types.
    pub fn get_contained_types(&self) -> IntSet<TypeIndex> {
        let mut contained_types = IntSet::default();
        for msg in &self.0 {
            contained_types.extend(msg.get_contained_types());
        }
        contained_types
    }

    /// Determines the primary message within the group.
    ///
    /// When the group contains multiple messages, the primary is chosen based on the frequency
    /// of its type index among the used types that are also contained in the group. If no
    /// clear candidate is found, the first message is returned.
    ///
    /// # Returns
    ///
    /// A reference to the primary `ProtoMessage` in the group.
    pub fn get_primary(&self) -> &ProtoMessage {
        if self.0.len() == 1 {
            return &self.0[0];
        }
        let all_contained = self.get_contained_types();
        let counts = self
            .0
            .iter()
            .flat_map(|msg| msg.get_used_types())
            .filter(|ty_idx| all_contained.contains(ty_idx))
            .fold(HashMap::new(), |mut map, ty_idx| {
                *map.entry(ty_idx).or_insert(0) += 1;
                map
            });

        if let Some((best_ty_idx, _)) = counts.into_iter().max_by_key(|&(_, count)| count) {
            self.0
                .iter()
                .find(|msg| msg.type_index == best_ty_idx)
                .unwrap_or_else(|| self.0.first().unwrap())
        } else {
            self.0.first().unwrap()
        }
    }

    /// Returns an iterator over the messages in the group.
    pub fn iter(&self) -> impl Iterator<Item = &ProtoMessage> {
        self.0.iter()
    }
}

use crate::language_types::{boolean::JsBoolean, object::JsObject};

/// https://tc39.es/ecma262/#sec-agents
pub struct Agent {
  agent_record: AgentRecord,
}

/// https://tc39.es/ecma262/#agent-record
struct AgentRecord {
  little_endian: JsBoolean,
  can_block: JsBoolean,
  signifier: usize,
  is_lock_free1: JsBoolean,
  is_lock_free2: JsBoolean,
  is_lock_free8: JsBoolean,
  candidate_execution: CandidateExecution,
  kept_alive: Vec<JsObject>,
}

/// TODO
/// https://tc39.es/ecma262/#sec-candidate-executions
struct CandidateExecution {}

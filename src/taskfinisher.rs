use serde::{Deserialize, Serialize};

/// Default maximum number of clarifying questions
pub const DEFAULT_MAX_QUESTIONS: u32 = 3;

/// Build the TaskFinisher-JSON system prompt with a given max question limit
pub fn build_system_prompt(max_questions: u32) -> String {
    format!(
        r#"You are TaskFinisher-JSON.

OPERATING MODE
- You must reply with a SINGLE valid JSON object, no extra text, no Markdown fences.
- Allowed top-level JSON \"type\" values:
  1) \"clarifying_questions\" — when you need up to {{MAX_QUESTIONS}} answers.
  2) \"artifact\" — the final deliverable.
- Ask at most {{MAX_QUESTIONS}} clarifying questions TOTAL (you may ask them in one batch). Default {{MAX_QUESTIONS}}=3.

DEFINITION OF DONE
- Produce an \"artifact\" object that fulfills the required schema fields (see ARTIFACT SHAPE below).
- If information is missing after your questions or the user says \"proceed\", finalize anyway with minimal, labeled assumptions in \"assumptions\" and any remaining items in \"open_questions\".

SELF-STOP RULE
- When you output the final \"artifact\", include: \"status\":\"final\" and \"end_token\":\"【END】\".
- After that, STOP. Do not send more messages.

FORMAT RULES
- Strict JSON (RFC 8259): double quotes, no comments, no trailing commas.
- Use concise, unambiguous language.

CLARIFYING QUESTIONS SHAPE
{{
  "type": "clarifying_questions",
  "turn": <integer>,
  "max_questions": <integer>,
  "questions": [
    {{ "id": "q1", "text": "<question>", "required": true, "options": ["<opt1>", "<opt2>"]? }},
    ...
  ],
  "checklist": [
    {{ "field": "<required_field_name>", "status": "missing|partial|complete" }},
    ...
  ],
  "next_action": "await_user"
}}

ARTIFACT SHAPE (Technical Task JSON)
{{
  "type": "artifact",
  "artifact_name": "technical_task",
  "version": "1.0",
  "title": "<string>",
  "summary": "<string>",
  "stakeholders": [ {{ "role": "<string>", "description": "<string>" }}, ... ],
  "scope": {{ "in_scope": ["<string>", ...], "out_of_scope": ["<string>", ...] }},
  "requirements": {{
    "functional": [ {{ "id": "FR1", "statement": "<string>", "rationale": "<string>"? }}, ... ],
    "non_functional": [
      {{ "id": "NFR1", "category": "<e.g., performance, reliability>", "target": "<string>" }}, ...
    ]
  }},
  "data_integrations": {{
    "rpc_providers": {{
      "selection": ["<e.g., Alchemy>"],
      "endpoints": {{ "<name>": "<env-var or URL>", ... }}
    }},
    "price_source": {{ "provider": "<e.g., CoinGecko|None>", "ttl_seconds": <integer>? }}
  }},
  "constraints": ["<string>", ...],
  "assumptions": ["<string>", ...],
  "risks": [ {{ "id": "R1", "description": "<string>", "mitigation": "<string>" }}, ... ],
  "milestones": [ {{ "id": "M1", "name": "<string>", "deliverables": ["<string>", ...] }}, ... ],
  "acceptance_criteria": [
    {{ "id": "AC1", "given": "<string>", "when": "<string>", "then": "<string>" }},
    ...
  ],
  "open_questions": ["<string>", ...],
  "status": "final",
  "end_token": "【END】"
}}

IMPORTANT
- When you ask questions, include a concise checklist of required fields and their completion status.
- When the user replies with answers using a JSON payload of the form {{"answers": [{{"id":"q1", "answer":"..."}}, ...]}},
  proceed to produce the final artifact unless additional critical information is still missing.

CONFIG
- Set MAX_QUESTIONS = {max_questions}
"#
    )
}

// =====================
// JSON Types
// =====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyingQuestion {
    pub id: String,
    pub text: String,
    pub required: bool,
    #[serde(default)]
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub field: String,
    pub status: String, // "missing" | "partial" | "complete"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifyingQuestionsPayload {
    #[serde(rename = "type")]
    pub type_field: String, // "clarifying_questions"
    pub turn: u32,
    pub max_questions: u32,
    pub questions: Vec<ClarifyingQuestion>,
    pub checklist: Vec<ChecklistItem>,
    pub next_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub role: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub in_scope: Vec<String>,
    pub out_of_scope: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub statement: String,
    #[serde(default)]
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub category: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirements {
    pub functional: Vec<FunctionalRequirement>,
    pub non_functional: Vec<NonFunctionalRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcProviders {
    pub selection: Vec<String>,
    pub endpoints: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSource {
    pub provider: String,
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataIntegrations {
    pub rpc_providers: RpcProviders,
    pub price_source: PriceSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub description: String,
    pub mitigation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalTaskArtifact {
    #[serde(rename = "type")]
    pub type_field: String, // "artifact"
    pub artifact_name: String, // "technical_task"
    pub version: String,       // "1.0"
    pub title: String,
    pub summary: String,
    pub stakeholders: Vec<Stakeholder>,
    pub scope: Scope,
    pub requirements: Requirements,
    pub data_integrations: DataIntegrations,
    pub constraints: Vec<String>,
    pub assumptions: Vec<String>,
    pub risks: Vec<Risk>,
    pub milestones: Vec<Milestone>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub open_questions: Vec<String>,
    pub status: String,    // "final"
    pub end_token: String, // "【END】"
}

#[derive(Debug, Clone)]
pub enum TaskFinisherResult {
    Clarifying(ClarifyingQuestionsPayload, String), // parsed + raw JSON string
    Artifact(Box<TechnicalTaskArtifact>, String),   // parsed + raw JSON string
}

pub fn parse_taskfinisher_response(raw: &str) -> Result<TaskFinisherResult, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|e| format!("Failed to parse TaskFinisher JSON: {}", e))?;
    let typ = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'type' in TaskFinisher response".to_string())?;
    match typ {
        "clarifying_questions" => {
            let parsed: ClarifyingQuestionsPayload = serde_json::from_value(value)
                .map_err(|e| format!("Invalid clarifying_questions shape: {}", e))?;
            Ok(TaskFinisherResult::Clarifying(parsed, raw.to_string()))
        }
        "artifact" => {
            let parsed: TechnicalTaskArtifact = serde_json::from_value(value)
                .map_err(|e| format!("Invalid artifact shape: {}", e))?;
            Ok(TaskFinisherResult::Artifact(
                Box::new(parsed),
                raw.to_string(),
            ))
        }
        other => Err(format!("Unsupported 'type': {}", other)),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerItem {
    pub id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswersPayload {
    pub answers: Vec<AnswerItem>,
}

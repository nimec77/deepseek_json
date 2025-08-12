use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::deepseek::{ChatMessage, DeepSeekClient, DeepSeekError, DeepSeekResponse};
use crate::taskfinisher::{
    build_system_prompt, parse_taskfinisher_response, AnswerItem, AnswersPayload,
    ClarifyingQuestion, TaskFinisherResult, DEFAULT_MAX_QUESTIONS, TechnicalTaskArtifact,
};

/// Console interface for the DeepSeek application
pub struct Console {
    client: DeepSeekClient,
}

impl Console {
    /// Create a new console interface with the provided DeepSeek client
    pub fn new(client: DeepSeekClient) -> Self {
        Self { client }
    }

    /// Display the welcome message and application information
    pub fn display_welcome() {
        println!(
            "{}",
            "🤖 DeepSeek JSON Chat Application".bright_blue().bold()
        );
        println!("{}", "This application sends your queries to DeepSeek and returns structured JSON responses.".blue());
        println!(
            "{}",
            "Make sure to set DEEPSEEK_API_KEY environment variable.".blue()
        );
        println!("{}", "Type '/quit' or '/exit' to stop.\n".blue());
    }

    /// Get user input from the console (async version)
    pub async fn get_user_input() -> Result<String> {
        print!("{}", "💬 Enter your question: ".bright_cyan().bold());
        io::stdout().flush().unwrap();

        let mut reader = BufReader::new(tokio::io::stdin());
        let mut input = String::new();
        reader.read_line(&mut input).await.context("Failed to read user input")?;

        Ok(input.trim().to_string())
    }

    /// Prompt the user with a custom message and return the entered line (trimmed)
    pub async fn prompt_user(prompt_text: &str) -> Result<String> {
        print!("{}", prompt_text.bright_cyan().bold());
        io::stdout().flush().unwrap();

        let mut reader = BufReader::new(tokio::io::stdin());
        let mut input = String::new();
        reader
            .read_line(&mut input)
            .await
            .context("Failed to read user input")?;

        Ok(input.trim().to_string())
    }

    /// Collect answers for clarifying questions interactively.
    /// Users enter answers one-by-one; empty input skips a question; typing '/proceed' finalizes early.
    async fn collect_answers_interactively(
        questions: &[ClarifyingQuestion],
    ) -> Result<AnswersPayload> {
        println!(
            "{}",
            "✍️ Answer the questions one-by-one. Press Enter to skip. Type '/proceed' to finalize now."
                .blue()
        );

        let mut answers: Vec<AnswerItem> = Vec::new();
        for q in questions {
            println!("\n{} {}", q.id.bright_white().bold(), q.text.white());
            if let Some(opts) = &q.options {
                if !opts.is_empty() {
                    println!("{} {:?}", "options:".white(), opts);
                }
            }

            let prompt = format!("Your answer for {}: ", q.id);
            let input = Self::prompt_user(&prompt).await?;

            if input.is_empty() {
                // Skip this question
                continue;
            }

            if Self::is_quit_command(&input) || input.eq_ignore_ascii_case("/proceed") {
                break;
            }

            answers.push(AnswerItem {
                id: q.id.clone(),
                answer: input,
            });
        }

        Ok(AnswersPayload { answers })
    }

    /// Check if the input is a quit command
    pub fn is_quit_command(input: &str) -> bool {
        input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit")
    }

    /// Display a loading message
    pub fn display_loading() {
        println!("{}", "🔄 Sending request to DeepSeek...".blue().italic());
    }

    /// Display the structured response from DeepSeek
    pub fn display_response(response: &DeepSeekResponse) {
        println!("\n{}", "📋 Structured Response:".bright_green().bold());
        println!(
            "{}",
            "┌─────────────────────────────────────────────────────────────".green()
        );
        println!(
            "{} {}",
            "│ 🏷️  Title:".green(),
            response.title.bright_white().bold()
        );
        println!(
            "{} {}",
            "│ 📝 Description:".green(),
            response.description.white()
        );
        println!("{} {}", "│ 📄 Content:".green(), response.content.white());

        if let Some(category) = &response.category {
            println!("{} {}", "│ 🏪 Category:".green(), category.white());
        }

        if let Some(timestamp) = &response.timestamp {
            println!("{} {}", "│ ⏰ Timestamp:".green(), timestamp.white());
        }

        if let Some(confidence) = response.confidence {
            println!(
                "{} {}",
                "│ 🎯 Confidence:".green(),
                format!("{:.2}", confidence).white()
            );
        }

        println!(
            "{}",
            "└─────────────────────────────────────────────────────────────\n".green()
        );
    }

    /// Display a TaskFinisher Technical Task artifact with colored sections
    pub fn display_taskfinisher_artifact(artifact: &TechnicalTaskArtifact) {
        println!("\n{}", "📦 Technical Task (Artifact):".bright_green().bold());
        println!("{}", "┌─────────────────────────────────────────────────────────────".green());

        // Basic metadata
        println!(
            "{} {}",
            "│ 🏷️  Title:".green(),
            artifact.title.bright_white().bold()
        );
        println!(
            "{} {} ({})",
            "│ 🧩 Artifact:".green(),
            artifact.artifact_name.bright_cyan(),
            format!("v{}", artifact.version).cyan().italic()
        );
        println!(
            "{} {}",
            "│ 📝 Summary:".green(),
            artifact.summary.white()
        );

        // Stakeholders
        println!("{}", "│ — Stakeholders".bright_cyan().bold());
        if artifact.stakeholders.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for stakeholder in &artifact.stakeholders {
                println!(
                    "{} {} — {}",
                    "│   •".cyan(),
                    stakeholder.role.bright_white().bold(),
                    stakeholder.description.white()
                );
            }
        }

        // Scope
        println!("{}", "│ — Scope".bright_cyan().bold());
        if !artifact.scope.in_scope.is_empty() {
            println!("{}", "│   In-scope:".green());
            for item in &artifact.scope.in_scope {
                println!("{} {}", "│     ✔".green(), item.white());
            }
        }
        if !artifact.scope.out_of_scope.is_empty() {
            println!("{}", "│   Out-of-scope:".bright_yellow());
            for item in &artifact.scope.out_of_scope {
                println!("{} {}", "│     ✖".yellow(), item.white());
            }
        }

        // Requirements
        println!("{}", "│ — Requirements".bright_cyan().bold());
        if artifact.requirements.functional.is_empty() {
            println!("{}", "│   Functional: (none)".truecolor(180, 180, 180));
        } else {
            println!("{}", "│   Functional:".green());
            for fr in &artifact.requirements.functional {
                println!(
                    "{} {} {}",
                    "│     •".green(),
                    fr.id.bright_white().bold(),
                    fr.statement.white()
                );
                if let Some(rationale) = &fr.rationale {
                    if !rationale.is_empty() {
                        println!("{} {}", "│       ↳ rationale:".truecolor(150, 150, 255), rationale.truecolor(170, 170, 255).italic());
                    }
                }
            }
        }
        if artifact.requirements.non_functional.is_empty() {
            println!("{}", "│   Non-functional: (none)".truecolor(180, 180, 180));
        } else {
            println!("{}", "│   Non-functional:".green());
            for nfr in &artifact.requirements.non_functional {
                println!(
                    "{} {} [{}] → {}",
                    "│     •".green(),
                    nfr.id.bright_white().bold(),
                    nfr.category.bright_cyan(),
                    nfr.target.white()
                );
            }
        }

        // Data integrations
        println!("{}", "│ — Data Integrations".bright_cyan().bold());
        // RPC providers
        if !artifact.data_integrations.rpc_providers.selection.is_empty() {
            println!("{} {}", "│   RPC providers:".green(), format!("{:?}", artifact.data_integrations.rpc_providers.selection).white());
        }
        if !artifact.data_integrations.rpc_providers.endpoints.is_empty() {
            println!("{}", "│   Endpoints:".green());
            for (name, value) in &artifact.data_integrations.rpc_providers.endpoints {
                println!("{} {} = {}", "│     •".green(), name.bright_white(), value.to_string().white());
            }
        }
        // Price source
        println!(
            "{} {}{}",
            "│   Price source:".green(),
            artifact.data_integrations.price_source.provider.bright_white(),
            match artifact.data_integrations.price_source.ttl_seconds {
                Some(ttl) => format!(" (ttl={}s)", ttl).truecolor(180, 180, 180),
                None => "".normal(),
            }
        );

        // Constraints
        println!("{}", "│ — Constraints".bright_cyan().bold());
        if artifact.constraints.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for c in &artifact.constraints {
                println!("{} {}", "│   •".green(), c.white());
            }
        }

        // Assumptions
        println!("{}", "│ — Assumptions".bright_cyan().bold());
        if artifact.assumptions.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for a in &artifact.assumptions {
                println!("{} {}", "│   •".green(), a.white());
            }
        }

        // Risks
        println!("{}", "│ — Risks".bright_cyan().bold());
        if artifact.risks.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for r in &artifact.risks {
                println!(
                    "{} {}: {}",
                    "│   ⚠".bright_yellow(),
                    r.id.bright_yellow().bold(),
                    r.description.white()
                );
                println!("{} {}", "│     mitigation:".green(), r.mitigation.bright_green());
            }
        }

        // Milestones
        println!("{}", "│ — Milestones".bright_cyan().bold());
        if artifact.milestones.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for m in &artifact.milestones {
                println!(
                    "{} {} — {}",
                    "│   ⏳".cyan(),
                    m.id.bright_white().bold(),
                    m.name.white()
                );
                if !m.deliverables.is_empty() {
                    println!("{}", "│     deliverables:".green());
                    for d in &m.deliverables {
                        println!("{} {}", "│       •".green(), d.white());
                    }
                }
            }
        }

        // Acceptance criteria
        println!("{}", "│ — Acceptance criteria".bright_cyan().bold());
        if artifact.acceptance_criteria.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for ac in &artifact.acceptance_criteria {
                println!("{} {}", "│   ✅".green(), ac.id.bright_white().bold());
                println!("{} {}", "│     Given:".truecolor(180, 180, 255), ac.given.white());
                println!("{} {}", "│     When:".truecolor(180, 180, 255), ac.when.white());
                println!("{} {}", "│     Then:".truecolor(180, 180, 255), ac.then.white());
            }
        }

        // Open questions
        println!("{}", "│ — Open questions".bright_cyan().bold());
        if artifact.open_questions.is_empty() {
            println!("{}", "│   (none)".truecolor(180, 180, 180));
        } else {
            for q in &artifact.open_questions {
                println!("{} {}", "│   •".bright_yellow(), q.bright_yellow());
            }
        }

        // Footer with status
        println!(
            "{} {} {} {}",
            "│ Status:".green(),
            artifact.status.bright_white().bold(),
            "End:".truecolor(180, 180, 180),
            artifact.end_token.truecolor(180, 180, 180)
        );
        println!("{}", "└─────────────────────────────────────────────────────────────".green());
    }

    /// Display an error message with context-aware messaging
    pub fn display_error(error: &anyhow::Error) {
        // Try to downcast to our custom DeepSeekError
        if let Some(deepseek_error) = error.downcast_ref::<DeepSeekError>() {
            Self::display_deepseek_error(deepseek_error);
        } else {
            // Fallback for other errors
            println!(
                "{} {}",
                "❌ Error:".bright_red().bold(),
                error.to_string().red()
            );
            println!(
                "{}",
                "Please check your configuration and try again.\n".red()
            );
        }
    }

    /// Display a DeepSeekError with appropriate styling and context
    pub fn display_deepseek_error(error: &DeepSeekError) {
        let user_message = error.user_message();

        match error {
            DeepSeekError::ServerBusy => {
                println!("{}", user_message.bright_yellow().bold());
                println!(
                    "{}",
                    "💡 Tip: Try again in a few minutes when server load is lower.".yellow()
                );
            }
            DeepSeekError::NetworkError { .. } => {
                println!("{}", user_message.bright_red().bold());
                println!(
                    "{}",
                    "💡 Tip: Check your internet connection and firewall settings.".red()
                );
            }
            DeepSeekError::Timeout { .. } => {
                println!("{}", user_message.bright_yellow().bold());
                println!(
                    "{}",
                    "💡 Tip: The server might be overloaded. Try again later.".yellow()
                );
            }
            DeepSeekError::ApiError { status, .. } => {
                println!("{}", user_message.bright_red().bold());
                match *status {
                    401 => println!(
                        "{}",
                        "💡 Tip: Check your DEEPSEEK_API_KEY environment variable.".red()
                    ),
                    403 => println!(
                        "{}",
                        "💡 Tip: Your API key may not have sufficient permissions.".red()
                    ),
                    429 => println!(
                        "{}",
                        "💡 Tip: You've hit the rate limit. Wait before trying again.".red()
                    ),
                    _ => println!(
                        "{}",
                        "💡 Tip: Check the DeepSeek API documentation for more details.".red()
                    ),
                }
            }
            DeepSeekError::ParseError { .. } => {
                println!("{}", user_message.bright_magenta().bold());
                println!(
                    "{}",
                    "💡 Tip: The server response was unexpected. Try rephrasing your query."
                        .magenta()
                );
            }
            DeepSeekError::ConfigError { .. } => {
                println!("{}", user_message.bright_red().bold());
                println!(
                    "{}",
                    "💡 Tip: Check your environment variables and configuration.".red()
                );
            }
        }
        println!(); // Add spacing
    }

    /// Display a goodbye message
    pub fn display_goodbye() {
        println!("{}", "👋 Goodbye!".bright_yellow().bold());
    }

    /// Run the main console loop
    pub async fn run(&self) -> Result<()> {
        Self::display_welcome();

        loop {
            tokio::select! {
                // Handle Ctrl+C gracefully
                _ = tokio::signal::ctrl_c() => {
                    Self::display_goodbye();
                    break;
                }
                // Handle user input
                input_result = Self::get_user_input() => {
                    let input = match input_result {
                        Ok(input) => input,
                        Err(e) => {
                            println!("Error reading input: {}", e);
                            continue;
                        }
                    };

                    if input.is_empty() {
                        continue;
                    }

                    if Self::is_quit_command(&input) {
                        Self::display_goodbye();
                        break;
                    }

                    Self::display_loading();

                    // Allow request to be cancelled by Ctrl+C
                    tokio::select! {
                        _ = tokio::signal::ctrl_c() => {
                            println!("\n{}", "⚠️ Request cancelled by user".bright_yellow());
                            Self::display_goodbye();
                            break;
                        }
                        result = self.client.send_request(&input) => {
                            match result {
                                Ok(response) => Self::display_response(&response),
                                Err(e) => Self::display_deepseek_error(&e),
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Run TaskFinisher-JSON interactive flow.
    pub async fn run_taskfinisher(&self, initial_prompt: Option<&str>, max_questions: u32) -> Result<()> {
        let max_q = if max_questions == 0 { DEFAULT_MAX_QUESTIONS } else { max_questions };
        println!("{}", "🤖 TaskFinisher-JSON Mode".bright_blue().bold());
        println!("{} {}", "Max clarifying questions:".blue(), max_q);

        let user_prompt = if let Some(p) = initial_prompt {
            p.to_string()
        } else {
            Self::prompt_user("💬 Enter your technical task request: ").await?
        };

        let system_prompt = build_system_prompt(max_q);
        let mut history: Vec<ChatMessage> = vec![
            ChatMessage { role: "system".to_string(), content: system_prompt.clone() },
            ChatMessage { role: "user".to_string(), content: format!(
                "Describe the result to collect and provide the answer accordingly. Example domain: technical specifications. User request: {}",
                user_prompt
            )},
        ];

        println!("{}", "🔄 Sending TaskFinisher request...".blue().italic());
        let mut raw = self
            .client
            .send_messages_raw(history.clone())
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let max_rounds = 5u32;
        let mut round = 1u32;

        loop {
            match parse_taskfinisher_response(&raw) {
                Ok(TaskFinisherResult::Artifact(artifact, _)) => {
                    Self::display_taskfinisher_artifact(&artifact);
                    break;
                }
                Ok(TaskFinisherResult::Clarifying(payload, _)) => {
                    // Show questions
                    println!("\n{} (round {})", "❓ Clarifying Questions:".bright_yellow().bold(), round);
                    for q in &payload.questions {
                        println!("- {} {}", q.id.bright_white().bold(), q.text.white());
                        if let Some(opts) = &q.options { println!("  options: {:?}", opts); }
                    }
                    // Show checklist
                    println!("\n{}", "🧾 Checklist:".bright_cyan().bold());
                    for item in &payload.checklist {
                        println!("- {} [{}]", item.field.white(), item.status.green());
                    }
                    println!(
                        "\n{}",
                        "💬 Enter answers one-by-one below (Enter = skip, '/proceed' = finalize now).".blue()
                    );

                    // Collect answers and send follow-up
                    let answers_payload = Self::collect_answers_interactively(&payload.questions).await?;
                    history.push(ChatMessage { role: "assistant".to_string(), content: raw });
                    history.push(ChatMessage {
                        role: "user".to_string(),
                        content: serde_json::to_string(&answers_payload).unwrap(),
                    });

                    println!("{}", "🔄 Processing answers...".blue().italic());
                    raw = self
                        .client
                        .send_messages_raw(history.clone())
                        .await
                        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

                    round += 1;
                    if round > max_rounds {
                        println!(
                            "{}",
                            "⚠️ Reached maximum clarification rounds. Showing latest assistant output.".bright_yellow()
                        );
                        println!("{}", raw);
                        break;
                    }
                }
                Err(e) => {
                    println!("{} {}", "❌ Parse error:".bright_red().bold(), e);
                    println!("{}", raw);
                    break;
                }
            }
        }

        Ok(())
    }
}

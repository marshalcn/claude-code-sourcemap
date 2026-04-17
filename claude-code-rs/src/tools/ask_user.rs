use crate::tools::Tool;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

pub struct AskUserQuestionTool;

impl AskUserQuestionTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize, Debug)]
struct QuestionOption {
    label: String,
    #[allow(dead_code)]
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
    preview: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Question {
    question: String,
    #[allow(dead_code)]
    header: String,
    options: Vec<QuestionOption>,
    #[serde(default)]
    #[serde(rename = "multiSelect")]
    #[allow(dead_code)]
    multi_select: bool,
}

#[derive(Deserialize, Debug)]
struct AskUserQuestionArgs {
    questions: Vec<Question>,
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    fn name(&self) -> std::borrow::Cow<'static, str> { "AskUserQuestion".into() }
    
    fn description(&self) -> std::borrow::Cow<'static, str> { 
        "Asks the user multiple choice questions to gather information, clarify ambiguity, understand preferences, make decisions or offer them choices.".into()
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "description": "Questions to ask the user (1-4 questions)",
                    "minItems": 1,
                    "maxItems": 4,
                    "items": {
                        "type": "object",
                        "properties": {
                            "question": {
                                "type": "string",
                                "description": "The complete question to ask the user. Should be clear, specific, and end with a question mark."
                            },
                            "header": {
                                "type": "string",
                                "description": "Very short label displayed as a chip/tag (max 12 chars). Examples: 'Auth method', 'Library'."
                            },
                            "options": {
                                "type": "array",
                                "description": "The available choices for this question. Must have 2-4 options.",
                                "minItems": 2,
                                "maxItems": 4,
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "label": {
                                            "type": "string",
                                            "description": "The display text for this option that the user will see and select."
                                        },
                                        "description": {
                                            "type": "string",
                                            "description": "Explanation of what this option means or what will happen if chosen."
                                        },
                                        "preview": {
                                            "type": "string",
                                            "description": "Optional preview content rendered when this option is focused."
                                        }
                                    },
                                    "required": ["label", "description"]
                                }
                            },
                            "multiSelect": {
                                "type": "boolean",
                                "description": "Set to true to allow the user to select multiple options instead of just one."
                            }
                        },
                        "required": ["question", "header", "options"]
                    }
                }
            },
            "required": ["questions"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let args: AskUserQuestionArgs = serde_json::from_value(args)
            .context("Failed to parse AskUserQuestionArgs")?;
            
        if args.questions.is_empty() {
            return Err(anyhow::anyhow!("At least one question must be provided"));
        }
        
        let mut result_parts = Vec::new();
        
        for (_i, q) in args.questions.iter().enumerate() {
            // Note: In a real interactive TUI application, this tool would pause 
            // execution and actually wait for the user to select an option using ratatui.
            // Since we are running asynchronously within the agent task, we would need a channel
            // to send the UI prompt and wait for the user's response.
            //
            // For now, to keep the architecture simple in this mock version,
            // we will simulate the user picking the first option.
            
            if let Some(first_opt) = q.options.first() {
                result_parts.push(format!("\"{}\"=\"{}\"", q.question, first_opt.label));
            } else {
                result_parts.push(format!("\"{}\"=\"No options provided\"", q.question));
            }
        }
        
        let answers_text = result_parts.join(", ");
        Ok(format!("User has answered your questions: {}. You can now continue with the user's answers in mind.", answers_text))
    }
}

//! Prompt template system

use std::collections::HashMap;

/// Prompt variable
pub type PromptVariable = String;

/// Prompt template with variable substitution
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    template: String,
    variables: Vec<PromptVariable>,
}

impl PromptTemplate {
    /// Create a new prompt template
    ///
    /// Variables are marked with {{variable_name}}
    pub fn new(template: String) -> Self {
        let variables = Self::extract_variables(&template);
        Self {
            template,
            variables,
        }
    }

    /// Get required variables
    pub fn variables(&self) -> &[PromptVariable] {
        &self.variables
    }

    /// Render the template with provided values
    pub fn render(&self, values: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.template.clone();

        // Check all required variables are provided
        for var in &self.variables {
            if !values.contains_key(var) {
                return Err(format!("Missing required variable: {}", var));
            }
        }

        // Substitute variables
        for (key, value) in values {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Extract variable names from template
    fn extract_variables(template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second {
                    let mut var_name = String::new();

                    // Extract variable name
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next();
                            if chars.peek() == Some(&'}') {
                                chars.next();
                                if !var_name.is_empty() && !variables.contains(&var_name) {
                                    variables.push(var_name.clone());
                                }
                                break;
                            }
                        } else {
                            var_name.push(ch);
                            chars.next();
                        }
                    }
                }
            }
        }

        variables
    }
}

/// Common prompt templates
#[allow(dead_code)]
pub struct CommonPrompts;

#[allow(dead_code)]
impl CommonPrompts {
    /// Code review prompt
    pub fn code_review() -> PromptTemplate {
        PromptTemplate::new(
            "You are reviewing code. Analyze the following code and provide feedback.\n\n\
             Code:\n```{{language}}\n{{code}}\n```\n\n\
             Focus on:\n{{focus_areas}}\n\n\
             Provide your review in JSON format."
                .to_string(),
        )
    }

    /// Decision making prompt
    pub fn decision() -> PromptTemplate {
        PromptTemplate::new(
            "You need to make a decision about: {{decision_topic}}\n\n\
             Context:\n{{context}}\n\n\
             Constraints:\n{{constraints}}\n\n\
             Consider the following alternatives and explain your reasoning:\n\
             Respond in JSON format with your decision and rationale."
                .to_string(),
        )
    }

    /// Task decomposition prompt
    pub fn decompose_task() -> PromptTemplate {
        PromptTemplate::new(
            "Break down the following task into subtasks:\n\n\
             Task: {{task}}\n\n\
             Context: {{context}}\n\n\
             Provide a list of subtasks in JSON format with dependencies."
                .to_string(),
        )
    }

    /// Risk analysis prompt
    pub fn risk_analysis() -> PromptTemplate {
        PromptTemplate::new(
            "Analyze risks for the following proposal:\n\n\
             Proposal: {{proposal}}\n\n\
             Context: {{context}}\n\n\
             Identify potential risks, their severity, and mitigation strategies.\n\
             Respond in JSON format."
                .to_string(),
        )
    }
}

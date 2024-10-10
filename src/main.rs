use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use log::{info, error};
use env_logger;
use chrono::Local;

/// Represents a single rule containing an intent, associated patterns, and possible responses.
#[derive(Deserialize, Debug)]
struct Rule {
    intent: String,
    patterns: Vec<String>,
    responses: Vec<String>,
}

/// Manages the state of a user session, including user details and conversation history.
#[derive(Serialize, Deserialize, Debug)]
struct Session {
    user_id: String,
    user_name: String,
    last_intent: Option<String>,
    conversation_history: Vec<String>,
}

impl Session {
    /// Initializes a new session with the given user ID and name.
    fn new(user_id: &str, user_name: &str) -> Self {
        Session {
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            last_intent: None,
            conversation_history: Vec::new(),
        }
    }
}

/// Entry point of the chatbot application.
fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger using environment variables.
    env_logger::init();
    info!("Logger initialized.");

    // Define the path to the rules JSON file.
    let rules_path = "rules_with_patterns.json";

    // Load chatbot rules from the JSON file.
    let mut rules = load_rules_from_json(rules_path)?;

    // Initialize a user session with default ID and name.
    let user_id = "user123";
    let user_name = "User";
    let mut session = Session::new(user_id, user_name);

    // Welcome message to the user.
    println!("Welcome to Rust Chatbot! Type 'exit' to quit.");

    // Start the main interaction loop.
    loop {
        // Display the prompt for user input.
        print!("You: ");
        io::stdout().flush()?; // Ensure the prompt is displayed immediately.

        // Read the user's input from standard input.
        let mut input_text = String::new();
        io::stdin().read_line(&mut input_text)?;
        let input_text = input_text.trim(); // Remove trailing newline and whitespace.

        // Handle the 'exit' command to terminate the chatbot.
        if input_text.eq_ignore_ascii_case("exit") {
            println!("Chatbot: Goodbye!");
            info!("User exited the chat.");
            break;
        }

        // Handle the 'reload rules' command to update chatbot rules dynamically.
        else if input_text.eq_ignore_ascii_case("reload rules") {
            match reload_rules(rules_path) {
                Ok(new_rules) => {
                    rules = new_rules;
                    println!("Chatbot: Rules reloaded successfully.");
                    info!("Rules reloaded.");
                },
                Err(e) => {
                    println!("Chatbot: Failed to reload rules: {}", e);
                    error!("Failed to reload rules: {}", e);
                }
            }
            continue; // Restart the loop after reloading rules.
        }

        // Handle the 'list intents' command to display all available chatbot intents.
        else if input_text.eq_ignore_ascii_case("list intents") {
            list_intents(&rules);
            continue; // Restart the loop after listing intents.
        }

        // Process the user's input to determine the appropriate response.
        let cleaned_input = clean_input(input_text); // Normalize the input.
        let rule = match_rule(&cleaned_input, &rules); // Attempt to match an intent.

        // Generate the chatbot's response based on the matched intent.
        let response = if let Some(rule) = rule {
            session.last_intent = Some(rule.intent.clone()); // Update session with the last intent.
            generate_response(rule, &session) // Generate a dynamic response.
        } else {
            "I'm sorry, I didn't understand that. Could you please rephrase?".to_string()
        };

        // Display the chatbot's response to the user.
        println!("Chatbot: {}", response);

        // Log the interaction details for monitoring and debugging.
        info!("User input: '{}', Response: '{}'", input_text, response);
    }

    Ok(())
}

/// Loads chatbot rules from a specified JSON file.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the JSON file.
///
/// # Returns
///
/// * `Result<Vec<Rule>, Box<dyn Error>>` - A vector of `Rule` structs or an error.
fn load_rules_from_json(file_path: &str) -> Result<Vec<Rule>, Box<dyn Error>> {
    let data = fs::read_to_string(file_path)?; // Read the entire file into a string.
    let rules: Vec<Rule> = serde_json::from_str(&data)?; // Deserialize JSON into a vector of Rule structs.
    Ok(rules)
}

/// Reloads chatbot rules from the JSON file.
///
/// # Arguments
///
/// * `rules_path` - A string slice that holds the path to the JSON file.
///
/// # Returns
///
/// * `Result<Vec<Rule>, Box<dyn Error>>` - A vector of updated `Rule` structs or an error.
fn reload_rules(rules_path: &str) -> Result<Vec<Rule>, Box<dyn Error>> {
    load_rules_from_json(rules_path) // Reuse the load function to fetch updated rules.
}

/// Cleans and normalizes user input for intent matching.
///
/// # Arguments
///
/// * `input` - A string slice containing the user's raw input.
///
/// # Returns
///
/// * `String` - A lowercase, trimmed version of the input.
fn clean_input(input: &str) -> String {
    input.to_lowercase().trim().to_string()
}

/// Attempts to match the user's input to a defined intent.
///
/// # Arguments
///
/// * `user_input` - A string slice containing the normalized user input.
/// * `rules` - A slice of `Rule` structs to match against.
///
/// # Returns
///
/// * `Option<&Rule>` - A reference to the matched `Rule` or `None` if no match is found.
fn match_rule<'a>(user_input: &str, rules: &'a [Rule]) -> Option<&'a Rule> {
    for rule in rules {
        for pattern in &rule.patterns {
            if user_input.contains(&pattern.to_lowercase()) {
                return Some(rule); // Return the first matching rule.
            }
        }
    }
    None // No matching intent found.
}

/// Generates a dynamic response based on the matched rule and session data.
///
/// # Arguments
///
/// * `rule` - A reference to the matched `Rule` struct.
/// * `session` - A reference to the current `Session` struct.
///
/// # Returns
///
/// * `String` - The generated response with placeholders replaced.
fn generate_response(rule: &Rule, session: &Session) -> String {
    let mut response = rule.responses[0].clone(); // Start with the first response template.
    response = response.replace("{name}", &session.user_name); // Replace `{name}` with the user's name.
    response = response.replace("{time}", &Local::now().format("%I:%M %p").to_string()); // Replace `{time}` with the current time.
    response // Return the finalized response.
}

/// Lists all available intents defined in the chatbot's rules.
///
/// # Arguments
///
/// * `rules` - A slice of `Rule` structs representing the chatbot's intents.
fn list_intents(rules: &[Rule]) {
    println!("Chatbot: Available intents:");
    for rule in rules {
        println!("- {}", rule.intent); // Display each intent.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_input() {
        let input = "  Hello World!  ";
        let expected = "hello world!".to_string();
        assert_eq!(clean_input(input), expected);
    }

    #[test]
    fn test_match_rule_found() {
        let rules = vec![
            Rule {
                intent: "greet".to_string(),
                patterns: vec!["hello".to_string(), "hi".to_string()],
                responses: vec!["Hello, {name}! How can I assist you today?".to_string()],
            },
            Rule {
                intent: "farewell".to_string(),
                patterns: vec!["bye".to_string(), "goodbye".to_string()],
                responses: vec!["Goodbye, {name}! Have a great day!".to_string()],
            },
        ];

        let input = "hi there";
        let matched_rule = match_rule(&clean_input(input), &rules);
        assert!(matched_rule.is_some());
        assert_eq!(matched_rule.unwrap().intent, "greet");
    }

    #[test]
    fn test_match_rule_not_found() {
        let rules = vec![
            Rule {
                intent: "greet".to_string(),
                patterns: vec!["hello".to_string(), "hi".to_string()],
                responses: vec!["Hello, {name}! How can I assist you today?".to_string()],
            },
        ];

        let input = "unknown command";
        let matched_rule = match_rule(&clean_input(input), &rules);
        assert!(matched_rule.is_none());
    }

    #[test]
    fn test_generate_response() {
        let rule = Rule {
            intent: "greet".to_string(),
            patterns: vec!["hello".to_string()],
            responses: vec!["Hello, {name}! It's {time}.".to_string()],
        };
        let session = Session {
            user_id: "user123".to_string(),
            user_name: "Alice".to_string(),
            last_intent: Some("greet".to_string()),
            conversation_history: vec![],
        };
        let response = generate_response(&rule, &session);
        assert!(response.contains("Alice"));
        assert!(response.contains("It’s"));
    }
}

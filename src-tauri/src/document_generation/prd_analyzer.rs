use super::types::*;
use std::collections::HashSet;

/// Analyze differences between two PRD versions
pub fn analyze_changes(previous: &PRDContent, current: &PRDContent) -> PRDChange {
    let timestamp = chrono::Utc::now().to_rfc3339();

    PRDChange {
        from_version: 0, // Will be set by caller
        to_version: 0,   // Will be set by caller
        timestamp,
        added_user_stories: find_added_items(
            &previous.user_stories.iter().map(|s| s.id.clone()).collect(),
            &current.user_stories.iter().map(|s| s.id.clone()).collect(),
        ),
        modified_user_stories: find_modified_user_stories(
            &previous.user_stories,
            &current.user_stories,
        ),
        removed_user_stories: find_removed_items(
            &previous.user_stories.iter().map(|s| s.id.clone()).collect(),
            &current.user_stories.iter().map(|s| s.id.clone()).collect(),
        ),
        added_requirements: find_added_items(
            &get_all_requirement_ids(previous),
            &get_all_requirement_ids(current),
        ),
        modified_requirements: find_modified_requirements(previous, current),
        removed_requirements: find_removed_items(
            &get_all_requirement_ids(previous),
            &get_all_requirement_ids(current),
        ),
        resolved_questions: find_resolved_questions(
            &previous.open_questions,
            &current.open_questions,
        ),
        new_questions: find_added_items(
            &previous
                .open_questions
                .iter()
                .map(|q| q.id.clone())
                .collect(),
            &current
                .open_questions
                .iter()
                .map(|q| q.id.clone())
                .collect(),
        ),
        added_technical_requirements: find_added_items(
            &previous
                .technical_requirements
                .iter()
                .map(|t| t.id.clone())
                .collect(),
            &current
                .technical_requirements
                .iter()
                .map(|t| t.id.clone())
                .collect(),
        ),
        added_risks: find_added_items(
            &previous.risks.iter().map(|r| r.id.clone()).collect(),
            &current.risks.iter().map(|r| r.id.clone()).collect(),
        ),
        added_dependencies: find_added_items(
            &previous.dependencies.iter().map(|d| d.id.clone()).collect(),
            &current.dependencies.iter().map(|d| d.id.clone()).collect(),
        ),
    }
}

fn find_added_items(previous_ids: &HashSet<String>, current_ids: &HashSet<String>) -> Vec<String> {
    current_ids.difference(previous_ids).cloned().collect()
}

fn find_removed_items(
    previous_ids: &HashSet<String>,
    current_ids: &HashSet<String>,
) -> Vec<String> {
    previous_ids.difference(current_ids).cloned().collect()
}

fn find_modified_user_stories(previous: &[UserStory], current: &[UserStory]) -> Vec<String> {
    let mut modified = Vec::new();

    for curr_story in current {
        if let Some(prev_story) = previous.iter().find(|s| s.id == curr_story.id) {
            if story_has_changes(prev_story, curr_story) {
                modified.push(curr_story.id.clone());
            }
        }
    }

    modified
}

fn story_has_changes(prev: &UserStory, curr: &UserStory) -> bool {
    prev.persona != curr.persona
        || prev.want != curr.want
        || prev.so_that != curr.so_that
        || prev.priority != curr.priority
        || prev.status != curr.status
}

fn get_all_requirement_ids(content: &PRDContent) -> HashSet<String> {
    let mut ids = HashSet::new();

    for req in &content.functional_requirements {
        ids.insert(req.id.clone());
    }

    for req in &content.non_functional_requirements {
        ids.insert(req.id.clone());
    }

    ids
}

fn find_modified_requirements(previous: &PRDContent, current: &PRDContent) -> Vec<String> {
    let mut modified = Vec::new();

    // Check functional requirements
    for curr_req in &current.functional_requirements {
        if let Some(prev_req) = previous
            .functional_requirements
            .iter()
            .find(|r| r.id == curr_req.id)
        {
            if requirement_has_changes(prev_req, curr_req) {
                modified.push(curr_req.id.clone());
            }
        }
    }

    // Check non-functional requirements
    for curr_req in &current.non_functional_requirements {
        if let Some(prev_req) = previous
            .non_functional_requirements
            .iter()
            .find(|r| r.id == curr_req.id)
        {
            if requirement_has_changes(prev_req, curr_req) {
                modified.push(curr_req.id.clone());
            }
        }
    }

    modified
}

fn requirement_has_changes(prev: &Requirement, curr: &Requirement) -> bool {
    prev.title != curr.title
        || prev.description != curr.description
        || prev.priority != curr.priority
        || prev.status != curr.status
}

fn find_resolved_questions(previous: &[Question], current: &[Question]) -> Vec<String> {
    let mut resolved = Vec::new();

    for curr_question in current {
        if let Some(prev_question) = previous.iter().find(|q| q.id == curr_question.id) {
            if !prev_question.resolved && curr_question.resolved {
                resolved.push(curr_question.id.clone());
            }
        }
    }

    resolved
}

/// Generate a summary of changes for display
pub fn summarize_changes(change: &PRDChange) -> String {
    let mut summary = Vec::new();

    if !change.added_user_stories.is_empty() {
        summary.push(format!(
            "✅ {} new user stories",
            change.added_user_stories.len()
        ));
    }

    if !change.modified_user_stories.is_empty() {
        summary.push(format!(
            "🔄 {} modified user stories",
            change.modified_user_stories.len()
        ));
    }

    if !change.added_requirements.is_empty() {
        summary.push(format!(
            "✅ {} new requirements",
            change.added_requirements.len()
        ));
    }

    if !change.modified_requirements.is_empty() {
        summary.push(format!(
            "🔄 {} modified requirements",
            change.modified_requirements.len()
        ));
    }

    if !change.resolved_questions.is_empty() {
        summary.push(format!(
            "✔️ {} resolved questions",
            change.resolved_questions.len()
        ));
    }

    if !change.new_questions.is_empty() {
        summary.push(format!("❓ {} new questions", change.new_questions.len()));
    }

    if !change.added_technical_requirements.is_empty() {
        summary.push(format!(
            "🔧 {} technical requirements",
            change.added_technical_requirements.len()
        ));
    }

    if !change.added_risks.is_empty() {
        summary.push(format!("⚠️ {} new risks", change.added_risks.len()));
    }

    if !change.added_dependencies.is_empty() {
        summary.push(format!(
            "🔗 {} new dependencies",
            change.added_dependencies.len()
        ));
    }

    if summary.is_empty() {
        "No significant changes".to_string()
    } else {
        summary.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_added_items() {
        let previous: HashSet<String> = vec!["US-001".to_string(), "US-002".to_string()]
            .into_iter()
            .collect();
        let current: HashSet<String> = vec![
            "US-001".to_string(),
            "US-002".to_string(),
            "US-003".to_string(),
        ]
        .into_iter()
        .collect();

        let added = find_added_items(&previous, &current);
        assert_eq!(added.len(), 1);
        assert!(added.contains(&"US-003".to_string()));
    }

    #[test]
    fn test_find_removed_items() {
        let previous: HashSet<String> = vec![
            "US-001".to_string(),
            "US-002".to_string(),
            "US-003".to_string(),
        ]
        .into_iter()
        .collect();
        let current: HashSet<String> = vec!["US-001".to_string(), "US-002".to_string()]
            .into_iter()
            .collect();

        let removed = find_removed_items(&previous, &current);
        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&"US-003".to_string()));
    }

    #[test]
    fn test_story_has_changes() {
        let story1 = UserStory {
            id: "US-001".to_string(),
            persona: "user".to_string(),
            want: "login".to_string(),
            so_that: "access my account".to_string(),
            priority: "high".to_string(),
            status: "planned".to_string(),
            mentioned_at: vec![1],
        };

        let story2_same = story1.clone();
        let mut story2_different = story1.clone();
        story2_different.priority = "medium".to_string();

        assert!(!story_has_changes(&story1, &story2_same));
        assert!(story_has_changes(&story1, &story2_different));
    }

    #[test]
    fn test_find_resolved_questions() {
        let previous = vec![
            Question {
                id: "Q-001".to_string(),
                question: "What database?".to_string(),
                context: "Storage".to_string(),
                asked_at: 5,
                resolved: false,
                resolution: None,
            },
            Question {
                id: "Q-002".to_string(),
                question: "Which framework?".to_string(),
                context: "Frontend".to_string(),
                asked_at: 10,
                resolved: false,
                resolution: None,
            },
        ];

        let mut current = previous.clone();
        current[0].resolved = true;
        current[0].resolution = Some("PostgreSQL".to_string());

        let resolved = find_resolved_questions(&previous, &current);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], "Q-001");
    }
}

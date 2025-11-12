use super::types::*;

/// Render PRDContent to markdown string
pub fn render_prd_markdown(
    content: &PRDContent,
    version: &PRDVersion,
    meeting_name: &str,
    project_name: &str,
) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# Product Requirements Document\n"));
    md.push_str(&format!("**Project**: {}\n", project_name));
    md.push_str(&format!("**Meeting**: {}\n", meeting_name));
    md.push_str(&format!("**Version**: {}\n", version.version));
    md.push_str(&format!("**Generated**: {}\n", version.generated_at));
    md.push_str(&format!(
        "**Segment Range**: {}-{} of {}\n",
        version.segment_range.0, version.segment_range.1, version.total_segments
    ));
    md.push_str(&format!("**Confidence**: {:.0}%\n", version.confidence * 100.0));
    md.push_str("\n---\n\n");

    // Executive Summary
    md.push_str("## Executive Summary\n\n");
    if content.executive_summary.is_empty() {
        md.push_str("*No summary generated yet.*\n\n");
    } else {
        md.push_str(&content.executive_summary);
        md.push_str("\n\n");
    }
    md.push_str("---\n\n");

    // User Stories
    md.push_str("## User Stories\n\n");
    if content.user_stories.is_empty() {
        md.push_str("*No user stories identified yet.*\n\n");
    } else {
        // Group by priority
        for priority in &["high", "medium", "low"] {
            let stories: Vec<_> = content
                .user_stories
                .iter()
                .filter(|s| s.priority.to_lowercase() == *priority)
                .collect();

            if !stories.is_empty() {
                md.push_str(&format!(
                    "### Priority: {}\n\n",
                    capitalize(priority)
                ));

                for story in stories {
                    md.push_str(&format!("**{}**: ", story.id));
                    md.push_str(&format!(
                        "As a {}, I want to {}, so that {}\n",
                        story.persona, story.want, story.so_that
                    ));
                    md.push_str(&format!("- **Status**: {}\n", capitalize(&story.status)));
                    if !story.mentioned_at.is_empty() {
                        md.push_str(&format!(
                            "- **Mentioned**: Segments {}\n",
                            format_segment_list(&story.mentioned_at)
                        ));
                    }
                    md.push_str("\n");
                }
            }
        }
    }
    md.push_str("---\n\n");

    // Functional Requirements
    md.push_str("## Functional Requirements\n\n");
    if content.functional_requirements.is_empty() {
        md.push_str("*No functional requirements defined yet.*\n\n");
    } else {
        for req in &content.functional_requirements {
            md.push_str(&format!("### {}: {}\n", req.id, req.title));
            md.push_str(&format!("**Priority**: {}\n", capitalize(&req.priority)));
            md.push_str(&format!("**Status**: {}\n", capitalize(&req.status)));
            md.push_str(&format!("**Description**: {}\n\n", req.description));

            // Find acceptance criteria for this requirement
            let criteria: Vec<_> = content
                .acceptance_criteria
                .iter()
                .filter(|c| c.requirement_id == req.id)
                .collect();

            if !criteria.is_empty() {
                md.push_str("**Acceptance Criteria**:\n");
                for criterion in criteria {
                    md.push_str(&format!(
                        "- [ ] {} {}\n",
                        criterion.description,
                        if criterion.testable { "✓" } else { "" }
                    ));
                }
                md.push_str("\n");
            }

            if !req.mentioned_at.is_empty() {
                md.push_str(&format!(
                    "**Mentioned**: Segments {}\n\n",
                    format_segment_list(&req.mentioned_at)
                ));
            }
        }
    }
    md.push_str("---\n\n");

    // Non-Functional Requirements
    md.push_str("## Non-Functional Requirements\n\n");
    if content.non_functional_requirements.is_empty() {
        md.push_str("*No non-functional requirements defined yet.*\n\n");
    } else {
        // Group by category if available
        let categories = extract_categories(&content.non_functional_requirements);

        for category in categories {
            let reqs: Vec<_> = content
                .non_functional_requirements
                .iter()
                .filter(|r| {
                    r.category.as_ref().map(|c| c.to_lowercase())
                        == Some(category.to_lowercase())
                })
                .collect();

            if !reqs.is_empty() {
                md.push_str(&format!("### {} Requirements\n\n", capitalize(&category)));

                for req in reqs {
                    md.push_str(&format!("**{}: {}**\n", req.id, req.title));
                    md.push_str(&format!("*Priority*: {}\n", capitalize(&req.priority)));
                    md.push_str(&format!("*Status*: {}\n\n", capitalize(&req.status)));
                    md.push_str(&format!("{}\n\n", req.description));
                }
            }
        }

        // Handle requirements without category
        let uncategorized: Vec<_> = content
            .non_functional_requirements
            .iter()
            .filter(|r| r.category.is_none())
            .collect();

        if !uncategorized.is_empty() {
            md.push_str("### Other Requirements\n\n");
            for req in uncategorized {
                md.push_str(&format!("**{}: {}**\n", req.id, req.title));
                md.push_str(&format!("*Priority*: {}\n", capitalize(&req.priority)));
                md.push_str(&format!("{}\n\n", req.description));
            }
        }
    }
    md.push_str("---\n\n");

    // Technical Requirements
    md.push_str("## Technical Requirements\n\n");
    if content.technical_requirements.is_empty() {
        md.push_str("*No technical requirements defined yet.*\n\n");
    } else {
        // Group by category
        let categories = extract_tech_categories(&content.technical_requirements);

        for category in categories {
            let reqs: Vec<_> = content
                .technical_requirements
                .iter()
                .filter(|r| r.category.to_lowercase() == category.to_lowercase())
                .collect();

            if !reqs.is_empty() {
                md.push_str(&format!("### {}\n\n", capitalize(&category)));

                for req in reqs {
                    md.push_str(&format!("**{}**: {}\n", req.title, req.description));
                    md.push_str(&format!("*Rationale*: {}\n", req.rationale));

                    if !req.alternatives_considered.is_empty() {
                        md.push_str(&format!(
                            "*Alternatives Considered*: {}\n",
                            req.alternatives_considered.join(", ")
                        ));
                    }

                    if !req.mentioned_at.is_empty() {
                        md.push_str(&format!(
                            "*Mentioned*: Segments {}\n",
                            format_segment_list(&req.mentioned_at)
                        ));
                    }
                    md.push_str("\n");
                }
            }
        }
    }
    md.push_str("---\n\n");

    // Dependencies
    md.push_str("## Dependencies\n\n");
    if content.dependencies.is_empty() {
        md.push_str("*No dependencies identified yet.*\n\n");
    } else {
        // Group by type
        for dep_type in &["internal", "external", "third_party"] {
            let deps: Vec<_> = content
                .dependencies
                .iter()
                .filter(|d| d.type_.to_lowercase() == *dep_type)
                .collect();

            if !deps.is_empty() {
                md.push_str(&format!("### {} Dependencies\n\n", capitalize(dep_type)));

                for dep in deps {
                    md.push_str(&format!(
                        "- **{}**: {} {} {}\n",
                        dep.id,
                        dep.name,
                        dep.description,
                        if dep.blocking { "⚠️ (Blocking)" } else { "" }
                    ));
                }
                md.push_str("\n");
            }
        }
    }
    md.push_str("---\n\n");

    // Risks & Mitigations
    md.push_str("## Risks & Mitigations\n\n");
    if content.risks.is_empty() {
        md.push_str("*No risks identified yet.*\n\n");
    } else {
        md.push_str("| Risk ID | Description | Severity | Likelihood | Mitigation |\n");
        md.push_str("|---------|-------------|----------|------------|------------|\n");

        for risk in &content.risks {
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                risk.id,
                risk.description,
                capitalize(&risk.severity),
                capitalize(&risk.likelihood),
                risk.mitigation
            ));
        }
        md.push_str("\n");
    }
    md.push_str("---\n\n");

    // Timeline & Milestones
    md.push_str("## Timeline & Milestones\n\n");
    if content.timeline.is_empty() {
        md.push_str("*No timeline defined yet.*\n\n");
    } else {
        for milestone in &content.timeline {
            md.push_str(&format!("### {}: {}\n", milestone.id, milestone.title));
            md.push_str(&format!("{}\n\n", milestone.description));

            if let Some(target_date) = &milestone.target_date {
                md.push_str(&format!("**Target Date**: {}\n", target_date));
            }

            if !milestone.deliverables.is_empty() {
                md.push_str("**Deliverables**:\n");
                for deliverable in &milestone.deliverables {
                    md.push_str(&format!("- {}\n", deliverable));
                }
            }
            md.push_str("\n");
        }
    }
    md.push_str("---\n\n");

    // Open Questions
    md.push_str("## Open Questions\n\n");
    if content.open_questions.is_empty() {
        md.push_str("*No open questions.*\n\n");
    } else {
        for question in &content.open_questions {
            md.push_str(&format!("### {}: {}\n", question.id, question.question));
            md.push_str(&format!("**Context**: {}\n", question.context));
            md.push_str(&format!("**Asked**: Segment #{}\n", question.asked_at));
            md.push_str(&format!(
                "**Status**: {}\n",
                if question.resolved {
                    "Resolved ✅"
                } else {
                    "Unresolved ❓"
                }
            ));

            if let Some(resolution) = &question.resolution {
                md.push_str(&format!("**Resolution**: {}\n", resolution));
            }
            md.push_str("\n");
        }
    }

    md
}

/// Render changelog to markdown
pub fn render_changelog_markdown(changes: &[PRDChange]) -> String {
    let mut md = String::new();

    md.push_str("# PRD Changelog\n\n");

    for change in changes {
        md.push_str(&format!(
            "## Version {} → {} ({})\n\n",
            change.from_version, change.to_version, change.timestamp
        ));

        // User Stories
        if !change.added_user_stories.is_empty() {
            md.push_str(&format!(
                "**✅ Added User Stories ({})**: {}\n\n",
                change.added_user_stories.len(),
                change.added_user_stories.join(", ")
            ));
        }
        if !change.modified_user_stories.is_empty() {
            md.push_str(&format!(
                "**🔄 Modified User Stories ({})**: {}\n\n",
                change.modified_user_stories.len(),
                change.modified_user_stories.join(", ")
            ));
        }
        if !change.removed_user_stories.is_empty() {
            md.push_str(&format!(
                "**❌ Removed User Stories ({})**: {}\n\n",
                change.removed_user_stories.len(),
                change.removed_user_stories.join(", ")
            ));
        }

        // Requirements
        if !change.added_requirements.is_empty() {
            md.push_str(&format!(
                "**✅ Added Requirements ({})**: {}\n\n",
                change.added_requirements.len(),
                change.added_requirements.join(", ")
            ));
        }
        if !change.modified_requirements.is_empty() {
            md.push_str(&format!(
                "**🔄 Modified Requirements ({})**: {}\n\n",
                change.modified_requirements.len(),
                change.modified_requirements.join(", ")
            ));
        }
        if !change.removed_requirements.is_empty() {
            md.push_str(&format!(
                "**❌ Removed Requirements ({})**: {}\n\n",
                change.removed_requirements.len(),
                change.removed_requirements.join(", ")
            ));
        }

        // Technical Requirements
        if !change.added_technical_requirements.is_empty() {
            md.push_str(&format!(
                "**✅ Added Technical Requirements ({})**: {}\n\n",
                change.added_technical_requirements.len(),
                change.added_technical_requirements.join(", ")
            ));
        }

        // Risks & Dependencies
        if !change.added_risks.is_empty() {
            md.push_str(&format!(
                "**⚠️ Added Risks ({})**: {}\n\n",
                change.added_risks.len(),
                change.added_risks.join(", ")
            ));
        }
        if !change.added_dependencies.is_empty() {
            md.push_str(&format!(
                "**🔗 Added Dependencies ({})**: {}\n\n",
                change.added_dependencies.len(),
                change.added_dependencies.join(", ")
            ));
        }

        // Questions
        if !change.resolved_questions.is_empty() {
            md.push_str(&format!(
                "**✔️ Resolved Questions ({})**: {}\n\n",
                change.resolved_questions.len(),
                change.resolved_questions.join(", ")
            ));
        }
        if !change.new_questions.is_empty() {
            md.push_str(&format!(
                "**❓ New Questions ({})**: {}\n\n",
                change.new_questions.len(),
                change.new_questions.join(", ")
            ));
        }

        md.push_str("---\n\n");
    }

    md
}

// Helper functions

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

fn format_segment_list(segments: &[usize]) -> String {
    if segments.is_empty() {
        return "N/A".to_string();
    }

    if segments.len() <= 3 {
        segments
            .iter()
            .map(|s| format!("#{}", s))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        format!(
            "#{}-#{} ({} total)",
            segments.first().unwrap(),
            segments.last().unwrap(),
            segments.len()
        )
    }
}

fn extract_categories(requirements: &[Requirement]) -> Vec<String> {
    let mut categories: Vec<String> = requirements
        .iter()
        .filter_map(|r| r.category.clone())
        .collect();

    categories.sort();
    categories.dedup();
    categories
}

fn extract_tech_categories(requirements: &[TechnicalRequirement]) -> Vec<String> {
    let mut categories: Vec<String> = requirements.iter().map(|r| r.category.clone()).collect();

    categories.sort();
    categories.dedup();
    categories
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("hello"), "Hello");
        assert_eq!(capitalize("HELLO"), "HELLO");
        assert_eq!(capitalize(""), "");
    }

    #[test]
    fn test_format_segment_list() {
        assert_eq!(format_segment_list(&[]), "N/A");
        assert_eq!(format_segment_list(&[1]), "#1");
        assert_eq!(format_segment_list(&[1, 2, 3]), "#1, #2, #3");
        assert_eq!(format_segment_list(&[1, 2, 3, 4, 5]), "#1-#5 (5 total)");
    }

    #[test]
    fn test_render_empty_prd() {
        let content = PRDContent::default();
        let version = PRDVersion {
            version: 1,
            generated_at: "2025-11-11T12:00:00Z".to_string(),
            segment_range: (0, 10),
            total_segments: 10,
            file_path: "test.md".to_string(),
            version_type: "initial".to_string(),
            confidence: 0.85,
            word_count: 0,
        };

        let md = render_prd_markdown(&content, &version, "Test Meeting", "Test Project");

        assert!(md.contains("# Product Requirements Document"));
        assert!(md.contains("**Project**: Test Project"));
        assert!(md.contains("**Version**: 1"));
        assert!(md.contains("*No user stories identified yet.*"));
    }
}

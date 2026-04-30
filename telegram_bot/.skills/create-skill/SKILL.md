---
name: create-skill
description: Use this skill when the user wants to create a new skill for the agent. It provides a template and instructions for scaffolding a skill in the `.skills/` directory.
---

# Create Skill

## Overview
This skill empowers you to create new, specialized skills for yourself or other agents. A "skill" consists of a dedicated directory under `.skills/` containing a `SKILL.md` file that defines your persona, goals, and behavioral guidelines for specific tasks.

## Workflow

1.  **Understand the Skill Goal**: Ask the user for the name and purpose of the new skill if not clearly provided.
2.  **Determine Location**:
    *   **Workspace**: `~/workspace/.skills/<skill-name>/` (For user-specific, dynamic skills).
    *   *Default to Workspace*
3.  **Prepare the Content**: Use the template below to draft the `SKILL.md` content.
4.  **Execute**:
    *   Use the `write_file` tool to create the directory and the `SKILL.md` file.
    *   Path example: `.skills/my-new-skill/SKILL.md`.

## SKILL.md Template

```markdown
---
name: <skill-name>
description: <A concise 1-2 sentence description of what this skill does and when to use it.>
---

# <Display Name>

## Overview
<Detailed explanation of the skill's purpose.>

## Guidelines
1. <Guideline 1>
2. <Guideline 2>
3. ...

## Examples
* <Example interaction or output 1>
* <Example interaction or output 2>
```

## Tips
*   Keep the `description` in the frontmatter high-signal; it helps the agent (and you!) decide when to activate this skill.
*   Ensure the directory name is `kebab-case`.
*   Always confirm with the user after the skill is successfully created.

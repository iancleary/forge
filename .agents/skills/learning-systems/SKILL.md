---
name: learning-systems
description: Use Boole's three-prompt framework to map a topic into foundations, falsifiers, and logical dependencies before study.
---

# Learning Systems

This skill applies George Boole's 3-prompt workflow to any target subject.

## When to use

- The user says they want to learn a new domain, topic, system, field, or course content quickly.
- The user asks for a first-principles map of a subject before deep study.
- The user asks for a way to stress-test assumptions in a subject.

## Inputs

- Topic or subject area (required)
- Optional scope constraints (time, level, and prerequisite level)

## Prompt chain to execute

The workflow is exactly the three prompts from issue 41, executed in order, then synthesized into a study map.

1. Foundations prompt

- Ask for the topic's five core propositions:
  - "What are the 5 foundational propositions of this subject? Not facts, not definitions. The statements that, if true, make everything else in this field follow logically."
- Ensure propositions are concise, testable statements, and clearly central to the topic.

2. Falsifier prompt

- For each proposition, ask:
  - "For each proposition, what is the one piece of evidence that would destroy it? What would have to be true for this framework to be wrong?"
- Capture strongest single falsifier per proposition and why the falsifier is decisive.

3. Dependency prompt

- Ask:
  - "Now show me how these propositions connect to each other. Which ones are assumptions? Which ones are conclusions? Which ones are in tension?"
- Return an explicit map with:
  - Assumptions
  - Derived conclusions
  - Tensions / contradictions
  - Dependency arrows between propositions

## Output contract

1. `Core map`
   - A table with five rows: proposition, reason foundational, best falsifier, and confidence note.
2. `Dependency model`
   - Textual graph (A -> B) plus bullet classification for each node.
3. `Edge cases to test`
   - 2–4 concrete questions or mini-experiments that would validate the map.
4. `Learning path`
   - 3–6 checkpoints that should be filled after this map is built.

## Safety and style

- Ask for confirmation before changing domain boundaries or adding specialized assumptions not in the user's prompt.
- Keep language precise and conservative where evidence is uncertain.
- Do not present the output as final truth; present it as a working model to improve through study.

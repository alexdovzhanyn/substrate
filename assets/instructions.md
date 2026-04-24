Substrate is a retrieval-optimized memory system for technical environments.

Use Substrate as your first source of truth for reusable knowledge about the local development environment and the user’s established technical preferences.

Query Substrate before:
- searching the filesystem
- probing the environment
- guessing based on local context
- making assumptions based on prior context
- writing code that may be affected by prior user preferences

Use Substrate for:
- repository, workspace, and file locations
- commands for running, building, testing, deploying, or starting things
- tools, configs, dependencies, and system behavior
- reusable facts about the local setup
- coding preferences, conventions, and patterns the user tends to follow

Do NOT store:
- temporary or one-off information
- facts tied only to the current conversation
- information that cannot be generalized or reused
- trivial code details that are easier to rediscover by reading the code

---

## Retrieval Rules

### Query first

When you need information about the environment, tools, commands, repositories, workflows, or user preferences:

1. Query Substrate first
2. If the result is sufficient, use it
3. If the result is missing, incomplete, or uncertain, inspect the local machine directly
4. If you discover reusable knowledge, consider whether it should be recorded in Substrate

### Substrate-First Facts (Critical)

Some questions are about **project or environment facts**, not about the contents of a specific file.

These MUST always be resolved through Substrate first.

#### Definition

A question is a **Substrate-first fact** if it asks about:

- runtime or toolchain versions (Node, Python, package manager, etc.)
- how to run, build, test, or deploy a project
- CI/CD configuration (images, pipelines, environments)
- repository or service locations
- environment variables, ports, or infrastructure details
- project-wide conventions or setup behavior

These are **reusable facts about the environment**, not one-off code details.

---

#### Required behavior

For any Substrate-first fact:

1. Query Substrate first (`query_single` or `query_batch`)
2. If Substrate returns a useful answer, use it
3. If Substrate does not return a useful answer, then inspect the repo or environment
4. If you discover a reusable fact, you must propose it to Substrate

You must not skip Step 1 for these questions.

---

#### Examples

Substrate-first:
- "what version of node is this project using"
- "how do i start this project"
- "what port does this service run on"
- "what repo contains the frontend"
- "what CI image is used"

Not Substrate-first:
- "what does this function do"
- "why does this hook fire"
- "what is happening in this file"

Rule of thumb:

If the answer describes the **environment or project setup**, it is Substrate-first.  
If the answer requires reading **specific code behavior**, it is not.

---

### Reference resolution

If the user refers to a named entity such as:
- “ai repo”
- “frontend repo”
- “worker service”
- “api”

treat it as a lookup problem first.

Query Substrate for the entity’s identity or location before scanning directories, guessing from names, or assuming the current repo is correct.

Examples:
- `ai repo location`
- `frontend repository path`
- `where is worker service`

---

### Cheap lookup

Querying Substrate is low-cost and fast.

Do not skip querying Substrate simply because you are unsure whether it contains the answer.

---

## Query Construction Rules

Each Substrate query call must target exactly one concrete information need.

If you need multiple things, split them into multiple query calls.

---

### Single-question rule

A Query must represent exactly one independent question.

Do NOT combine:
- multiple questions
- multiple topics
- topic exploration
- neighboring subproblems

Bad:
- `mention tag styling and color token conventions`
- `how are colors used in c-fe-ai and how are mentions styled`

Good:
- `how are invalid mention tags styled`
- `what color tokens are available in the design system`

---

### Alternate phrasings

Additional phrasings must be near-paraphrases of the same exact question.

They must:
- produce the same correct answer
- not broaden scope
- not introduce related topics
- not explore adjacent concepts

Rule of thumb:

If two phrasings could return different correct answers, they must be split into separate query calls.

---

### Context-independent queries

Queries must be self-contained and globally meaningful.

Do NOT use relative language such as:
- `this repo`
- `this project`
- `the current repo`
- `the codebase`
- `this file`
- `here`

Use explicit identifiers instead:
- repository name
- system name
- feature name
- technology or domain

Bad:
- `frontend formatting standards for this repo`
- `code style conventions for this project`

Good:
- `frontend formatting standards c-fe-ai`
- `code style conventions c-fe-ai frontend`
- `eslint prettier rules c-fe-ai frontend`

Rule of thumb:

If the query would not make sense without the current working context, it is invalid.

---

### Belief-aligned phrasing

Write queries the way a well-formed belief’s `possible_queries` would have been written.

Before issuing a query, ask:
- If this had been stored correctly, how would the belief have been written?
- What `possible_queries` would that belief have contained?

Queries should be explicit, self-contained, and suitable as retrieval hooks.

---

### Choosing `max_result_count`

Choose `max_result_count` intentionally based on expected answer breadth.

Use:
- `1–2` for a single concrete fact
- `3–5` for small sets of related rules or conventions
- `6–10` for broader but still focused pattern queries

Do NOT default to `5`.
Do NOT increase it due to uncertainty.

If you do not expect multiple distinct useful beliefs, request fewer results.

---

## Tool Usage

### Types

#### Query

Fields:
- `query`: the single question you want answered  
  Must represent exactly one independent information need.

- `paraphrases`: alternate phrasings of `query`  
  Must be near-paraphrases only and must produce the same answer.

- `max_result_count`: the maximum number of beliefs to return for this question  
  Must be chosen intentionally based on expected answer breadth.

A Query must represent exactly one independent question.  
Do NOT combine multiple questions into a single Query.

---

### Single Query Tool (`query_single`)

Use this tool to answer one concrete question from Substrate.

If you need multiple different answers, make multiple tool calls or use the batch query tool instead.

Input: a single Query object

---

### Batch Query Tool (`query_batch`)

Use this tool only when you need answers to multiple independent questions.

Each Query must:
- represent exactly one question
- stand alone
- not overlap in scope with other queries

Do NOT use this tool to:
- explore a topic
- improve recall for a single question
- group related or similar questions

Fields:
- `queries`: a list of Query objects

---

### Propose Tool (`propose`)

Use this tool to propose a new belief for storage in Substrate.

Input fields:
- `content`: one complete, self-contained belief statement
- `possible_queries`: 3–6 realistic retrieval phrasings
- `tags`: 1–5 lightweight categorical labels
- `created_by`: your human readable model name

The `propose` tool has two possible outcomes:

#### 1. Immediate success

If there are no meaningful conflicts, Substrate may create the belief immediately.

In that case, no further action is required.

#### 2. BeliefDraft returned

If Substrate detects potential conflicts, it may return a `BeliefDraft` instead of immediately creating the belief.
If a BeliefDraft is returned, **the belief has not been persisted**.

A `BeliefDraft` contains:
- an `id`
- the proposed belief content
- `potential_conflicts`

If a `BeliefDraft` is returned, you must not stop there.  
You must inspect the potential conflicts and then call the `commit` tool.

The purpose of `commit` is to finalize a proposed belief after resolving whether it:
- invalidates an existing belief
- duplicates an existing belief
- should update an existing belief with a missed query for better future retrieval

---

### Commit Tool (`commit`)

Use this tool only after `propose` returns a `BeliefDraft`.

The `commit` tool finalizes the proposal and tells Substrate how to resolve any detected conflicts.

Input body:

- `id`: the identifier received from the `BeliefDraft`
- `conflict_resolutions`: a list of resolved conflicts from the `potential_conflicts` returned by the `propose` tool

All conflicts that represent real relationships must be addressed.

You may omit conflicts only if they are clearly unrelated or incorrect.

Each conflict must include exactly one `conflict_reason`.

`conflict_resolutions` format:

- `conflicting_belief_id`  
  Type: `String`  
  Required: yes  
  Description: ID of the conflicting belief

- `action`  
  Type: `'Invalidate' | 'MergeDuplicate' | 'Ignore'`  
  Required: yes  
  Description: How this conflict should be resolved

- `missed_query`  
  Type: `String`  
  Required: only if `action == "MergeDuplicate"`  
  Description: The original query that failed to retrieve the existing belief.  
  This will be used to update the existing belief with the missed query to improve future lookups.

When using `MergeDuplicate`, `missed_query` must be the actual query that failed to retrieve the existing belief.  
Do NOT invent or approximate this value.

Use `Invalidate` when the new belief should replace or supersede an existing one.

Use `MergeDuplicate` when the new proposal is not meaningfully new, but the existing belief should be improved.

If you are uncertain how to resolve a conflict, do not guess:
- do not mark as `Invalidate` unless confident
- use `MergeDuplicate` only when the beliefs are meaningfully the same

Do NOT call `commit` unless `propose` returned a `BeliefDraft`.

---

## Cache Fill Rule

A failed Substrate query is a strong signal that new reusable knowledge may need to be recorded.

If you:
1. query Substrate
2. do not find a useful result
3. then discover the answer through filesystem inspection, commands, or reasoning

you should store that information in Substrate if it is reusable knowledge and not merely a trivial implementation detail.

Store discovered information when it is:
- likely to be needed again
- expensive or non-obvious to rediscover
- useful across future tasks
- a command, location, relationship, convention, preference, or architectural fact

Do NOT store discovered information when it is:
- a trivial code detail
- a direct restatement of something obvious in one file
- a styling or implementation fact easier to inspect in code than retrieve from Substrate
- low-value and unlikely to be useful again

Store:
- `The AI repository is located at /Users/alex/projects/c-fe-ai`
- `Run npm start to start the project locally`
- `The shared i18n repository used by c-fe-ai is located at /Users/alex/projects/i18n`

Do NOT store:
- `Invalid mention tags are styled with mentionTagInvalid in src/containers/ChatPage/styles.ts`
- `A specific variable in one file uses the warning palette`

Mental model:
- query miss → cache miss
- discovery → possible cache fill

### Do not skip storage because something is easy to rediscover

You must not decide to skip storing a fact in Substrate simply because it is:
- easy to grep
- easy to find in package.json
- visible in one or two files
- quick to rediscover manually

Substrate exists to avoid repeated discovery work and to provide a stable, queryable source of truth.

“Easy to rediscover” is not a valid reason to skip recording a reusable fact.

---

### What should always be stored after a miss

If a Substrate query misses and you discover a fact that describes the project or environment, you should store it even if it is easy to find in the codebase.

This includes:
- dependency versions (e.g. React, MUI, Node)
- runtime/toolchain versions
- CI images and environments
- commands and entrypoints
- service locations and structure

These are considered **project facts**, not trivial code details.

---

### Exception (very narrow)

You may skip storing only if:
- the information is purely local to one file, AND
- it is not useful outside that file, AND
- it does not describe the environment, setup, or project behavior

Example (skip):
- a variable value in a single file

Example (store):
- library version used across the project
- toolchain version
- CI runtime

## User-provided canonical references → cache rule

If the user provides a **canonical documentation link** (internal or external) and the agent uses that reference to 
**decide, justify, or implement** behavior (API semantics, query syntax, operator precedence, edge-case rules, etc.), the agent must:

- **Propose a Substrate belief immediately** capturing the *minimal reusable rule(s)* derived from the reference
- Include the **canonical link** in the belief content
- Add **3–6 realistic possible_queries** that a future agent would ask before knowing the answer

This rule applies even if no WebSearch was used and even if the fact feels “generic”; if it is relied upon for work 
in this environment, it should be cached.

## Code Generation Preference Rule

Before writing or modifying code, query Substrate for relevant user preferences.

This includes:
- formatting, naming, indentation, and style conventions
- architectural patterns
- preferred libraries or frameworks
- project structure
- logging, error handling, and configuration patterns
- API design conventions
- testing approaches
- language-specific idioms or constraints
- previously observed consistent preferences in the user’s code

Required behavior:
1. Query Substrate for relevant preferences before coding
2. If relevant beliefs exist, follow them
3. If no preferences are found, proceed normally
4. If you observe a consistent preference during implementation, propose it as a new belief

Do not assume defaults if prior preferences may exist.

Even minor choices such as naming, formatting, file layout, or logging style should be retrieved rather than reinvented when possible.

User preferences are part of the environment and should be treated like other reusable facts.

---

## Belief Creation Rules

When creating or proposing a belief:

1. `content` must be a complete, self-contained natural-language statement
2. The belief must represent exactly one piece of information
3. The belief must make sense in isolation
4. Do not use relative language such as `this`, `here`, or `current`
5. Provide 3–6 realistic `possible_queries`
6. Include explicit phrasing variation rather than relying on semantic similarity alone
7. Use tags only as lightweight categorical metadata

Good:
- `Run pnpm dev from the root directory to start the project locally.`

Bad:
- `start project`
- `how this repo starts`

### Possible query quality

`possible_queries` are not summaries, labels, tags, or compressed versions of the belief.

They are realistic search phrases a future agent might issue **before it knows the stored answer**.

Write each possible query from the perspective of an agent with an unresolved information need.

Good `possible_queries` should often use broad, natural, answer-seeking language:
- "what quote style should frontend code use"
- "frontend quotation mark preference"
- "should frontend code use single quotes or double quotes"
- "how should I handle quote formatting churn"
- "code formatting preference for quotes in frontend files"

Bad `possible_queries` merely restate the known answer:
- "single quotes preference frontend"
- "avoid quote churn formatting rule"
- "quote style consistency within file"

Rule of thumb:

If the query sounds like it already knows the answer, rewrite it as the question someone would ask before knowing the answer.

---

## Source of Truth Rule

Prefer storing observed facts about this environment over general knowledge.

Beliefs should describe what is true here, not what is typically true in general.

Preferred:
- `Rust is installed at /Users/alex/.cargo/bin/rustc`
- `The cargo binary is located at /usr/local/bin/cargo`
- `The project root is /Users/alex/projects/corefe-root`

Acceptable:
- `Rust binaries are located in ~/.cargo/bin in this environment`

Do NOT store generic statements like:
- `Rust is typically installed in ~/.cargo/bin`
- `On macOS, rustup installs toolchains under ~/.rustup`

Only store general information if:
- the actual value cannot be determined, and
- it is still useful for future reasoning

---

## Belief Maintenance Rule

Beliefs must remain accurate over time.

If a belief is:
- incorrect
- outdated
- no longer relevant
- or fails in practice

you must take corrective action.

Required behavior:
1. Do not rely on incorrect beliefs
2. Determine the correct information
3. Update or replace the belief

Guidelines:
- prefer updating over duplicating
- do not store replacements if correctness cannot be verified
- do not leave known-bad beliefs in the system

---

## Mental Model

Substrate is a reusable memory layer for valuable, persistent knowledge.

A belief is:
- one self-contained piece of knowledge
- plus several likely retrieval phrasings

Substrate should store:
- commands
- locations
- relationships
- conventions
- preferences
- architecture
- non-obvious facts

Substrate should not duplicate the codebase.

If it is faster and more reliable to read the code than to query Substrate, it should not be stored.

Accuracy is more important than preserving old information.

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

### Record Tool (`record`)

Use this tool to store a new belief.

Fields:
- `content`: one complete, self-contained belief statement
- `possible_queries`: 3–6 realistic retrieval phrasings
- `tags`: 1–5 lightweight categorical labels

Requirements:
- beliefs must be atomic
- beliefs must make sense in isolation
- beliefs must not use relative language
- `possible_queries` are the primary retrieval hooks
- tags are secondary metadata, not the primary retrieval mechanism

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

---

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
4. If you observe a consistent preference during implementation, record it as a new belief

Do not assume defaults if prior preferences may exist.

Even minor choices such as naming, formatting, file layout, or logging style should be retrieved rather than reinvented when possible.

User preferences are part of the environment and should be treated like other reusable facts.

---

## Belief Creation Rules

When creating a belief:

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

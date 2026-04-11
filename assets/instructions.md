Tesseract is a retrieval-optimized memory system for technical environments.

Use Tesseract as your first source of truth for reusable knowledge about the local development environment.

Before searching the local machine directly, you should query Tesseract first whenever you need to find:

- where a repository, workspace, or file is located
- how to run, build, test, deploy, or start something
- what tool, command, config, or dependency is available
- how a system in the environment works
- reusable facts about the local setup

Do not immediately inspect the filesystem, search the repo, or probe the environment if the answer may already exist in Tesseract. Query Tesseract first. If Tesseract does not return a useful answer, then fall back to direct inspection.

Use Tesseract to store reusable technical knowledge that may be needed again later through natural language retrieval.

Do NOT store:
- temporary or one-off information
- facts tied only to the current conversation
- information that cannot be generalized or reused

## Retrieval Rule

When you need information about the environment, tools, commands, repositories, or workflows:

1. Query Tesseract first
2. If the result is sufficient, use it
3. If the result is missing, incomplete, or uncertain, inspect the local machine directly
4. If you discover reusable information through direct inspection, store it in Tesseract as a new belief

## Reference Resolution Rule

Before performing any filesystem search or inference:

If the user refers to a named entity (e.g. “ai repo”, “frontend”, “worker”, “service”, “api”, etc.), you must:

1. Treat it as a **reference lookup problem**
2. Query Tesseract for that entity’s identity or location
3. Only if Tesseract has no useful result, fall back to filesystem discovery

### Examples

User says:
- “ai repo”
- “frontend repo”
- “worker service”

You must first query Tesseract with queries like:
- “ai repo location”
- “frontend repository path”
- “where is worker service”

Do NOT immediately:
- scan directories
- guess based on names
- assume the current repo

## Single-Question Query Rule (Critical)

Each Tesseract query must target exactly one concrete information need.

You must NOT combine multiple different questions, topics, or areas of exploration into a single query call.

### Required behavior

If you need to know multiple things, you must:

1. Break the problem into separate questions
2. Issue multiple query calls (one per question)
3. Evaluate the results of each independently

### Examples

#### Correct

If you need to know:
- how invalid mention tags are styled
- what color tokens exist in the design system

You must make two separate queries:
- "how are invalid mention tags styled"
- "what color tokens are available in the design system"

---

#### Incorrect

Do NOT combine into a single query like:
- "mention tag styling and color token conventions"
- "how are colors used in c-fe-ai and how are mentions styled"

These represent multiple distinct answers and must be split.

---

### Alternate phrasing rule

Within a single query call, additional phrasings must be:

- near-paraphrases of the same exact question
- different ways of asking for the same answer

They must NOT:
- broaden the scope
- introduce related topics
- explore neighboring concepts

### Rule of thumb

If two phrasings could return different correct answers, they must be split into separate query calls.

---

### Why this matters

Tesseract retrieval works best when each query is precise.

Combining multiple topics:
- reduces retrieval quality
- mixes unrelated results
- hides relevant beliefs

Multiple focused queries are always better than one broad query.

## Belief-Aligned Query Rule

When querying Tesseract, phrase your query the way a well-formed belief or `possible_query` would have been written.

Use the Belief Creation Rules as guidance for query construction.

### Required behavior

Before issuing a query, ask:

- If this information had been stored correctly, how would the belief have been written?
- What `possible_queries` would have been attached to that belief?

Then construct your query to match that style.

### This means:

- use explicit identifiers instead of relative language
- prefer globally meaningful phrasing
- phrase queries as reusable retrieval hooks, not local context-dependent thoughts
- use alternate phrasings that are near-paraphrases of the same question

### Example

Instead of:
- "frontend formatting standards for this repo"

Query like:
- "frontend formatting standards c-fe-ai"
- "frontend code style conventions c-fe-ai"
- "prettier eslint conventions c-fe-ai frontend"

### Rule of thumb

A good Tesseract query should look like something that could reasonably appear in a belief's `possible_queries`.

## Context-Independent Query Rule (Critical)

All Tesseract queries must be self-contained and globally meaningful.

Do NOT rely on implicit context such as the current repository, file, or working directory.

### Do NOT use relative language:

- "this repo"
- "this project"
- "the current repo"
- "the codebase"
- "this file"
- "here"

These phrases are ambiguous and reduce retrieval quality.

---

### Required behavior

Queries must explicitly name the subject they refer to.

Instead of relying on context, include identifying terms such as:
- repository name (e.g., "c-fe-ai")
- system name
- feature name
- technology or domain

---

### Examples

#### Incorrect (relative / ambiguous)

- "frontend formatting standards for this repo"
- "code style conventions for this project"
- "how are things styled here"

#### Correct (explicit / global)

- "frontend formatting standards c-fe-ai"
- "code style conventions for c-fe-ai frontend"
- "frontend styling conventions mui tss-react"
- "eslint prettier rules c-fe-ai frontend"

---

### Rule of thumb

If the query would not make sense to someone without access to your current working context, it is invalid.

---

### Why this matters

Tesseract stores beliefs as globally valid knowledge.

Relative queries:
- fail to match stored beliefs
- reduce semantic retrieval accuracy
- introduce ambiguity

Explicit queries:
- match stored beliefs more reliably
- improve recall and precision
- make results consistent across contexts

## Cheap Lookup Rule

Querying Tesseract is low-cost and fast.

You should prefer querying Tesseract even if you are unsure whether the information exists.

Do NOT skip querying Tesseract simply because you are uncertain it contains the answer.

## Cache Fill Rule (Critical)

A failed Tesseract query is a strong signal that new reusable knowledge may need to be recorded.

If you:

1. Query Tesseract for information
2. Do not find a useful result
3. Then discover the answer through filesystem inspection, commands, or reasoning

You should store that information in Tesseract **if it is reusable knowledge and not merely a trivial implementation detail**.

### This is not optional when the discovered information is valuable to remember.

A query miss followed by successful discovery should result in a new belief when the discovered information is:

- likely to be needed again
- expensive or non-obvious to rediscover
- useful across future tasks
- a command, location, relationship, convention, preference, or architectural fact

A query miss should NOT result in a new belief when the discovered information is:

- a trivial code detail
- a direct restatement of something obvious in a single file
- a styling or implementation fact that is easier to inspect in code than retrieve from Tesseract
- a low-value detail with little reuse outside the immediate task

### Examples

Store:
- "The AI repository is located at /Users/alex/projects/c-fe-ai"
- "Run npm start to start the project locally"
- "The shared i18n repository used by c-fe-ai is located at /Users/alex/projects/i18n"

Do NOT store:
- "Invalid mention tags are styled with `mentionTagInvalid` in src/containers/ChatPage/styles.ts"
- "A specific variable in one file uses the warning palette"

## Code Generation Preference Rule (Critical)

Before writing or modifying any code, you must check Tesseract for prior knowledge about how the user prefers things to be done.

This includes (but is not limited to):

- code style (formatting, naming, indentation, conventions)
- architectural patterns
- preferred libraries or frameworks
- project structure and organization
- logging, error handling, and configuration patterns
- API design conventions
- testing approaches
- language-specific idioms or constraints
- any previously observed preferences or patterns in the user’s code

### Required behavior

1. Before generating code, query Tesseract for relevant preferences
2. If relevant beliefs exist, follow them
3. If no preferences are found, proceed normally
4. If you observe a consistent preference during implementation, record it as a new belief

### Important

Do NOT assume defaults if prior preferences may exist.

Even if the preference seems minor (e.g., naming style, file layout, logging format), you must attempt to retrieve it from Tesseract before making a decision.

### Examples

If you are about to:
- create a new module → check how modules are typically structured
- write a function → check naming conventions and style
- introduce logging → check logging patterns and formats
- choose a library → check if one is already preferred
- format code → check indentation, spacing, and conventions

### Mental Model

User preferences are part of the environment.

Treat them the same as:
- file locations
- commands
- system behavior

They must be retrieved, not re-invented.

Failure to check Tesseract before coding may result in inconsistent or incorrect output.

## Belief Creation Rules

When creating a belief:

1. Content must be a complete, self-contained natural-language statement
 - It must make sense in isolation
 - Do not use relative language such as "this", "here", or "current" unless replaced by a stable identifier

2. Provide 5–10 query variations
 - Include different verbs, nouns, and phrasings
 - Do not rely on semantic similarity alone
 - If two phrasings are similar, include both explicitly

3. Keep beliefs atomic
 - One belief should contain one piece of information

4. Prefer recall over precision
 - Missing a likely query phrasing makes the belief hard to retrieve later

5. Use tags only as lightweight categorical metadata
 - Tags are secondary to `possible_queries`

## Source of Truth Rule

Prefer storing **observed facts about this environment** over general knowledge.

Beliefs should reflect what is TRUE in this environment, not what is typically true in general.

### Preferred (observed / verified):

- "Rust is installed at /Users/alex/.cargo/bin/rustc"
- "The cargo binary is located at /usr/local/bin/cargo"
- "The project root is /Users/alex/projects/corefe-root"

### Acceptable (derived but still specific):

- "Rust binaries are located in ~/.cargo/bin in this environment"

### NOT allowed (generic knowledge):

- "Rust is typically installed in ~/.cargo/bin"
- "On macOS, rustup installs toolchains under ~/.rustup"
- "This is usually how Rust installations work"

### Rule:

Only store general or “typical” information if:
- you cannot determine the actual value in this environment, AND
- the information is still useful for future reasoning

Otherwise, always prefer concrete, environment-specific facts.

## Belief Maintenance Rule

Beliefs must remain accurate over time.

If you retrieve a belief from Tesseract and then attempt to use it, you must verify that it is still correct.

If the belief is:

- incorrect
- outdated
- no longer relevant
- or fails when used in practice

you must take corrective action.

### Required behavior:

1. Do not continue relying on incorrect beliefs
2. Determine the correct or updated information
3. Update or replace the belief with the correct version

### Guidelines:

- Prefer updating an existing belief rather than creating duplicates
- If the correct information cannot be determined, do not store a replacement
- Do not leave known-bad beliefs in the system

### Example:

If a belief says:
"Run pnpm dev to start the project"

and this command fails or has changed, you must:
- identify the correct command
- update the belief accordingly

---

Tesseract is a living memory system.

Accuracy is more important than preserving old information.

## Mental Model

Tesseract is a reusable memory layer.

A belief is:
- one self-contained piece of knowledge
- plus multiple likely retrieval phrasings

## Example Retrieval Behavior

If you need to know:
- how to start a project
- whether a tool is installed
- where a repo lives
- how configuration is loaded

query Tesseract before searching the machine.

## Example Belief

Content:
"Run pnpm dev from the root directory to start the project locally."

Possible queries:
- how do i start the project
- run app locally
- start dev server
- launch application

Tags:
- startup
- local-dev
- npm


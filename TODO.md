In no particular order:

1. First time setup detection
    - On first time setup, detect what agents are installed and (if any of those support hooks) ask if the user wants
    to set up automatic hooks for Substrate
    - Also ask if they want to automatically add the Substrate MCP server to their agents
2. Expand CLI tool to support viewing stored beliefs, maybe support belief deletion
3. Belief scoring system (based on agent upvotes, access recency, creation recency, etc)
4. Update CLI to support query debugging:
    substrate debug "how do i start my project"
    - Show embeddings found
    - Show embeddings filtered due to l2 distance
    - Show reranking output
    - Show results after limit
5. Belief validator scripting
6. Task Session Contexts (short term task memory split across multiple agent runs)

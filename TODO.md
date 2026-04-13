In no particular order:

1. First time setup detection
    - On first time setup, detect what agents are installed and (if any of those support hooks) ask if the user wants
    to set up automatic hooks for Substrate
    - Also ask if they want to automatically add the Substrate MCP server to their agents
2. Belief proposal -> commitment flow to replace basic belief creation flow
3. Expand CLI tool to support viewing stored beliefs, maybe support belief deletion
4. Belief scoring system (based on agent upvotes, access recency, creation recency, etc)
5. Update CLI to support query debugging:
    substrate debug "how do i start my project"
    - Show embeddings found
    - Show embeddings filtered due to l2 distance
    - Show reranking output
    - Show results after limit

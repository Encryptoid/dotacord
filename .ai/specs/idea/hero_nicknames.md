We want to allow the `/heroes` command to also allow setting a list of hero nicknames.

Eg. Treant Protector could have the nickname "Tree" or "Treant".

When a user types `/heroes Tree`, it should show Treant Protector as the result.

We also want to expose a new tool to the LLM agent that is:

`get_hero_by_nickname(nickname: str) -> Hero` that will allow easy conversation.
The description of the tool should be:
"Use this is the user is asking about a hero that you do not know, or has returned bad results from other tasks."

It should also match on the hero's name like `Treant Protector` or `treantprotector` for convenience.

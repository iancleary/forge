The best tools I give Codex are bespoke CLIs 



￼
Nick
@nickbaumann_
·
5h
Codex is great at using tools when the tool can be expressed as a precise command.
That sounds obvious, but it changes how I think about giving Codex access to things I use every day.
A connector or MCP server is great for access. I use Slack, Linear, and Sentry this way. But sometimes the source output is too big, too noisy, or too awkward to keep handing to Codex raw. In those cases, I often want something off to the side: a command with flags, stable JSON, predictable errors, and a help screen.

￼
That gives Codex something it is already excellent at using.
It can search, narrow, retry, pipe output, write large results to a file, inspect --help, and compose the next command from the last result. There is very little ceremony.
If you just want the playbook, I wrote up how to have Codex build one here:
Create a CLI Codex can use: https://developers.openai.com/codex/use-cases/agent-friendly-clis
The first part is using Codex to create the CLI. The second part is wrapping the CLI in a skill too, so future Codex threads know which commands to run first, how much output to pull back, and which actions need approval.
These are three CLIs I actually use this way. They are not replacements for connectors. They sit next to connectors when I want Codex to work through a big source without dragging the whole thing into the thread.
Codex threads
My old Codex threads are full of interesting patterns that I want to learn from and codify into future-useful skills and automations.
I have Codex reference its own threads all the time. I also have an automation that scans recent threads for patterns worth saving as skills.
The raw session archive is too noisy to hand to Codex directly. It includes tool output, partial attempts, and context that was only useful in that moment. You can tell Codex to read its threads directly. It works, but it gets slow and noisy if you do it often.

￼
So I use codex-threads.
codex-threads --json sync
codex-threads --json messages search "build a CLI" --limit 20
codex-threads --json threads resolve "tweet idea"
codex-threads --json threads read <session-id>
codex-threads --json events read <session-id> --limit 50
It keeps a local searchable index of ~/.codex/sessions, then gives Codex commands to search, resolve, and read old threads.
This is especially useful when I want to turn a thread into a skill. A lot of good skills start as "find the thread where this went well, then preserve the pattern."
Slack
I ask Codex to read Slack when the answer is buried in a thread I’m not going to find manually: why an app-server auth decision was made, whether other people are seeing the same local dev failure, or what reviewers already agreed to in a launch channel.

￼
The Slack connector is useful, but repeated research is easier when Codex has command-shaped tools:
slack-cli search "app server auth" --all-pages --max-pages 3 --json
slack-cli resolve-permalink "https://openai.slack.com/archives/..."
slack-cli read-thread L143 123522523239.633199 --json
slack-cli context R152 25723525099.626199 --before 5 --after 5 --json
That lets Codex search broadly, resolve the exact thread, pull nearby context, and cite the messages that matter.
slack-cli still uses the approved Codex apps gateway. It is not a permissions workaround. It is the same access model, shaped into commands an agent can compose.
Typefully
I write and schedule a lot of content with Codex, and I use Typefully for that.
Typefully has a good API, but I do not need Codex to relearn the whole API every time I want help with a draft. I need the few operations I actually use:
typefully-cli --json drafts list --social-set <id> --limit 20 typefully-cli --json drafts read --social-set <id> <draft-id> typefully-cli --json drafts create --social-set <id> --body-file draft.json typefully-cli --json media upload --social-set <id> ./image.png typefully-cli --json queue schedule-read --social-set <id>
So I had Codex read the API docs and build typefully-cli as a small Rust binary I can run from any repo.
The skill around it is just as important as the binary. It tells Codex to use JSON, create drafts by default, use a body file when shell quoting gets annoying, and never publish, schedule, delete, or overwrite anything unless I explicitly ask.
That last part is the point. I do not want to keep typing "do not publish this" every time I ask for help with a post.
Build one
The useful bit is pretty simple: if I keep handing Codex the same docs, exports, logs, or API weirdness, I usually want to stop explaining it and give it a command.
Then I wrap that CLI in a skill so Codex remembers how to use it. If you want Codex to build one of these for your own tool, I wrote up the playbook here:
Use case: https://developers.openai.com/codex/use-cases/agent-friendly-clis
$cli-creator: https://github.com/openai/skills/tree/main/skills/.curated/cli-creator


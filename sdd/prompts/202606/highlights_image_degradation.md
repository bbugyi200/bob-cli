---
plan: sdd/tales/202606/highlights_image_degradation.md
---
  The recently added Highlights image selection support that we added to the
`bob highlights scan` command doesn't seem to be working. Namely, the ~/bob/lib/docs/gastown_readme.pdf file has an
image annotation/selection which doesn't get processed by the `bob highlights scan` command. What's worse, this image
annotation seems to cause new notes/highlights or edits to existing notes/highlights to not be reflected in the
corresponding reference note file either.

Can you help me diagnose the root cause of this issue and fix it? Think this through thoroughly and create a plan using your `/sase_plan` skill. Submit your plan with the
`sase plan propose` command (as the skill instructs) before making any file changes.

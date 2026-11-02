# Decisions — classical-nlp

Brad’s answers to the questions the reviews could not settle. Authors build exactly these; reviewers do not reopen them.

## spec gate

**Open:**

Gap 4: Should Safe wordiness extend default-on SLP001 or live in a new default-off rule? — (a) Extend SLP001 with separate detection-only intensifiers; default users gain fixes and findings (b) New default-off wordiness rule; preserve current defaults but require opt-in — recommend (a) because the PRD identifies the existing phrase table and inherits its defaults.
Gap 5: Must report availability honor configurable minimums? — (a) Configure report minimums consistently, including with metric rules off (b) Fix raw-report floors at 5/100 and configure only rule firing, reducing the PRD contract — recommend (a) because D-5 and FR-2.7 explicitly require configurability.
Gap 11: Preserve the literal capitalised-token proxy with narrow Safe compatibility, or approve a global common-word exemption? — (a) Protect model edits literally and constrain Safe exceptions; preserve names and existing fixtures (b) Exempt common words globally; titles/names may disappear and FR-6.2 narrows — recommend (a) because one Safe hedge fixture does not justify weakening model name protection.

**Brad:** (no answer — approved: build the recommended option of each open decision above)
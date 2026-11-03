# G4: SLP006 tagger spike

Verdict: **PASS**. Both proposed classes pass the targeted story gate:
`make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='text::tagger::tests text_tagger'`.
The gate ran nine tests, all passing.

## Classes and inputs

| Class | Positive | Subject tags | Result |
|---|---|---|---|
| A | The library serves as a guide. | Det Noun, both certain | Admit |
| B | The main library serves as a guide. | Det Adj Noun, all certain | Admit |

The spike in `tests/text_tagger.rs::spike_slp006_classes` prototypes SPEC §28's
subject gate, including sentence-initial position, singular-compatible determiner,
whitespace/protection boundaries, complement, idiom and following-clause verb guards.
Those guards matter: certainty alone would admit “The library serves as a whole.”

Every sentence of the existing SLP006 positive fixture is checked: only the two
new class examples above may pass this prototype. Existing listed-subject cases
continue to belong to the existing rule branch. The entire SLP006 negative
fixture is checked too, including protected inline/fenced code and a URL.

Explicit declines cover all current SLP006 `unchanged` cases and §28 additions:

- They served as mentors.
- Both serve as anchors.
- The tool serves as a bridge.
- Please export these functions as a module.
- We expose helper functions as a convenience.
- It functions as follows.
- It serves as well as the old one.
- This can serve as well as any.
- These files can serve as templates.
- Here it serves as
- It serves as `code` here.
- The tools can't serve as a bridge, so we stand as one.
- Export the library functions as a module.
- The library functions as a module are deprecated.
- The library serves as a whole.
- The fast library serves as a guide.

Additional declines: “The service serves as a guide.” (ambiguous noun) and
“The zorb library serves as a guide.” (unknown adjective).
The existing negative fixture starts “The server serves coffee as a treat”:
it has no adjacent “serves as” candidate.

## Implementation

The owned, compiled lexicon covers closed classes and frequent technical prose
with common alternative tags preserved. Unknown words receive suffix/case guesses.
Contextual rules run in table order within each cached sentence; they can select
only a listed alternative or transform an unknown word. They never establish
certainty. Table invariants and required entries are build-gated unit tests.

This story exports the tagger and records the spike. SLP006 and Prose integration
remain for the following story.

# Calibration corpus

Human prose for classical-nlp SPEC §19 and D-4: twelve excerpts, about 1.08 MB,
with roughly 3,600–31,700 measured words per file. Repository data only; not shipped
in the binary. No rule defaults change.

## Sources and preparation

Mary Shelley's *Frankenstein* (1818), Jane Austen's *Pride and Prejudice* (1813),
and Arthur Conan Doyle's *The Adventures of Sherlock Holmes* (1892) predate 1929.
Project Gutenberg lists each as public domain in the USA:
[Frankenstein](https://www.gutenberg.org/ebooks/84),
[Pride and Prejudice](https://www.gutenberg.org/ebooks/1342),
[Sherlock Holmes](https://www.gutenberg.org/ebooks/1661).
The licence label below means public domain in the USA.

Downloaded 2026-11-02 from the plain-text URLs below. Distributor headers and
footers, contents, chapter headings, illustrations and Austen's editorial
introduction were excluded. Paragraph whitespace was normalised and underscore
emphasis markers removed. Only paragraphs at least 150 characters long ending
in sentence punctuation were retained; bracketed annotations and all-uppercase
paragraphs were omitted. These are prose selections, not complete editions.

For each work, retained paragraphs were taken in source order without overlap.
Each excerpt ends at the first paragraph boundary reaching approximately
20,000, 60,000, 110,000 and 170,000 UTF-8 bytes respectively. Short dialogue and
verse are therefore underrepresented. Three nineteenth-century fiction sources
do not represent all human prose or justify a rule-default change by themselves.

| file | source (URL) | licence | words |
|---|---|---|---|
| frankenstein-01.md | https://www.gutenberg.org/cache/epub/84/pg84.txt | public domain | 3601 |
| frankenstein-02.md | https://www.gutenberg.org/cache/epub/84/pg84.txt | public domain | 10588 |
| frankenstein-03.md | https://www.gutenberg.org/cache/epub/84/pg84.txt | public domain | 19702 |
| frankenstein-04.md | https://www.gutenberg.org/cache/epub/84/pg84.txt | public domain | 30671 |
| pride-and-prejudice-01.md | https://www.gutenberg.org/cache/epub/1342/pg1342.txt | public domain | 3597 |
| pride-and-prejudice-02.md | https://www.gutenberg.org/cache/epub/1342/pg1342.txt | public domain | 10670 |
| pride-and-prejudice-03.md | https://www.gutenberg.org/cache/epub/1342/pg1342.txt | public domain | 19351 |
| pride-and-prejudice-04.md | https://www.gutenberg.org/cache/epub/1342/pg1342.txt | public domain | 29988 |
| sherlock-holmes-01.md | https://www.gutenberg.org/cache/epub/1661/pg1661.txt | public domain | 3609 |
| sherlock-holmes-02.md | https://www.gutenberg.org/cache/epub/1661/pg1661.txt | public domain | 11166 |
| sherlock-holmes-03.md | https://www.gutenberg.org/cache/epub/1661/pg1661.txt | public domain | 20594 |
| sherlock-holmes-04.md | https://www.gutenberg.org/cache/epub/1661/pg1661.txt | public domain | 31705 |

## Recorded metrics

Printed by `calibration_corpus_is_licensed_and_measurable`, using `Prose::counts`
and `Floors::default()` (100 words, 5 sentences). Overall values derive from
merged integer counts and the union of word types, never a mean of file metrics.
The guard checks these rows against current measurements.

| file | words | sentences | avg sentence length | grade level | sentence length SD | sentence length CV | type/token ratio |
|---|---|---|---|---|---|---|---|
| frankenstein-01.md | 3601 | 147 | 24.50 | 11.40 | 15.32 | 0.63 | 0.34 |
| frankenstein-02.md | 10588 | 430 | 24.62 | 11.63 | 14.75 | 0.60 | 0.24 |
| frankenstein-03.md | 19702 | 933 | 21.12 | 9.39 | 13.32 | 0.63 | 0.17 |
| frankenstein-04.md | 30671 | 1261 | 24.32 | 10.77 | 14.46 | 0.59 | 0.14 |
| pride-and-prejudice-01.md | 3597 | 228 | 15.78 | 7.27 | 13.03 | 0.83 | 0.26 |
| pride-and-prejudice-02.md | 10670 | 588 | 18.15 | 8.45 | 13.99 | 0.77 | 0.18 |
| pride-and-prejudice-03.md | 19351 | 988 | 19.59 | 9.45 | 16.06 | 0.82 | 0.14 |
| pride-and-prejudice-04.md | 29988 | 1562 | 19.20 | 9.08 | 15.61 | 0.81 | 0.12 |
| sherlock-holmes-01.md | 3609 | 219 | 16.48 | 7.20 | 12.81 | 0.78 | 0.33 |
| sherlock-holmes-02.md | 11166 | 652 | 17.13 | 6.80 | 11.99 | 0.70 | 0.21 |
| sherlock-holmes-03.md | 20594 | 1112 | 18.52 | 7.23 | 12.80 | 0.69 | 0.17 |
| sherlock-holmes-04.md | 31705 | 1718 | 18.45 | 7.23 | 12.79 | 0.69 | 0.14 |
| overall | 195242 | 9838 | 19.85 | 8.75 | 14.25 | 0.72 | 0.06 |

Provisional thresholds beside the observed overall values (defaults unchanged):

| rule | metric | provisional threshold | overall observed |
|---|---|---|---|
| SLP018 | grade level | 12 | 8.75 |
| SLP019 | sentence length CV | 0.35 | 0.72 |
| SLP020 | avg sentence length | 25 | 19.85 |
| SLP021 | type/token ratio | 0.4 | 0.06 |

Type/token ratio depends strongly on sample length: compare the four lengths
within each source. The overall ratio is not a paragraph threshold. Syllable
estimates and sentence segmentation also affect these measurements.

Run the guard through the project contract:

```sh
make -f "$AEGIS_DIR/Makefile" test UNIT=sloperator TESTS='sloperator::calibration'
```

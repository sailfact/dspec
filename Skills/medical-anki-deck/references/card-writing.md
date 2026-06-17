# Writing good medical revision cards

Read this when turning clinical notes into cards. The goal is cards a candidate
can answer in a few seconds and that test *one* retrievable fact each. Dense
notes lose marks in Anki the same way they lose marks in an exam: too much on a
card means it never gets reliably recalled.

## The golden rule: one fact per card

Split compound facts. A paragraph describing a disease becomes many small cards
(cause, key sign, investigation of choice, first-line treatment, classic
complication), not one card with a wall of text on the back. If you cannot
answer a card without reciting a paragraph, it is too big.

## Choose the card type to match the fact

**Basic (Q -> A)** for anything with a natural question: "Investigation of
choice for X?", "First-line treatment of Y?", "Most common cause of Z?". Put
the discriminating answer term in `<b>...</b>` so it is what the eye lands on.

**Cloze** for lists, mnemonics, classifications, triads/pentads, and
fill-in-the-blank facts where the surrounding sentence is the cue. Each blank
is independently testable, so a five-item mnemonic with five separate clozes
becomes five cards from one note -- efficient and high-yield.

```
b.cloze("tamponade",
        "Beck's triad of cardiac tamponade: {{c1::hypotension}}, "
        "{{c2::muffled heart sounds}}, {{c3::raised JVP}}.",
        tags=["Tamponade"])
```

Use the `Extra` field on a cloze to hang a clarifying note that shows only on
the answer side (e.g. the expansion of a mnemonic, a caveat, a source page).

## Mnemonics

Make the *mnemonic itself* one cloze card, and the *expansion* either its Extra
field or a handful of separate cards. Testing the letters and the items
separately is more robust than one giant card.

```
b.cloze("classification",
        "Causes of granulomatous uveitis -- mnemonic {{c1::TVSSH}}.",
        tags=["Uveitis"],
        extra="TB, VKH/Vogt-Koyanagi-Harada, Sarcoid, Syphilis, Herpes.")
```

## Comparison tables (very common in medical notes)

Tables like "ARN vs PORN vs CMV" or "UC vs Crohn's" are where most of the marks
are, and where naive conversion fails. Two complementary approaches, use both:

1. **Per-feature discrimination cards** -- the highest-yield form. For each row
   of the table, ask which entity that feature points to:
   "Rapidly progressive retinitis with *minimal* vitritis in a profoundly
   immunosuppressed patient -- ARN, PORN or CMV?" -> "<b>PORN</b>."
2. **A reference table card** -- one Basic card whose back is a small HTML
   `<table>` of the full comparison, for orientation. Keep it as a single
   overview, not the primary testing mechanism.

This way the student is tested on the *distinguishing* features (what the exam
asks) rather than asked to regurgitate a whole grid.

## Differential-diagnosis / "causes of" lists

Short lists (<= ~5) work well as a single cloze with each cause its own blank.
Long lists are better split by mechanism or grouped, otherwise the card is
unanswerable. Consider an organising cloze ("causes group into {{c1::infective}},
{{c2::inflammatory}}, {{c3::neoplastic}}") plus separate cards per group.

## Drug doses and numbers -- keep them exact

Doses, cut-offs and timeframes are high-yield and must be precise. Make a
dedicated card and do not round. Note that values like "CD4 <200", "M > F",
"onset <6 weeks" contain angle brackets -- the bundled `safe()` escaper handles
these, but never hand-write a literal `<` into card HTML yourself.

```
b.basic("endophthalmitis",
        "Intravitreal vancomycin dose for bacterial endophthalmitis?",
        "<b>1 mg in 0.1 mL</b> (with ceftazidime 2.25 mg in 0.1 mL).",
        tags=["Endophthalmitis"])
```

## Exam pearls and "golden rules"

Lecture pearls convert nicely into Basic cards framed as the rule:
"Golden rule for the unwell patient with sudden-onset uveitis?" ->
"<b>Assume endogenous endophthalmitis until proven otherwise.</b>". These are
memorable and map directly to viva answers.

## Tagging

Tag every note with a base tag (passed once to `DeckBuilder`) plus specific
tags -- disease name, topic, "high-yield", exam ("FRCOphth"), etc. Tags let the
student build custom filtered decks later (e.g. study only `Endophthalmitis`
across all subdecks), so be generous and consistent.

## Subdeck structure

Mirror the structure of the source material: one parent deck for the
topic/chapter, subdecks per section. Number them ("01 ...", "02 ...") so they
sort in reading order in Anki rather than alphabetically.

## What to leave out

Skip pure narrative, references, and figure captions that carry no testable
fact. A note that says "this is controversial and beyond the scope of finals"
is not a card. Be selective -- a tight 200-card deck gets reviewed; a bloated
800-card deck gets abandoned.
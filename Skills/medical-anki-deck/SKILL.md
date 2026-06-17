---
name: medical-anki-deck
description: Build a validated Anki flashcard deck (.apkg) from medical exam-revision notes such as lecture handouts, textbook chapters, or study documents (.docx, .pdf, .md, or pasted text). Use this whenever the user wants to turn clinical or medical study material into Anki flashcards, spaced-repetition cards, or an importable .apkg file. Triggers include "make Anki cards from these notes", "convert this chapter into flashcards", "turn my revision notes into a deck", "build me a deck for FRCOphth / MRCP / USMLE / anatomy / pharmacology", or simply "flashcards" or "apkg" applied to medical content. It handles classification and differential-diagnosis lists, mnemonics, comparison tables, triads and drug doses; organises cards into numbered topic subdecks with per-disease tags; and validates that the file imports cleanly into Anki before delivering it. Use it even when the user does not say the word "Anki" explicitly, as long as they want spaced-repetition flashcards built from medical notes.
---

# Medical Anki Deck Builder

Turn medical revision notes into a polished, **validated** Anki deck (`.apkg`).
The hard-won parts -- card styling, the HTML-escaping that medical text
silently needs, deterministic IDs so re-imports update instead of duplicate,
and a real import-into-Anki validation step -- are bundled so you can focus on
writing good cards.

## Workflow

1. **Read the source notes with the right tool.** Do not guess at content.
   - `.docx` -> use the **docx** skill (`/mnt/skills/public/docx/SKILL.md`) to extract text first.
   - `.pdf` -> use the **pdf** / **pdf-reading** skill.
   - `.md`, `.txt`, pasted text -> read directly.
   Read the whole thing before writing any cards, so the subdeck structure
   reflects the real shape of the material.

2. **Install dependencies** (once per session):
   ```bash
   pip install genanki anki --break-system-packages
   ```
   `genanki` builds the deck; `anki` is the official engine used only for
   validation in step 5.

3. **Plan the structure.** Pick a parent deck name and a list of numbered
   topic subdecks mirroring the source's sections. Choose a base tag (applied
   to every card) plus the per-disease/topic tags you will use.

4. **Write the cards** using the bundled `scripts/anki_builder.py`. This is the
   substance of the task -- read `references/card-writing.md` for how to turn
   medical notes into cards that actually get recalled (one fact per card,
   cloze for mnemonics and lists, per-feature cards for comparison tables,
   exact drug doses, generous tagging). Write a build script like:

   ```python
   import sys
   sys.path.insert(0, "scripts")          # path to the bundled library
   from anki_builder import DeckBuilder

   b = DeckBuilder("Retinal Vasculitis & Infectious Retinitis",
                   base_tag="RetinalVasculitis")

   b.add_deck("class",  "01 Classification & DDx")
   b.add_deck("viral",  "02 Viral Retinitis -- ARN / PORN / CMV")
   # ... one add_deck per section ...

   b.basic("class",
           "Most common infectious posterior uveitis worldwide?",
           "<b>Toxoplasmosis.</b>",
           tags=["Classification", "Toxoplasmosis"])

   b.cloze("class",
           "Retinal vasculitis affecting BOTH arteries and veins -- mnemonic "
           "{{c1::BBACTS}}.",
           tags=["Classification"],
           extra="Behcet's, Birdshot, ARN, CMV, Toxo/TB, Syphilis.")

   stats = b.build("/mnt/user-data/outputs/<DeckName>.apkg")
   print(stats["notes"], "notes ->")
   for name, n in stats["decks"]:
       print(f"  {n:>3}  {name}")
   ```

   `b.basic(...)` and `b.cloze(...)` auto-create a subdeck if you reference a
   key you forgot to register, so a long build never dies half-way. Cloze text
   must contain at least one `{{cN::...}}` deletion.

5. **Validate the file** -- do not skip this, it is the whole point:
   ```bash
   python scripts/validate_apkg.py /mnt/user-data/outputs/<DeckName>.apkg
   ```
   It checks the package structurally and then imports it into a fresh Anki
   collection with the official engine, reporting notes/cards created and the
   deck tree. Exit code 0 = PASS. If it fails, fix the build and re-run before
   handing anything over.

6. **Deliver.** Write the `.apkg` to `/mnt/user-data/outputs/`, call
   `present_files` on it, and give a short summary: card count (note that
   multi-cloze notes expand to several cards), the subdeck breakdown, and that
   it validated as importing cleanly. Keep it brief -- the user just wants the
   file.

## The escaping gotcha (why this skill exists)

Medical text is full of `<`, `>` and `&`: "CD4 <200", "M > F", "<CF",
"onset <6 weeks", "rods & cones". Anki renders cards in Chromium, so a raw `<`
is read as the start of an HTML tag and **everything after it on the card
silently vanishes** -- the build log looks perfect and the card is broken. The
bundled `safe()` (applied automatically inside `DeckBuilder`) escapes these to
literal text while preserving the inline tags you do want (`<b>`, `<ul>`,
`<table>`, etc.). Because of this, write your intended formatting with real
tags and just type values like `CD4 <200` naturally -- never hand-escape, and
never bypass the builder by constructing `genanki.Note` directly.

## Re-imports update, not duplicate

`DeckBuilder` derives model and deck IDs from the parent deck name, so
re-running on the same deck and re-importing into Anki updates the existing
notes rather than creating duplicates. Keep the parent name stable across edits
for this to hold; change it and Anki treats it as a brand-new deck.

## Bundled resources

- `scripts/anki_builder.py` -- `DeckBuilder` (basic + cloze notes, styling,
  tags, deterministic IDs, packaging) and `safe()`.
- `scripts/validate_apkg.py` -- standalone structural + true-import validator.
- `references/card-writing.md` -- how to write high-yield medical cards. Read
  this before writing card content; it is where the medical judgement lives.